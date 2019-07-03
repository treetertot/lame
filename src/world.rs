use crate::entity::{Entity, Action};
use std::sync::Arc;
use crossbeam::atomic::AtomicCell;
use crossbeam::channel::{Sender, Receiver, unbounded, bounded};
use std::thread;
use std::iter::Iterator;
use std::time::Instant;

pub struct World<E: Entity> {
    num_entities: Vec<AtomicCell<usize>>,
    channels: Vec<Sender<E::Template>>,
    pub shared: E::Shared,
    frames: Vec<Receiver<E::Drawer>>,
}
impl<E: Entity> World<E> {
    pub fn add_entity(&self, template: E::Template) {
        let mut smallest = (0usize, self.num_entities[0].load());
        for i in 1..self.num_entities.len() {
            let current = self.num_entities[i].load();
            if current < smallest.1 {
                smallest = (i, current);
            }
        }
        self.num_entities[smallest.0].store(smallest.1 + 1);
        self.channels[smallest.0].send(template).unwrap();
    }
    #[inline]
    pub fn total_entities(&self) -> usize {
        let mut sum = 0;
        for n in self.num_entities.iter() {
            sum += n.load();
        }
        sum
    }
    pub fn new(shared: E::Shared) -> Arc<Self> {
        //todo: get cpus and build corresponding World
        let cpus = num_cpus::get();
        let mut entity_counters = Vec::with_capacity(cpus);
        let mut ch_recievers: Vec<Receiver<E::Template>> = Vec::new();
        let mut ch_senders: Vec<Sender<E::Template>> = Vec::new();
        let mut f_senders: Vec<Sender<E::Drawer>> = Vec::new();
        let mut f_recievers: Vec<Receiver<E::Drawer>> = Vec::new();
        for _ in 0..cpus {
            entity_counters.push(AtomicCell::new(0usize));
            let (s, r) = unbounded();
            ch_recievers.push(r);
            ch_senders.push(s);
            let (s, r) = unbounded();
            f_recievers.push(r);
            f_senders.push(s);
        }
        let w = Arc::new(World{
            num_entities: entity_counters,
            channels: ch_senders,
            shared: shared,
            frames: f_recievers,
        });
        for i in 0..cpus {
            update(w.clone(), ch_recievers.pop().unwrap(), f_senders.pop().unwrap(), i);
        }
        w
    }
    pub fn init(s: E::Shared, starters: Vec<E::Template>) -> Arc<Self> {
        let cpus = num_cpus::get();
        let mut temp_entity_counters = Vec::with_capacity(cpus);
        let mut ch_recievers: Vec<Receiver<E::Template>> = Vec::new();
        let mut ch_senders: Vec<Sender<E::Template>> = Vec::new();
        let mut f_senders: Vec<Sender<E::Drawer>> = Vec::new();
        let mut f_recievers: Vec<Receiver<E::Drawer>> = Vec::new();
        for _ in 0..cpus {
            temp_entity_counters.push(0usize);
            let (s, r) = unbounded();
            ch_recievers.push(r);
            ch_senders.push(s);
            let (s, r) = bounded(starters.len()/4);
            f_recievers.push(r);
            f_senders.push(s);
        }
        for (i, temp) in starters.into_iter().enumerate() {
            let dest = i % cpus;
            temp_entity_counters[dest] += 1;
            ch_senders[dest].send(temp).unwrap();
        }
        let mut entity_counters = Vec::with_capacity(cpus);
        for count in temp_entity_counters {
            entity_counters.push(AtomicCell::new(count));
        }
        let w = Arc::new(World{
            num_entities: entity_counters,
            channels: ch_senders,
            shared: s,
            frames: f_recievers,
        });
        for i in 0..cpus {
            update(w.clone(), ch_recievers.pop().unwrap(), f_senders.pop().unwrap(), i);
        }
        w
    }
    pub fn iter_draws<'a>(&'a self) -> DrawIter<'a, E> {
        let mut left = Vec::with_capacity(self.num_entities.len());
        for n in self.num_entities.iter() {
            left.push(n.load());
        }
        DrawIter {
            world: self,
            left: left,
            current: 0,
        }
    }
}

fn update<E: Entity>(world: Arc<World<E>>, entity_source: Receiver<E::Template>, frames: Sender<E::Drawer>, me: usize) {
    thread::spawn(move || {
        let mut entities = Vec::new();
        let mut time = Instant::now();
        loop {
            for temp in entity_source.try_iter() {
                entities.push(E::construct(&temp, &world));
            }
            let mut to_remove = Vec::new();
            let delta = time.elapsed().as_micros() as f32 / 1000000.0;
            time = Instant::now();
            for (i, entity) in entities.iter_mut().enumerate() {
                match entity.update(&world, delta) {
                    Action::Draw(drawing) => frames.send(drawing).unwrap(),
                    Action::Kill => {to_remove.push(i); world.num_entities[me].store(world.num_entities[me].load() + 1)},
                }
            }
            if to_remove.len() != 0 {
                let mut shifted = 0;
                for n in to_remove {
                    entities.remove(n + shifted);
                    shifted += 1;
                }
            }
        }
    });
}

pub struct DrawIter<'a, E: Entity> {
    world: &'a World<E>,
    left: Vec<usize>,
    current: usize,
}
impl<'a, E: Entity> Iterator for DrawIter<'a, E> {
    type Item = E::Drawer;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current == self.left.len() {
            return None;
        }
        if self.left[self.current] == 0 {
            self.current += 1;
            return self.next();
        }
        self.left[self.current] -= 1;
        Some(self.world.frames[self.current].recv().unwrap())
    }
}
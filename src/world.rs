use crate::entity::{Action, Entity};
use crossbeam_channel::{bounded, unbounded, Receiver, Sender};
use std::iter::Iterator;
use std::ops::{Deref, Drop};
use std::sync::{Arc, RwLock, Weak};
use std::thread;
use std::time::Instant;
/// World is a type for communicating with threads
pub struct World<E: Entity> {
    num_entities: RwLock<Vec<usize>>,
    channels: Vec<Sender<Option<E::Template>>>,
    pub shared: E::Shared,
    frames: Vec<Receiver<E::Drawer>>,
}
impl<E: Entity> World<E> {
    pub fn add_entity(&self, template: E::Template) {
        let mut num_entities = self.num_entities.write().unwrap();
        let mut smallest = (0usize, num_entities[0]);
        for i in 1..num_entities.len() {
            let current = num_entities[i];
            if current < smallest.1 {
                smallest = (i, current);
            }
        }
        num_entities[smallest.0] = smallest.1 - 1;
        self.channels[smallest.0].send(Some(template)).unwrap();
    }
    #[inline]
    pub fn total_entities(&self) -> usize {
        let entities = self.num_entities.read().unwrap();
        let mut sum = entities[0];
        for n in entities.iter().skip(1) {
            sum += n;
        }
        sum
    }
    pub fn init(s: E::Shared, starters: Vec<E::Template>) -> LameHandle<E> {
        let cpus = num_cpus::get();
        let mut temp_entity_counters = Vec::with_capacity(cpus);
        let mut ch_recievers: Vec<Receiver<Option<E::Template>>> = Vec::new();
        let mut ch_senders: Vec<Sender<Option<E::Template>>> = Vec::new();
        let mut f_senders: Vec<Sender<E::Drawer>> = Vec::new();
        let mut f_recievers: Vec<Receiver<E::Drawer>> = Vec::new();
        for _ in 0..cpus {
            temp_entity_counters.push(0usize);
            let (s, r) = unbounded();
            ch_recievers.push(r);
            ch_senders.push(s);
            let (s, r) = bounded(starters.len() / 4);
            f_recievers.push(r);
            f_senders.push(s);
        }
        for (i, temp) in starters.into_iter().enumerate() {
            let dest = i % cpus;
            temp_entity_counters[dest] += 1;
            ch_senders[dest].send(Some(temp)).expect("failed to send template");
        }
        let entity_counters = RwLock::new(temp_entity_counters);
        let w = Arc::new(World {
            num_entities: entity_counters,
            channels: ch_senders,
            shared: s,
            frames: f_recievers,
        });
        for i in 0..cpus {
            update(
                w.clone(),
                ch_recievers.pop().unwrap(),
                f_senders.pop().unwrap(),
                i,
            );
        }
        LameHandle { world: w }
    }
    pub fn iter_draws<'a>(&'a self) -> DrawIter<'a, E> {
        let entities = self.num_entities.read().unwrap();
        let mut left = Vec::with_capacity(entities.len());
        for (i, n) in entities.iter().enumerate() {
            left.push((i, *n));
        }
        DrawIter {
            world: self,
            left: left,
        }
    }
}

/// Essentially an owning handle for the world
/// Destroys threads on drop
pub struct LameHandle<E: Entity> {
    world: Arc<World<E>>,
}
impl<E: Entity> Deref for LameHandle<E> {
    type Target = World<E>;
    fn deref(&self) -> &World<E> {
        &self.world
    }
}
impl<E: Entity> Drop for LameHandle<E> {
    fn drop(&mut self) {
        for ch in self.channels.iter() {
            ch.send(None).unwrap()
        }
    }
}
impl<E: Entity> LameHandle<E> {
    pub fn get_arc(&self) -> Arc<World<E>> {
        self.world.clone()
    }
    pub fn get_weak(&self) -> Weak<World<E>> {
        Arc::downgrade(&self.world)
    }
}

fn update<E: Entity>(
    world: Arc<World<E>>,
    entity_source: Receiver<Option<E::Template>>,
    frames: Sender<E::Drawer>,
    me: usize,
) {
    thread::Builder::new().name(String::from("eggs")).spawn(move || {
        let mut entities = Vec::new();
        let mut time = Instant::now();
        loop {
            for temp in entity_source.try_iter() {
                if let Some(temp) = temp {
                    entities.push(E::construct(temp, &world.shared));
                } else {
                    break;
                }
            }
            let mut to_remove = Vec::new();
            let delta = time.elapsed().as_micros() as f32 / 1000000.0;
            time = Instant::now();
            for (i, entity) in entities.iter_mut().enumerate() {
                match entity.update(&world, delta) {
                    Action::Draw(drawing) => frames.send(drawing).expect("failed to send sprite"),
                    Action::Kill => {
                        to_remove.push(i);
                        world.num_entities.write().expect("failed to unlock rwlock and record kill")[me] += 1;
                    }
                }
            }
            if to_remove.len() != 0 {
                let mut shifted = 0;
                for n in to_remove {
                    entities.remove(n - shifted);
                    shifted += 1;
                }
            }
        }
    }).unwrap();
}

/// Allows iterating through Drawers
pub struct DrawIter<'a, E: Entity> {
    world: &'a World<E>,
    left: Vec<(usize, usize)>,
}
impl<'a, E: Entity> Iterator for DrawIter<'a, E> {
    type Item = E::Drawer;
    fn next(&mut self) -> Option<Self::Item> {
        self.left.retain(|&(_index, count)| count != 0);
        let mut i = 0;
        if self.left.len() == 0 {
            return None;
        }
        loop {
            match self.world.frames[self.left[i].0].try_recv() {
                Ok(val) => {
                    self.left[i].1 -= 1;
                    return Some(val);
                }
                _ => (),
            }
            i = (i + 1) % self.left.len();
        }
    }
}

pub struct FrozenWorld<E: Entity> {
    pub shared: E::Shared,
    pub templates: Vec<E::Template>
}
impl<E: Entity> FrozenWorld<E> {
    pub fn start(self) -> LameHandle<E> {
        World::init(self.shared, self.templates)
    }
}

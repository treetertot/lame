use crate::entity::{Action, Entity};
use crossbeam_channel::{bounded, Receiver, Sender};
use std::collections::BinaryHeap;
use std::iter;
use std::mem;
use std::ops::Deref;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Instant;

mod capsule;
use capsule::Capsule;

pub struct World<E: Entity> {
    num_entities: RwLock<Vec<usize>>,
    channels: Vec<Sender<E::Template>>,
    pub shared: E::Shared,
    frames: Vec<Receiver<Capsule<E::Drawer>>>,
    kill: RwLock<bool>,
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
        self.channels[smallest.0].send(template).unwrap();
    }
    fn drain<'a>(&'a self) -> DrainIter<'a, E> {
        let entities = self.num_entities.read().unwrap();
        let mut left = Vec::with_capacity(entities.len());
        for (i, n) in entities.iter().enumerate() {
            left.push((i, *n));
        }
        DrainIter {
            world: self,
            left: left,
        }
    }
    pub fn iter_draws(&self) -> DrawIter<E::Drawer> {
        let mut heap = BinaryHeap::new();
        heap.extend(self.drain());
        DrawIter(heap)
    }
}
fn update<E: Entity>(
    world: Arc<World<E>>,
    index: usize,
    entities_in: Receiver<E::Template>,
    frames: Sender<Capsule<E::Drawer>>,
) {
    thread::spawn(move || {
        let mut entities = Vec::new();
        let mut time = Instant::now();
        while !*world.kill.read().unwrap() {
            for template in entities_in.try_iter() {
                entities.push(E::construct(template, &world.shared));
            }
            let mut to_remove = Vec::new();
            let delta = mem::replace(&mut time, Instant::now())
                .elapsed()
                .as_micros() as f32
                / 1000000.0;
            for (i, entity) in entities.iter_mut().enumerate() {
                match entity.update(&world, delta) {
                    Action::Draw(layer, drawing) => frames
                        .send(Capsule {
                            layer: layer,
                            data: drawing,
                        })
                        .expect("failed to send sprite"),
                    Action::Kill => {
                        to_remove.push(i);
                        world
                            .num_entities
                            .write()
                            .expect("failed to unlock rwlock and record kill")[index] -= 1;
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
    });
}

pub fn init<E: Entity>(
    mut start_list: Vec<E::Template>,
    num_threads: usize,
    shared: E::Shared,
) -> WorldHandle<E> {
    let last = start_list.len() - ((start_list.len() / num_threads) * (num_threads - 1));
    //Yes, this line is stupid. I just wanted to see if I could do it without making a mutable variable.
    let temp_counters: Vec<_> = (0..num_threads - 1)
        .map(|_n| start_list.len() / num_threads)
        .chain(iter::once(last))
        .collect();
    let (esenders, ereceivers): (Vec<_>, Vec<_>) =
        temp_counters.iter().map(|&n| bounded(n)).unzip();
    let (fsenders, freceivers): (Vec<_>, Vec<_>) =
        temp_counters.iter().map(|&n| bounded(n)).unzip();

    for (&count, sender) in temp_counters.iter().zip(esenders.iter()) {
        for _ in 0..count {
            sender.send(start_list.pop().unwrap()).unwrap();
        }
    }

    let w = Arc::new(World {
        num_entities: RwLock::new(temp_counters),
        channels: esenders,
        frames: freceivers,
        shared: shared,
        kill: RwLock::new(false),
    });

    for (i, (receiver, frameout)) in ereceivers.into_iter().zip(fsenders).enumerate() {
        update(w.clone(), i, receiver, frameout)
    }

    WorldHandle(w)
}

pub struct WorldHandle<E: Entity>(Arc<World<E>>);
impl<E: Entity> Drop for WorldHandle<E> {
    fn drop(&mut self) {
        *self.0.kill.write().unwrap() = true;
    }
}
impl<E: Entity> Deref for WorldHandle<E> {
    type Target = World<E>;
    fn deref(&self) -> &World<E> {
        &self.0
    }
}
impl<E: Entity> WorldHandle<E> {
    pub fn get_arc(&self) -> Arc<World<E>> {
        self.0.clone()
    }
}

/// Allows iterating through Drawers
struct DrainIter<'a, E: Entity> {
    world: &'a World<E>,
    left: Vec<(usize, usize)>,
}
impl<'a, E: Entity> Iterator for DrainIter<'a, E> {
    type Item = Capsule<E::Drawer>;
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
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.left.iter().fold(0, |last, (_, count)| last + count);
        (remaining, Some(remaining))
    }
}
impl<'a, E: Entity> ExactSizeIterator for DrainIter<'a, E> {}

pub struct DrawIter<T>(BinaryHeap<Capsule<T>>);
impl<T> Iterator for DrawIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        Some(self.0.pop()?.data)
    }
}

pub struct Frozen<E: Entity> {
    pub threads: usize,
    pub templates: Vec<E::Template>,
    pub shared: E::Shared,
}
impl<E: Entity> Frozen<E> {
    pub fn start(self) -> WorldHandle<E> {
        init(self.templates, self.threads, self.shared)
    }
}

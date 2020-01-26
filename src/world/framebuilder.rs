use parking_lot::Mutex;
use crate::entity::{Entity, Action};
use crate::world::{WeakWorld, capsule::Capsule};
use std::thread::{current, ThreadId};

pub struct Frame<E: Entity> {
    threads: usize,
    done: Mutex<Vec<ThreadId>>,
    drawings: Mutex<Vec<Vec<Capsule<E::Drawer>>>>,
}
impl<E: Entity> Frame<E> {
    pub fn new(threads: usize) -> Self {
        Frame {
            threads,
            done: Mutex::new(Vec::with_capacity(threads)),
            drawings: Mutex::new(Vec::with_capacity(threads)),
        }
    }
    pub fn run(&self, entities: &mut Vec<E>, world: &WeakWorld<E>, delta: f32) -> bool {
        let mut out = Vec::with_capacity(entities.len());
        let id = current().id();
        let mut lock = self.done.lock();
        let ret = if lock.contains(&id) {
            return false;
        } else {
            lock.push(id);
            if lock.len() == self.threads {
                true
            } else {
                false
            }
        };
        let mut index = 0;
        while index < entities.len() {
            match entities[index].update(world, delta) {
                Action::Draw(layer, data) => {
                    out.push(Capsule{layer, data});
                    index += 1;
                },
                Action::Wait => index += 1,
                Action::Kill => {entities.remove(index);},
            }
        }
        self.drawings.lock().push(out);
        ret
    }
    pub fn collapse(self) -> Vec<Capsule<E::Drawer>> {
        let mut out: Vec<_> = self.drawings.into_inner().into_iter().flatten().collect();
        out.sort();
        out
    }
}

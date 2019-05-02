use std::sync::{RwLock, RwLockReadGuard, Arc};
use std::thread;

use crate::entity::{Entity};

#[derive(Clone)]
pub struct World<E: Entity> {
    pub entities: Arc<RwLock<Vec<RwLock<E>>>>,
}

impl<E: Entity> World<E> {
    pub fn new() -> World<E> {
        let world = World{entities: Arc::new(RwLock::new(Vec::new()))};
        for i in 0..2 {
            let world = world.clone();
            thread::spawn(move || {
                world.run_offset(i, 3)
            });
        }
        world
    }
    pub fn push(&self, entity: E) {
        self.entities.write().unwrap().push(RwLock::new(entity));
    }
    pub fn kill(&self, number: usize) {
        self.entities.write().unwrap().remove(number);
    }
    pub fn read_list(&self) -> RwLockReadGuard<Vec<RwLock<E>>> {
        self.entities.read().unwrap()
    }
    pub fn run_offset(&self, start: usize, amount: usize) {
		let list = self.read_list();
		for i in (start..list.len()).step_by(amount) {
			E::update(i, self);
		}
	}
}

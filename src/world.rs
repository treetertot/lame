use std::sync::{RwLock, RwLockReadGuard};

use crate::entity::{Entity};

pub struct World<E: Entity> {
    pub entities: RwLock<Vec<RwLock<E>>>,
}

impl<E: Entity> World<E> {
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

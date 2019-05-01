use std::sync::{RwLock, RwLockReadGuard};

use crate::entity::{Entity};

pub struct World<E: Entity> {
    entities: RwLock<Vec<RwLock<E>>>,
}

impl<E: Entity> World<E> {
    pub fn push(&self, entity: E) {
        self.entities.write().unwrap().push(RwLock::new(entity));
    }
    pub fn kill(&self, number: usize) {
        self.entities.write().unwrap().remove(number);
    }
    pub fn read_enemy(&self, number: usize) -> RwLockReadGuard {
        self.entities.read().unwrap()
    }
}

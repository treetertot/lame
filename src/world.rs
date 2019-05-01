use std::sync::RwLock;

use crate::entity::{Entity};

pub struct World<E: Entity<T>, T: Sized + Send + Sync> {
    entities: RwLock<Vec<RwLock<E>>>,
    _placeholder: T,
}

impl <E: Entity<T>, T: Sized + Send + Sync> World<E, T> {
    pub fn update_offset(&self, offset_start: usize, offset_amount: usize) {
        for i in (offset_start..self.entities.read().unwrap().len()).step_by(offset_amount) {

        }
    }
}

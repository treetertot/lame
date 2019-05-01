use std::sync::RwLock;

use crate::entity::{Entity};

pub struct World<E: Entity<T>, T: Sized + Send + Sync> {
    entities: RwLock<Vec<RwLock<E>>>,
    _placeholder: T,
}

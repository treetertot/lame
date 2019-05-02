use crate::entity::Entity;

pub struct Dispatcher<E: crate::entity::Entity> {
	guts: std::sync::Arc<crate::world::World<E>>,
}

impl<> Dispatcher

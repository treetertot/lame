use crate::world::World;
/// The core of lame
/// update uses index and world to allow more interaction
pub trait Entity: 'static + Sized + Send + Sync + Clone {
    fn update(entity_num: usize, world: &World<Self>);
}

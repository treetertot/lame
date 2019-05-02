use crate::world::World;
/// The core of lame
/// update uses index and world to allow more interaction
pub trait Entity: Sized + Send + Sync {
    fn update(entity_num: usize, world: &World<Self>);
    fn center(&self) -> (f32, f32);
}

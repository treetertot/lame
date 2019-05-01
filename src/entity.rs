use crate::world::World;
/// The core of lame
/// update uses index and world to allow more interaction
pub trait Entity: Sized + Send + Sync {
    fn update(&self, world: &World<Self>) -> Self;
    fn center(&self) -> (f32, f32);
}

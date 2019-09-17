use crate::world::World;
/// Entity is the type getting updated.
pub trait Entity: Sized + 'static {
    /// Indicates the shared resource the world should use

    type Shared: Send + Sync + 'static;

    /// The entity is constructed from this type
    type Template: Send + 'static;

    /// Controls drawing and communication with the main thread
    type Drawer: Send + 'static;

    /// Constructs the Entity from a template and world (for access to the shared resource)
    fn construct(template: Self::Template, world: &World<Self>) -> Self;

    /// Updates the Entity.
    /// Has a world, so it can create more entities or access the shared resource, and delta time in seconds as f32
    fn update(&mut self, world: &World<Self>, delta: f32) -> Action<Self::Drawer>;
}
/// Since lame entities don't know where another entity is, entities have to handle their own destruction
/// The Action type lets an enemy draw or kill itself, becaus lame expects all living entities to draw
pub enum Action<T> {
    Draw(T),
    Kill,
}

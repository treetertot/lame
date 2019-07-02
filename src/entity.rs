use crate::world::World;

pub trait Entity: Sized + 'static {
    type Shared: Send + Sync + 'static;
    type Template: Send + 'static;
    type Drawer: Send + 'static;

    fn construct(template: &Self::Template, world: &World<Self>) -> Self;
    fn update(&mut self, world: &World<Self>, delta: f32) -> Action<Self>;
}

pub enum Action<E: Entity> {
    Draw(E::Drawer),
    Kill,
}
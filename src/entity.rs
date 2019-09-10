use crate::LameWorld;
pub trait Entity: Sized + 'static {
    type Drawer: Send;
    type Template: Send;
    type Shared: Send + Sync;

    fn construct(template: Self::Template, shared: &Self::Shared) -> Self;
    fn update(&mut self, world: &LameWorld<Self>, delta: f32) -> Action<Self::Drawer>;
}

pub enum Action<T> {
    Draw(T),
    Kill,
}

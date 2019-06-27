use crate::ops::Ops;
pub trait Entity<S: 'static + Send + Sync + Sized>: Send + Sized {
    fn update(&mut self, shared: &S, delta: f32) -> Option<Vec<Ops<Self>>>;
}
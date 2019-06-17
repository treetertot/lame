use crate::ops::Ops;
pub trait Entity<S: 'static + Send + Sync + Sized>: 'static + Send + Sync + Sized {
    fn update(&mut self, delta: f32) -> Option<Vec<Ops<Self>>>;
}
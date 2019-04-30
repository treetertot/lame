/// The core of lame
/// update uses index and world to allow more interaction
pub trait Entity: Clone {
    fn update(&self) -> Self;
    fn center(&self) -> (f32, f32);
}

/// The core of lame
/// update uses index and world to allow more interaction
pub trait Entity<T, H: Hitbox<T>>: Sized {
    fn update(&self, collision_data: T) -> Self;
    fn center(&self) -> (f32, f32);
    fn get_hitbox(&self) -> &H;
}

pub trait Hitbox<T> {
    fn resolve(&self, other: &Self) -> T;
}

pub fn run<T, H: Hitbox<T>, E: Entity<T, H>>(index: usize, world: &Vec<E>) {
    
}

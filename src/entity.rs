/// The core of lame
/// update uses index and world to allow more interaction
pub trait Entity<T: Sized + Send + Sync>: Sized + Send + Sync {
    fn update(&self, collision_data: Option<T>) -> Self;
    fn center(&self) -> (f32, f32);
    fn resolve(&self, other: &Self) -> Option<T>;
}

pub fn run<E: Entity>(index: usize, world: &Vec<E>) {
    let entity = &world[index];
    let mut it = world.iter().enumerate();
    let resolution = loop {
        match it.next() {
            Some((i, other)) => if i == index {
                
            },
            None => break None,
        }
    }
}

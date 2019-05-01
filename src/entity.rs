/// The core of lame
/// update uses index and world to allow more interaction
pub trait Entity<T: Sized + Send + Sync>: Sized + Send + Sync {
    fn update(&self, collision_data: Option<T>) -> Self;
    fn center(&self) -> (f32, f32);
    fn resolve(&self, other: &Self) -> Option<T>;
}

pub fn run<T: Sized + Send + Sync, E: Entity<T>>(index: usize, world: &Vec<E>) -> E {
    let entity = &world[index];
    let mut it = world.iter().enumerate();
    let resolution = loop {
        match it.next() {
            Some((i, other)) => if i == index {
                if i == index {
                    continue;
                }
                match entity.resolve(other) {
                    Some(collision) => {
                        break Some(collision);
                    },
                    None => (),
                }
            },
            None => break None,
        }
    };
    entity.update(resolution)
}

pub mod entity;
pub mod world;

#[cfg(test)]
mod tests {
    #[test]
    fn atomic_check() {
        use crossbeam::atomic::AtomicCell;
        assert_eq!(AtomicCell::<bool>::is_lock_free(), false)
    }

    struct TestEnt {}
    use crate::entity::{Entity, Action};
    use crate::world::World;
    use std::sync::Arc;
    impl Entity for TestEnt {
        type Shared = ();
        type Template = ();
        type Drawer = ();

        fn construct(_template: &Self::Template, _world: &World<Self>) -> Self {
            TestEnt{}
        }
        fn update(&mut self, _world: &World<Self>, _delta: f32) -> Action<Self> {
            println!("updating");
            Action::Draw(())
        }
    }
    #[test]
    fn entity_test() {
        let w: Arc<World<TestEnt>> = World::init((), vec![(), (), (), ()]);
        for _ in 0..4 {
            for (i, _) in w.iter_draws().enumerate() {
                println!("drawing {}", i);
            }
            println!("batch finished");
        }
    }
}
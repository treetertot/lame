//! # Example
//!
//! ```
//!use crate::entity::{Entity, Action};
//!use crate::world::World;
//!use crate::world::LameHandle;
//!
//!struct TestEnt {}
//!impl Entity for TestEnt {
//!    type Shared = ();
//!    type Template = ();
//!    type Drawer = ();
//!
//!    fn construct(_template: &Self::Template, _world: &World<Self>) -> Self {
//!        TestEnt{}
//!    }
//!    fn update(&mut self, _world: &World<Self>, _delta: f32) -> Action<Self> {
//!        println!("updating");
//!        Action::Draw(())
//!    }
//!}
//!let w: LameHandle<TestEnt> = World::init((), vec![(), (), (), ()]);
//!for _ in 0..4 {
//!    for (i, _) in w.iter_draws().enumerate() {
//!        println!("drawing {}", i);
//!    }
//!    println!("batch finished");
//!}
//! ```

pub mod entity;
pub mod world;

#[cfg(test)]
mod tests {
    struct TestEnt {}
    use crate::entity::{Action, Entity};
    use crate::world::World;
    impl Entity for TestEnt {
        type Shared = ();
        type Template = ();
        type Drawer = ();

        fn construct(_template: Self::Template, _world: &Self::Shared) -> Self {
            TestEnt {}
        }
        fn update(&mut self, _world: &World<Self>, _delta: f32) -> Action<Self::Drawer> {
            println!("updating");
            Action::Draw(())
        }
    }
    #[test]
    fn entity_test() {
        use crate::world::LameHandle;
        let w: LameHandle<TestEnt> = World::init((), vec![(), (), (), ()]);
        for _ in 0..4 {
            for (i, _) in w.iter_draws().enumerate() {
                println!("drawing {}", i);
            }
            println!("batch finished");
        }
    }
}

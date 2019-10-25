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
//!    fn construct(_template: Self::Template, _world: &Self::Shared) -> Self {
//!        TestEnt{}
//!    }
//!    fn update(&mut self, _world: &WeakWorld<Self>, _delta: f32) -> Action<Self> {
//!        println!("updating");
//!        Action::Draw(())
//!    }
//!}
//!let w: LameHandle<TestEnt> = World::init((), vec![(), (), (), ()]);
//!for _ in 0..4 {
//!    for (i, _) in w.draws().enumerate() {
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
    use crate::world::{World, WeakWorld};
    impl Entity for TestEnt {
        type Shared = ();
        type Template = ();
        type Drawer = ();

        fn construct(_template: Self::Template, _world: &Self::Shared) -> Self {
            TestEnt {}
        }
        fn update(&mut self, _world: &WeakWorld<Self>, _delta: f32) -> Action<Self::Drawer> {
            println!("updating");
            Action::Draw(4, ())
        }
    }
    #[test]
    fn entity_test() {
        let w: World<TestEnt> = World::new(vec![(), (), (), ()], ());
        for _ in 0..4 {
            for (i, _) in w.draws().into_iter().enumerate() {
                println!("drawing {}", i);
            }
            println!("batch finished");
        }
    }
}

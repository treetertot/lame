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
//!    fn construct(_template: Self::Template, _world: &World<Self>) -> Self {
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

use crossbeam_channel::{self as channel, Receiver, Sender, TryRecvError};
use std::mem;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Instant;

pub mod entity;
use entity::{Action, Entity};

struct Controller<E: Entity> {
    counters: Vec<usize>,
    frame_outputs: Vec<Sender<E::Drawer>>,
    frame_receivers: Vec<Receiver<E::Drawer>>,
    entity_senders: Vec<Sender<E::Template>>,
    entity_receivers: Vec<Receiver<E::Template>>,
    dropped: bool,
}
impl<E: Entity> Controller<E> {
    fn recalibrate(&mut self) {
        let (mut frame_outputs, mut frame_receivers) =
            self.counters.iter().map(|&n| channel::bounded(n)).unzip();
        mem::swap(&mut self.frame_outputs, &mut frame_outputs);
        mem::swap(&mut self.frame_receivers, &mut frame_receivers);
        mem::drop(frame_outputs);
        for (i, recv) in frame_receivers.iter().enumerate() {
            for sprite in recv.iter() {
                self.frame_outputs[i].send(sprite).unwrap();
            }
        }
    }
}

pub struct LameWorld<E: Entity> {
    controller: Arc<RwLock<Controller<E>>>,
    pub shared: Arc<E::Shared>,
}
impl<E: Entity> LameWorld<E> {
    fn secret_clone(&self) -> Self {
        LameWorld {
            controller: self.controller.clone(),
            shared: self.shared.clone(),
        }
    }
    pub fn recalibrate(&self) {
        let newer = self.controller.clone();
        thread::spawn(move || {
            let mut writer = newer.write().unwrap();
            writer.recalibrate();
        });
    }
    pub fn new(shared: E::Shared) -> LameWorld<E> {
        Self::new_cpus(shared, 8)
    }
    pub fn new_cpus(shared: E::Shared, threads: usize) -> LameWorld<E> {
        let (frame_outputs, frame_receivers) = (0..threads).map(|_n| channel::bounded(20)).unzip();
        let (entity_senders, entity_receivers) =
            (0..threads).map(|_n| channel::bounded(20)).unzip();
        let world = LameWorld {
            controller: Arc::new(RwLock::new(Controller {
                counters: (0..threads).map(|_| 0).collect(),
                frame_outputs: frame_outputs,
                frame_receivers: frame_receivers,
                entity_senders: entity_senders,
                entity_receivers: entity_receivers,
                dropped: false,
            })),
            shared: Arc::new(shared),
        };
        for i in 0..threads {
            let world = world.secret_clone();
            thread::spawn(move || {
                update(world, i);
            });
        }
        world.secret_clone()
    }
    pub fn add_entity(&self, entity: E::Template) {
        let controller = self.controller.clone();
        thread::spawn(move || {
            let mut writer = controller.write().unwrap();
            let (mindex, _) = writer
                .counters
                .iter()
                .enumerate()
                .min_by_key(|(_i, &val)| val)
                .unwrap();
            writer.counters[mindex] += 1;
            writer.entity_senders[mindex].send(entity).unwrap();
        });
    }
    pub fn add_entities<C: IntoIterator<Item = E::Template> + Send + 'static>(&self, entities: C) {
        let controller = self.controller.clone();
        thread::spawn(move || {
            let mut writer = controller.write().unwrap();
            for entity in entities {
                let (mindex, _) = writer
                    .counters
                    .iter()
                    .enumerate()
                    .min_by_key(|(_i, &val)| val)
                    .unwrap();
                writer.counters[mindex] += 1;
                writer.entity_senders[mindex].send(entity).unwrap();
            }
            writer.recalibrate();
        });
    }
    pub fn iter_draws(&self) -> Draws<E> {
        Draws {
            right: self
                .controller
                .read()
                .unwrap()
                .counters
                .iter()
                .enumerate()
                .map(|(i, count)| (i, *count))
                .collect(),
            left: Vec::new(),
            control: self.controller.clone(),
        }
    }
}
impl<E: Entity> std::ops::Drop for LameWorld<E> {
    fn drop(&mut self) {
        let new_controller = self.controller.clone();
        if !new_controller.read().unwrap().dropped {
            new_controller.write().unwrap().dropped = true;
        }
    }
}

fn update<E: Entity>(world: LameWorld<E>, index: usize) {
    let mut entities: Vec<E> = Vec::new();
    let mut time = Instant::now();
    loop {
        let reader = world.controller.read().unwrap();
        for temp in (*reader).entity_receivers[index].try_iter() {
            entities.push(E::construct(temp, &world.shared));
        }
        let mut to_remove = Vec::new();
        let delta = time.elapsed().as_micros() as f32 / 1000000.0;
        time = Instant::now();
        for (i, entity) in entities.iter_mut().enumerate() {
            match entity.update(&world, delta) {
                Action::Draw(sprite) => reader.frame_outputs[index].send(sprite).unwrap(),
                Action::Kill => to_remove.push(i),
            }
        }
        if to_remove.len() != 0 {
            let mut shifted = 0;
            for n in to_remove {
                entities.remove(n - shifted);
                shifted += 1;
            }
        }
        if world.controller.read().unwrap().dropped {
            break;
        }
    }
}

pub struct Draws<E: Entity> {
    left: Vec<(usize, usize)>,
    control: Arc<RwLock<Controller<E>>>,
    right: Vec<(usize, usize)>,
}
impl<E: Entity> Iterator for Draws<E> {
    type Item = E::Drawer;

    fn next(&mut self) -> Option<E::Drawer> {
        //note: yes, I can do this better and realize that left is just a waste of memory, but I'm lazy and method chaining is fun
        self.right
            .extend(self.left.iter().map(|&pair| pair).filter(|(_, n)| *n == 0));
        std::mem::swap(&mut self.left, &mut self.right);
        self.right.clear();

        if self.left.len() > 0 {
            loop {
                let reader = self.control.read().unwrap();
                for (index, count) in &mut self.left {
                    match reader.frame_receivers[*index].try_recv() {
                        Ok(res) => {
                            *count -= 1;
                            return Some(res);
                        }
                        Err(TryRecvError::Empty) => (),
                        _ => panic!("I think a thread crashed or something"),
                    }
                }
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let sum = self.right.iter().map(|(_, n)| n).sum();
        (sum, Some(sum))
    }
}

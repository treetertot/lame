use std::sync::{RwLock, RwLockReadGuard, Arc};
use std::thread;
use std::time::Instant;

use crate::entity::{Entity};

#[derive(Clone)]
pub struct World<E: Entity> {
    pub entities: Arc<RwLock<Vec<RwLock<E>>>>,
}

impl<E: Entity> World<E> {
    pub fn new() -> World<E> {
        let world = World{entities: Arc::new(RwLock::new(Vec::new()))};
        world
    }
    pub fn start(&self) {
        for i in 0..2 {
            let world = self.clone();
            thread::spawn(move || {
                world.run_offset(i, 3)
            });
        }
    }
    pub fn push(&self, entity: E) {
        self.entities.write().unwrap().push(RwLock::new(entity));
    }
    pub fn kill(&self, number: usize) {
        self.entities.write().unwrap().remove(number);
    }
    pub fn read_list(&self) -> RwLockReadGuard<Vec<RwLock<E>>> {
        self.entities.read().unwrap()
    }
    pub fn run_offset(&self, start: usize, amount: usize) {
		let list = self.read_list();
        let mut deltas: Vec<Instant> = Vec::new();
        loop {
            let mut last_step = 0;
		    for (step, i) in (start..list.len()).step_by(amount).enumerate() {
                if step == deltas.len() {
                    deltas.push(Instant::now())
                }
                let delta = deltas[step].elapsed().as_millis() as f32 * 1000.0;
                last_step = step;
			    E::update(i, self, delta);
		    }
            if deltas.len() > last_step + 1 {
                for _ in (last_step + 1)..deltas.len() {
                    deltas.pop();
                }
            }
        }
	}
}

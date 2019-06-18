use crate::entity::Entity;
use crate::ops::Ops::*;
use rayon::prelude::*;
use std::sync::Mutex;
use std::mem;
use std::time::Instant;

pub struct World<S: 'static + Send + Sync + Sized, E: Entity<S>> {
    pub shared: S,
    pub entities: Vec<E>,
    time: Instant,
}
impl<S: 'static + Send + Sync + Sized, E: Entity<S>> World<S, E> {
    pub fn new(s: S) -> Self {
        World { shared: s, entities: Vec::new(), time: Instant::now() }
    }
    pub fn run(&mut self) {
        let shared = &self.shared;
        let list_ops = Mutex::new(Vec::new());
        let delta = self.time.elapsed().as_micros() as f32 * 1000000.0;
        self.time = Instant::now();
        (&mut self.entities).par_iter_mut().enumerate().for_each(|(i, ent)| match ent.update(shared, delta) {
            None => (),
            Some(ops) => {
                let mut new_ops = Vec::new();
                for op in ops {
                    match op {
                        Kill => new_ops.push(SecretOps::Kill(i)),
                        Spawn(to_spawn) => new_ops.push(SecretOps::Spawn(to_spawn)),
                    }
                }
                list_ops.lock().unwrap().append(&mut new_ops);
            },
        });
        let derefed = &mut *list_ops.lock().unwrap();
        if derefed.len() == 0 {
            return;
        }
        let mut switcher = Vec::new();
        mem::swap(derefed, &mut switcher);
        let mut deletions = Vec::new();
        for op in switcher {
            match op {
                SecretOps::Spawn(to_spawn) => self.entities.push(to_spawn),
                SecretOps::Kill(num) => {
                    let actual_num = num - nums_before(&deletions, num);
                    deletions.push(num);
                    self.entities.remove(actual_num);
                },
            }
        }
    }
}

fn nums_before(v: &Vec<usize>, num: usize) -> usize {
    let mut bef = 0;
    for &numb in v.iter() {
        if numb < num {
            bef += 1;
        }
    }
    bef
}

enum SecretOps<T> {
    Kill(usize),
    Spawn(T),
}
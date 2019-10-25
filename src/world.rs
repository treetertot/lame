use crate::entity::Entity;
use crossbeam_channel::{bounded, Receiver, Sender, unbounded, TryRecvError};
use std::iter;
use std::mem;
use std::sync::{Arc, atomic::{AtomicBool, Ordering::Relaxed}};
use std::thread;
use std::time::Instant;
use parking_lot::{Mutex, RwLock};

mod capsule;
pub use capsule::Capsule;
mod framebuilder;
use framebuilder::*;
use lazy_static::lazy_static;

lazy_static!{
    static ref THREADS: usize = num_cpus::get();
}

struct SecretWorld<E: Entity> {
    current_frame: RwLock<Frame<E>>,
    counts: Mutex<Vec<usize>>,
    entity_senders: Vec<Sender<E::Template>>,
    frame_out: Sender<Frame<E>>,
    shutdown: AtomicBool,
    shared: E::Shared,
}
impl<E: Entity> SecretWorld<E> {
    fn add_entity(&self, temp: E::Template) {
        let mut counts = self.counts.lock();
        let (mindex, minval) = counts.iter().enumerate().skip(1).fold((0, counts[0]), |prev, new| {
            if prev.1 > *new.1 {
                (new.0, *new.1)
            } else {
                prev
            }
        });
        counts[mindex] = minval + 1;
        self.entity_senders[mindex].send(temp).unwrap();
    }
    fn add_entities(&self, temps: Vec<E::Template>) {
        let mut counts = self.counts.lock();
        for temp in temps {
            let (mindex, minval) = counts.iter().enumerate().skip(1).fold((0, counts[0]), |prev, new| {
                if prev.1 > *new.1 {
                    (new.0, *new.1)
                } else {
                    prev
                }
            });
            counts[mindex] = minval + 1;
            self.entity_senders[mindex].send(temp).unwrap();
        }
    }
}

pub struct WeakWorld<E: Entity> {
    secret: Arc<SecretWorld<E>>,
}
impl<E:Entity> WeakWorld<E> {
    pub fn add_entity(&self, template: E::Template) {
        self.secret.add_entity(template)
    }
    pub fn add_entities(&self, templates: Vec<E::Template>) {
        self.secret.add_entities(templates)
    }
    pub fn shared(&self) -> &E::Shared {
        &self.secret.shared
    }
}
fn update<E: Entity>(secret: Arc<SecretWorld<E>>, entities_in: Receiver<E::Template>) {
    thread::spawn(move || {
        let me = WeakWorld {
            secret
        };
        let mut entities = Vec::new();
        let mut timer = Instant::now();
        while !me.secret.shutdown.load(Relaxed) {
            loop{
                match entities_in.try_recv() {
                    Ok(e) => entities.push(E::construct(e, &me.secret.shared)),
                    Err(err) => match err {
                        TryRecvError::Empty => break,
                        TryRecvError::Disconnected => return,
                    },
                }
            }
            let time = (timer.elapsed().as_micros() % 1000000) as f32 * 1e6;
            timer = Instant::now();
            if me.secret.current_frame.read().run(&mut entities, &me, time) {
                let mut new_frame = Frame::new(*THREADS);
                let mut lock = me.secret.current_frame.write();
                mem::swap(&mut *lock, &mut new_frame);
                match me.secret.frame_out.send(new_frame) {
                    Ok(_) => (),
                    Err(_) => {
                        me.secret.shutdown.store(true, Relaxed);
                        return
                    },
                }
            }
        }
    });
}

pub struct World<E: Entity> {
    secret: Arc<SecretWorld<E>>,
    frames: Receiver<Frame<E>>,
}
impl<E: Entity> World<E> {
    pub fn new(start: Vec<E::Template>, shared: E::Shared) -> Self {
        let (frame_out, frecv) = bounded(0);
        let (entity_senders, entrecvs): (Vec<_>, Vec<_>) = (0..*THREADS).map(|_| unbounded()).unzip();
        let secret = Arc::new(SecretWorld {
            counts: Mutex::new((0..*THREADS-1).map(|_| start.len() / *THREADS).chain(iter::once(start.len() - (*THREADS * (start.len() / *THREADS)))).collect()),
            current_frame: RwLock::new(Frame::new(*THREADS)),
            shutdown: AtomicBool::new(false),
            entity_senders,
            frame_out,
            shared,
        });
        for entrecv in entrecvs {
            update(secret.clone(), entrecv);
        }
        World {
            secret,
            frames: frecv,
        }
    }
    pub fn add_entity(&self, template: E::Template) {
        self.secret.add_entity(template)
    }
    pub fn add_entities(&self, templates: Vec<E::Template>) {
        self.secret.add_entities(templates)
    }
    pub fn shared(&self) -> &E::Shared {
        &self.secret.shared
    }
    pub fn draws(&self) -> Vec<Capsule<E::Drawer>> {
        self.frames.recv().unwrap().collapse()
    }
}

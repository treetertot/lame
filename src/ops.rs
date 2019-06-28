#[derive(Clone)]
pub enum Ops<E> {
    Kill,
    Spawn(E),
}
use std::cmp::Ordering;

/// Holds data to be ordered by layer
#[derive(Debug)]
pub struct Capsule<T> {
    pub layer: u8,
    pub data: T,
}

impl<T> PartialEq for Capsule<T> {
    fn eq(&self, other: &Self) -> bool {
        self.layer == other.layer
    }
}
impl<T> PartialOrd for Capsule<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.layer.cmp(&other.layer))
    }
}
impl<T> Eq for Capsule<T> {}
impl<T> Ord for Capsule<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.layer.cmp(&other.layer)
    }
}

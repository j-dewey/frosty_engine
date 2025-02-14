#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct AllocName {
    uoid: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct AllocId {
    uid: u64,
}

impl AllocId {
    pub fn new(val: u64) -> Self {
        Self { uid: val }
    }
}

use std::hash::Hash;

pub trait HashedId: Hash + Eq {}
impl<T> HashedId for T where T: Hash + Eq {}

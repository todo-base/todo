use std::sync::atomic::{AtomicU64, Ordering};

pub trait IdGenerator {
    type Id;

    fn next(&self) -> Self::Id;
}

#[derive(Debug)]
pub struct IntIdGenerator {
    next_id: AtomicU64,
}

impl IntIdGenerator {
    pub fn new(start_id: u64) -> Self {
        Self {
            next_id: AtomicU64::new(start_id),
        }
    }
}

macro_rules! id_generator_impl {
    ($gen:ty) => {
        impl IdGenerator for $gen {
            type Id = u64;

            fn next(&self) -> Self::Id {
                self.next_id.fetch_add(1, Ordering::SeqCst)
            }
        }
    };
}

id_generator_impl!(IntIdGenerator);
id_generator_impl!(&IntIdGenerator);

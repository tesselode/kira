use std::{
	marker::PhantomData,
	sync::atomic::{AtomicUsize, Ordering},
};

static NEXT_INDEX: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Id<T> {
	index: usize,
	phantom_data: PhantomData<T>,
}

impl<T> Id<T> {
	pub fn new() -> Self {
		let index = NEXT_INDEX.fetch_add(1, Ordering::Relaxed);
		Self {
			index,
			phantom_data: PhantomData,
		}
	}
}

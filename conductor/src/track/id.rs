use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_SUB_TRACK_INDEX: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct SubTrackId {
	index: usize,
}

impl SubTrackId {
	pub(crate) fn new() -> Self {
		let index = NEXT_SUB_TRACK_INDEX.fetch_add(1, Ordering::Relaxed);
		Self { index }
	}
}

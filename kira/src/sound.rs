pub mod data;
pub mod handle;
pub mod instance;

use std::sync::{atomic::AtomicBool, Arc};

use atomic_arena::Index;

use self::data::SoundData;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SoundId(pub(crate) Index);

pub(crate) struct SoundShared {
	pub removed: AtomicBool,
}

impl SoundShared {
	pub(crate) fn new() -> Self {
		Self {
			removed: AtomicBool::new(false),
		}
	}
}

pub(crate) struct Sound {
	pub data: Arc<dyn SoundData>,
	pub shared: Arc<SoundShared>,
}

pub mod data;

use atomic_arena::Index;

use self::data::SoundData;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SoundId(pub(crate) Index);

pub(crate) struct Sound {
	data: Box<dyn SoundData>,
}

impl Sound {
	pub fn new(data: Box<dyn SoundData>) -> Self {
		Self { data }
	}
}

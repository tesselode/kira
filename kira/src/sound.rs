use self::data::SoundData;

pub mod data;

pub struct Sound {
	data: Box<dyn SoundData>,
}

impl Sound {
	pub fn new(data: Box<dyn SoundData>) -> Self {
		Self { data }
	}
}

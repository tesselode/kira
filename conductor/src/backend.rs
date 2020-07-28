use crate::{
	event::Command,
	sound_bank::{SoundBank, SoundId},
	stereo_sample::StereoSample,
};
use ringbuf::Consumer;

pub struct Backend {
	sample_rate: u32,
	sound_bank: SoundBank,
	command_consumer: Consumer<Command>,
}

impl Backend {
	pub fn new(
		sample_rate: u32,
		sound_bank: SoundBank,
		command_consumer: Consumer<Command>,
	) -> Self {
		Self {
			sample_rate,
			sound_bank,
			command_consumer,
		}
	}

	fn play_sound(&mut self, sound_id: SoundId) {
		let index = sound_id.index;
		let sound = &mut self.sound_bank.sounds[index];
		sound.play();
	}

	pub fn process(&mut self) -> StereoSample {
		while let Some(command) = self.command_consumer.pop() {
			match command {
				Command::PlaySound(id) => {
					self.play_sound(id);
				}
			}
		}
		let mut out = StereoSample::from_mono(0.0);
		for sound in &mut self.sound_bank.sounds {
			out += sound.process();
		}
		out
	}
}

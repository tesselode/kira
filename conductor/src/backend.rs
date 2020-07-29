use crate::{
	event::Command,
	manager::PlaySoundSettings,
	sound_bank::{SoundBank, SoundId},
	stereo_sample::StereoSample,
};
use ringbuf::Consumer;

pub struct Backend {
	dt: f32,
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
			dt: 1.0 / sample_rate as f32,
			sound_bank,
			command_consumer,
		}
	}

	fn play_sound(&mut self, sound_id: SoundId, settings: PlaySoundSettings) {
		let index = sound_id.index;
		let sound = &mut self.sound_bank.sounds[index];
		sound.play(settings);
	}

	pub fn process(&mut self) -> StereoSample {
		while let Some(command) = self.command_consumer.pop() {
			match command {
				Command::PlaySound(id, settings) => {
					self.play_sound(id, settings);
				}
			}
		}
		let mut out = StereoSample::from_mono(0.0);
		for sound in &mut self.sound_bank.sounds {
			out += sound.process(self.dt);
		}
		out
	}
}

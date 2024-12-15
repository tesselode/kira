mod builder;
mod handle;

pub use builder::*;
pub use handle::*;

use crate::{
	command::{CommandReader, ValueChangeCommand},
	effect::Effect,
	info::Info,
	manager::backend::resources::ResourceStorage,
	sound::Sound,
	tween::Parameter,
	Decibels, Frame,
};

pub(crate) struct MainTrack {
	volume: Parameter<Decibels>,
	set_volume_command_reader: CommandReader<ValueChangeCommand<Decibels>>,
	sounds: ResourceStorage<Box<dyn Sound>>,
	effects: Vec<Box<dyn Effect>>,
	temp_buffer: Vec<Frame>,
}

impl MainTrack {
	pub fn init_effects(&mut self, sample_rate: u32) {
		for effect in &mut self.effects {
			effect.init(sample_rate);
		}
	}

	pub fn on_change_sample_rate(&mut self, sample_rate: u32) {
		for effect in &mut self.effects {
			effect.on_change_sample_rate(sample_rate);
		}
	}

	pub fn on_start_processing(&mut self) {
		self.volume
			.read_command(&mut self.set_volume_command_reader);
		self.sounds.remove_and_add(|sound| sound.finished());
		for (_, sound) in &mut self.sounds {
			sound.on_start_processing();
		}
		for effect in &mut self.effects {
			effect.on_start_processing();
		}
	}

	pub fn process(&mut self, out: &mut [Frame], dt: f64, info: &Info) {
		self.volume.update(dt * out.len() as f64, info);
		for (_, sound) in &mut self.sounds {
			sound.process(&mut self.temp_buffer[..out.len()], dt, info);
			for (summed_out, sound_out) in out.iter_mut().zip(self.temp_buffer.iter().copied()) {
				*summed_out += sound_out;
			}
			self.temp_buffer.fill(Frame::ZERO);
		}
		for effect in &mut self.effects {
			effect.process(out, dt, info);
		}
		for frame in out {
			*frame *= self.volume.value().as_amplitude();
		}
	}
}

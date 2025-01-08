mod builder;
mod handle;

pub use builder::*;
pub use handle::*;

use crate::{
	backend::resources::ResourceStorage,
	command::{CommandReader, ValueChangeCommand},
	effect::Effect,
	info::Info,
	sound::Sound,
	Decibels, Frame, Parameter,
};

pub(crate) struct MainTrack {
	volume: Parameter<Decibels>,
	set_volume_command_reader: CommandReader<ValueChangeCommand<Decibels>>,
	sounds: ResourceStorage<Box<dyn Sound>>,
	effects: Vec<Box<dyn Effect>>,
	temp_buffer: Vec<Frame>,
	internal_buffer_size: usize,
}

impl MainTrack {
	pub fn init_effects(&mut self, sample_rate: u32) {
		for effect in &mut self.effects {
			effect.init(sample_rate, self.internal_buffer_size);
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
		let num_frames = out.len();
		for (i, frame) in out.iter_mut().enumerate() {
			let time_in_chunk = (i + 1) as f64 / num_frames as f64;
			let volume = self.volume.interpolated_value(time_in_chunk).as_amplitude();
			*frame *= volume;
		}
	}
}

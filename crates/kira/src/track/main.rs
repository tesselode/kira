mod builder;
mod handle;

pub use builder::*;
pub use handle::*;

use crate::{
	command::{CommandReader, ValueChangeCommand},
	effect::Effect,
	info::Info,
	resources::{clocks::Clocks, modulators::Modulators, ResourceStorage},
	sound::Sound,
	Decibels, Frame, Parameter, INTERNAL_BUFFER_SIZE,
};

pub(crate) struct MainTrack {
	volume: Parameter<Decibels>,
	set_volume_command_reader: CommandReader<ValueChangeCommand<Decibels>>,
	sounds: ResourceStorage<Box<dyn Sound>>,
	effects: Vec<Box<dyn Effect>>,
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

	pub(crate) fn process(
		&mut self,
		out: &mut [Frame],
		dt: f64,
		clocks: &Clocks,
		modulators: &Modulators,
	) {
		let info = Info::new(&clocks.0.resources, &modulators.0.resources);

		// process sounds
		let mut per_sound_buffer = [Frame::ZERO; INTERNAL_BUFFER_SIZE];
		for (_, sound) in &mut self.sounds {
			sound.process(&mut per_sound_buffer[..out.len()], dt, &info);
			for (summed_out, sound_out) in out.iter_mut().zip(per_sound_buffer.iter_mut()) {
				*summed_out += *sound_out;
			}
			per_sound_buffer = [Frame::ZERO; INTERNAL_BUFFER_SIZE];
		}

		// process effects
		for effect in &mut self.effects {
			effect.process(out, dt, &info);
		}

		// apply post-effects volume
		let mut volume_buffer = [Decibels::IDENTITY; INTERNAL_BUFFER_SIZE];
		self.volume
			.update_chunk(&mut volume_buffer[..out.len()], dt, &info);
		for (frame, volume) in out.iter_mut().zip(volume_buffer.iter().copied()) {
			*frame *= volume.as_amplitude();
		}
	}
}

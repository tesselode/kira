mod builder;
mod handle;

pub use builder::*;
pub use handle::*;

use crate::{
	command::{CommandReader, ValueChangeCommand},
	info::Info,
	resources::{clocks::Clocks, modulators::Modulators, ResourceStorage},
	sound::Sound,
	Decibels, Frame, Parameter, INTERNAL_BUFFER_SIZE,
};

pub(crate) struct MainTrack {
	volume: Parameter<Decibels>,
	set_volume_command_reader: CommandReader<ValueChangeCommand<Decibels>>,
	sounds: ResourceStorage<Box<dyn Sound>>,
}

impl MainTrack {
	pub fn on_start_processing(&mut self) {
		self.volume
			.read_command(&mut self.set_volume_command_reader);
		self.sounds.remove_and_add(|sound| sound.finished());
		for (_, sound) in &mut self.sounds {
			sound.on_start_processing();
		}
		/* for effect in &mut self.effects {
			effect.on_start_processing();
		} */
	}

	pub(crate) fn process(
		&mut self,
		out: &mut [Frame],
		dt: f64,
		clocks: &Clocks,
		modulators: &Modulators,
	) {
		let mut per_sound_buffer = [Frame::ZERO; INTERNAL_BUFFER_SIZE];
		let info = Info::new(&clocks.0.resources, &modulators.0.resources);
		for (_, sound) in &mut self.sounds {
			sound.process(&mut per_sound_buffer[..out.len()], dt, &info);
			for (summed_out, sound_out) in out.iter_mut().zip(per_sound_buffer.iter_mut()) {
				*summed_out += *sound_out;
			}
			per_sound_buffer = [Frame::ZERO; INTERNAL_BUFFER_SIZE];
		}
		let mut volume_buffer = [Decibels::IDENTITY; INTERNAL_BUFFER_SIZE];
		self.volume.update_chunk(&mut volume_buffer, dt, &info);
		for (frame, volume) in out.iter_mut().zip(volume_buffer.iter().copied()) {
			*frame *= volume.as_amplitude();
		}
	}
}

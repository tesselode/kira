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
	Dbfs, Frame,
};

pub(crate) struct MainTrack {
	volume: Parameter<Dbfs>,
	set_volume_command_reader: CommandReader<ValueChangeCommand<Dbfs>>,
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

	pub fn process(&mut self, input: Frame, dt: f64, info: &Info) -> Frame {
		self.volume.update(dt, info);
		let mut output = input;
		for (_, sound) in &mut self.sounds {
			output += sound.process(dt, info);
		}
		for effect in &mut self.effects {
			output = effect.process(output, dt, info);
		}
		output *= self.volume.value().as_amplitude();
		output
	}
}

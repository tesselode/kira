//! Adjusts the volume of audio.

mod builder;
mod handle;

pub use builder::*;
pub use handle::*;

use ringbuf::Consumer;

use crate::{
	clock::ClockTime,
	dsp::Frame,
	tween::{Tween, Tweener},
	Volume,
};

use super::Effect;

enum Command {
	SetVolume(Volume, Tween),
}

struct VolumeControl {
	command_consumer: Consumer<Command>,
	volume: Tweener<Volume>,
}

impl VolumeControl {
	fn new(builder: VolumeControlBuilder, command_consumer: Consumer<Command>) -> Self {
		Self {
			command_consumer,
			volume: Tweener::new(builder.0),
		}
	}
}

impl Effect for VolumeControl {
	fn on_start_processing(&mut self) {
		while let Some(command) = self.command_consumer.pop() {
			match command {
				Command::SetVolume(volume, tween) => self.volume.set(volume, tween),
			}
		}
	}

	fn process(&mut self, input: Frame, dt: f64) -> Frame {
		self.volume.update(dt);
		input * self.volume.value().as_amplitude() as f32
	}

	fn on_clock_tick(&mut self, time: ClockTime) {
		self.volume.on_clock_tick(time);
	}
}

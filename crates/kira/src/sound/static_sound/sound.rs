#[cfg(test)]
mod test;

use std::{
	convert::TryInto,
	sync::{
		atomic::{AtomicU64, Ordering},
		Arc,
	},
};

use ringbuf::HeapConsumer;

use crate::{
	clock::clock_info::ClockInfoProvider,
	dsp::Frame,
	modulator::value_provider::ModulatorValueProvider,
	sound::{transport::Transport, PlaybackRate, Sound},
	tween::Parameter,
};

use super::{data::StaticSoundData, Command};

pub(super) struct StaticSound {
	command_consumer: HeapConsumer<Command>,
	data: StaticSoundData,
	transport: Transport,
	playback_rate: Parameter<PlaybackRate>,
	shared: Arc<Shared>,
}

impl StaticSound {
	pub fn new(data: StaticSoundData, command_consumer: HeapConsumer<Command>) -> Self {
		let settings = data.settings;
		let transport = Transport::new(
			data.frames.len() as i64,
			data.settings.loop_region,
			data.settings.reverse,
			data.sample_rate,
		);
		let starting_frame_index = transport.position;
		let position = starting_frame_index as f64 / data.sample_rate as f64;
		Self {
			command_consumer,
			data,
			transport,
			playback_rate: Parameter::new(settings.playback_rate, PlaybackRate::Factor(1.0)),
			shared: Arc::new(Shared {
				position: AtomicU64::new(position.to_bits()),
			}),
		}
	}

	pub(super) fn shared(&self) -> Arc<Shared> {
		self.shared.clone()
	}

	fn is_playing_backwards(&self) -> bool {
		let mut is_playing_backwards = self.playback_rate.value().as_factor().is_sign_negative();
		if self.data.settings.reverse {
			is_playing_backwards = !is_playing_backwards
		}
		is_playing_backwards
	}

	fn update_position(&mut self) {
		if self.is_playing_backwards() {
			self.transport.decrement_position();
		} else {
			self.transport.increment_position();
		}
	}

	fn seek_to_index(&mut self, index: i64) {
		self.transport.seek_to(index);
	}

	fn seek_by(&mut self, amount: f64) {
		let current_position = self.transport.position as f64 / self.data.sample_rate as f64;
		let position = current_position + amount;
		let index = (position * self.data.sample_rate as f64) as i64;
		self.seek_to_index(index);
	}

	fn seek_to(&mut self, position: f64) {
		let index = (position * self.data.sample_rate as f64) as i64;
		self.seek_to_index(index);
	}
}

impl Sound for StaticSound {
	fn sample_rate(&self) -> f64 {
		self.data.sample_rate as f64 * self.playback_rate.value().as_factor().abs()
	}

	fn on_start_processing(&mut self) {
		self.shared.position.store(
			(self.transport.position as f64 / self.data.sample_rate as f64).to_bits(),
			Ordering::SeqCst,
		);
		while let Some(command) = self.command_consumer.pop() {
			match command {
				Command::SetPlaybackRate(playback_rate, tween) => {
					self.playback_rate.set(playback_rate, tween)
				}
				Command::SetLoopRegion(loop_region) => self.transport.set_loop_region(
					loop_region,
					self.data.sample_rate,
					self.data.frames.len(),
				),
				Command::SeekBy(amount) => {
					self.seek_by(amount);
				}
				Command::SeekTo(position) => {
					self.seek_to(position);
				}
			}
		}
	}

	fn process(
		&mut self,
		clock_info_provider: &ClockInfoProvider,
		modulator_value_provider: &ModulatorValueProvider,
	) -> Frame {
		if !self.transport.playing {
			return Frame::ZERO;
		}

		// update parameters
		self.playback_rate.update(
			1.0 / self.sample_rate(),
			clock_info_provider,
			modulator_value_provider,
		);

		// play back audio
		let num_frames: i64 = self
			.data
			.frames
			.len()
			.try_into()
			.expect("sound is too long, cannot convert usize to i64");
		let out = if self.transport.position < 0 || self.transport.position >= num_frames {
			Frame::ZERO
		} else {
			let frame_index: usize = self
				.transport
				.position
				.try_into()
				.expect("cannot convert i64 into usize");
			self.data.frames[frame_index]
		};
		self.update_position();
		out
	}

	fn finished(&self) -> bool {
		!self.transport.playing
	}
}

pub(super) struct Shared {
	position: AtomicU64,
}

impl Shared {
	pub fn position(&self) -> f64 {
		f64::from_bits(self.position.load(Ordering::SeqCst))
	}
}

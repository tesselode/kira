#[cfg(test)]
mod test;

use std::sync::{
	atomic::{AtomicU64, Ordering},
	Arc,
};

use crate::{
	clock::clock_info::ClockInfoProvider,
	command::ValueChangeCommand,
	dsp::Frame,
	modulator::value_provider::ModulatorValueProvider,
	sound::{transport::Transport, PlaybackRate, Sound},
	tween::Parameter,
};

use super::{
	data::StaticSoundData, frame, num_frames, CommandReaders, SeekCommand, SetLoopRegionCommand,
};

pub(super) struct StaticSound {
	sample_rate: u32,
	frames: Arc<[Frame]>,
	slice: Option<(usize, usize)>,
	reverse: bool,
	transport: Transport,
	playback_rate: Parameter<PlaybackRate>,
	shared: Arc<Shared>,
	command_readers: CommandReaders,
}

impl StaticSound {
	pub fn new(data: StaticSoundData, command_readers: CommandReaders) -> Self {
		let settings = data.settings;
		let transport = Transport::new(
			data.num_frames(),
			data.settings.loop_region,
			data.settings.reverse,
			data.sample_rate,
			data.settings.start_position.into_samples(data.sample_rate),
		);
		let starting_frame_index = transport.position;
		let position = starting_frame_index as f64 / data.sample_rate as f64;
		Self {
			sample_rate: data.sample_rate,
			frames: data.frames,
			slice: data.slice,
			reverse: data.settings.reverse,
			transport,
			playback_rate: Parameter::new(settings.playback_rate, PlaybackRate::Factor(1.0)),
			shared: Arc::new(Shared {
				position: AtomicU64::new(position.to_bits()),
			}),
			command_readers,
		}
	}

	pub(super) fn shared(&self) -> Arc<Shared> {
		self.shared.clone()
	}

	fn is_playing_backwards(&self) -> bool {
		let mut is_playing_backwards = self.playback_rate.value().as_factor().is_sign_negative();
		if self.reverse {
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

	fn seek_to_index(&mut self, index: usize) {
		self.transport.seek_to(index);
	}

	fn seek_by(&mut self, amount: f64) {
		let current_position = self.transport.position as f64 / self.sample_rate as f64;
		let position = current_position + amount;
		let index = (position * self.sample_rate as f64) as usize;
		self.seek_to_index(index);
	}

	fn seek_to(&mut self, position: f64) {
		let index = (position * self.sample_rate as f64) as usize;
		self.seek_to_index(index);
	}
}

impl Sound for StaticSound {
	fn sample_rate(&self) -> f64 {
		self.sample_rate as f64 * self.playback_rate.value().as_factor().abs()
	}

	fn on_start_processing(&mut self) {
		self.shared.position.store(
			(self.transport.position as f64 / self.sample_rate as f64).to_bits(),
			Ordering::SeqCst,
		);
		if let Some(ValueChangeCommand { target, tween }) =
			self.command_readers.playback_rate_change.read().copied()
		{
			self.playback_rate.set(target, tween);
		}
		if let Some(SetLoopRegionCommand(loop_region)) =
			self.command_readers.set_loop_region.read().copied()
		{
			self.transport.set_loop_region(
				loop_region,
				self.sample_rate,
				num_frames(&self.frames, self.slice),
			);
		}
		if let Some(seek_command) = self.command_readers.seek.read().copied() {
			match seek_command {
				SeekCommand::By(amount) => self.seek_by(amount),
				SeekCommand::To(position) => self.seek_to(position),
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
		let out = frame(&self.frames, self.slice, self.transport.position);
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

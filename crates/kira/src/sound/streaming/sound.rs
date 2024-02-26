pub(crate) mod decode_scheduler;

#[cfg(test)]
mod test;

use std::sync::{
	atomic::{AtomicBool, AtomicU64, Ordering},
	Arc,
};

use crate::{
	clock::clock_info::ClockInfoProvider,
	command::ValueChangeCommand,
	dsp::Frame,
	modulator::value_provider::ModulatorValueProvider,
	sound::{PlaybackRate, Sound},
	tween::Parameter,
};
use ringbuf::HeapConsumer;

use super::{SoundCommandReaders, StreamingSoundSettings};

use self::decode_scheduler::DecodeScheduler;

pub(crate) struct Shared {
	position: AtomicU64,
	reached_end: AtomicBool,
	stopped: AtomicBool,
}

impl Shared {
	pub fn new() -> Self {
		Self {
			position: AtomicU64::new(0.0f64.to_bits()),
			reached_end: AtomicBool::new(false),
			stopped: AtomicBool::new(false),
		}
	}

	pub fn position(&self) -> f64 {
		f64::from_bits(self.position.load(Ordering::SeqCst))
	}

	pub fn reached_end(&self) -> bool {
		self.reached_end.load(Ordering::SeqCst)
	}
}

pub(crate) struct StreamingSound {
	sample_rate: u32,
	frame_consumer: HeapConsumer<TimestampedFrame>,
	current_frame: usize,
	playback_rate: Parameter<PlaybackRate>,
	shared: Arc<Shared>,
	command_readers: SoundCommandReaders,
}

impl StreamingSound {
	pub fn new<Error: Send + 'static>(
		sample_rate: u32,
		settings: StreamingSoundSettings,
		shared: Arc<Shared>,
		frame_consumer: HeapConsumer<TimestampedFrame>,
		scheduler: &DecodeScheduler<Error>,
		command_readers: SoundCommandReaders,
	) -> Self {
		let current_frame = scheduler.current_frame();
		let start_position = current_frame as f64 / sample_rate as f64;
		shared
			.position
			.store(start_position.to_bits(), Ordering::SeqCst);
		Self {
			sample_rate,
			frame_consumer,
			current_frame,
			playback_rate: Parameter::new(settings.playback_rate, PlaybackRate::Factor(1.0)),
			shared,
			command_readers,
		}
	}

	fn update_current_frame(&mut self) {
		let current_frame = &mut self.current_frame;
		let (a, b) = self.frame_consumer.as_slices();
		let mut iter = a.iter().chain(b.iter());
		if let Some(TimestampedFrame { index, .. }) = iter.nth(1) {
			*current_frame = *index;
		}
	}

	fn position(&self) -> f64 {
		self.current_frame as f64 / self.sample_rate as f64
	}
}

impl Sound for StreamingSound {
	fn sample_rate(&self) -> f64 {
		self.sample_rate as f64 * self.playback_rate.value().as_factor().abs()
	}

	fn on_start_processing(&mut self) {
		self.update_current_frame();
		self.shared
			.position
			.store(self.position().to_bits(), Ordering::SeqCst);
		if let Some(ValueChangeCommand { target, tween }) =
			self.command_readers.playback_rate_change.read().copied()
		{
			self.playback_rate.set(target, tween);
		}
	}

	fn process(
		&mut self,
		clock_info_provider: &ClockInfoProvider,
		modulator_value_provider: &ModulatorValueProvider,
	) -> Frame {
		// update parameters
		self.playback_rate.update(
			1.0 / self.sample_rate(),
			clock_info_provider,
			modulator_value_provider,
		);

		self.frame_consumer
			.pop()
			.map(|TimestampedFrame { frame, .. }| frame)
			.unwrap_or(Frame::ZERO)
	}

	fn finished(&self) -> bool {
		self.shared.reached_end() && self.frame_consumer.is_empty()
	}

	fn on_stop(&mut self) {
		self.shared.stopped.store(true, Ordering::SeqCst);
	}
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct TimestampedFrame {
	frame: Frame,
	index: usize,
}

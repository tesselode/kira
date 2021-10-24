mod scheduler;

use std::{ops::Range, sync::Arc, time::Duration};

use ringbuf::Producer;
use triple_buffer::{Input, Output};

use crate::{
	sound::streaming::scheduler::{DecodeScheduler, DecodeSchedulerOutputs},
	util, Frame,
};

use super::{PlaybackInfo, Sound};

const BLOCK_SIZE: usize = 10000;

type DecodedFramesInputs = Vec<Input<Option<Arc<Vec<Frame>>>>>;
type DecodedFramesOutputs = Vec<Output<Option<Arc<Vec<Frame>>>>>;
type StaleDataTimers = Vec<Option<Duration>>;

pub trait Decoder: Send + Sync {
	fn sample_rate(&mut self) -> u32;

	fn frame_count(&mut self) -> usize;

	fn decode(&mut self, frame_indices: Range<usize>) -> Vec<Frame>;
}

pub struct StreamingSound {
	duration: Duration,
	sample_rate: u32,
	playback_info_producer: Producer<PlaybackInfo>,
	decoded_frames_outputs: DecodedFramesOutputs,
	quit_signal_producer: Producer<()>,
}

impl StreamingSound {
	pub fn new(decoder: impl Decoder + 'static) -> Self {
		let DecodeSchedulerOutputs {
			sample_rate,
			duration,
			playback_info_producer,
			decoded_frames_outputs,
			quit_signal_producer,
		} = DecodeScheduler::start(Box::new(decoder));
		Self {
			duration,
			sample_rate,
			playback_info_producer,
			decoded_frames_outputs,
			quit_signal_producer,
		}
	}

	fn frame_at_index(&mut self, index: usize) -> Option<Frame> {
		let block_index = index / BLOCK_SIZE;
		let relative_index = index % BLOCK_SIZE;
		if let Some(output) = self.decoded_frames_outputs.get_mut(block_index) {
			output.read().as_ref().map(|frames| frames[relative_index])
		} else {
			Some(Frame::ZERO)
		}
	}
}

impl Sound for StreamingSound {
	fn duration(&mut self) -> Duration {
		self.duration
	}

	fn frame_at_position(&mut self, position: f64) -> Option<Frame> {
		let sample_position = self.sample_rate as f64 * position;
		let fraction = (sample_position % 1.0) as f32;
		let current_sample_index = sample_position as usize;
		let previous = if current_sample_index == 0 {
			Frame::ZERO
		} else {
			self.frame_at_index(current_sample_index - 1)?
		};
		let current = self.frame_at_index(current_sample_index)?;
		let next_1 = self.frame_at_index(current_sample_index + 1)?;
		let next_2 = self.frame_at_index(current_sample_index + 2)?;
		Some(util::interpolate_frame(
			previous, current, next_1, next_2, fraction,
		))
	}

	fn report_playback_info(&mut self, playback_info: PlaybackInfo) {
		self.playback_info_producer.push(playback_info).ok();
	}
}

impl Drop for StreamingSound {
	fn drop(&mut self) {
		self.quit_signal_producer
			.push(())
			.expect("Failed to send the quit signal to the decode scheduler")
	}
}

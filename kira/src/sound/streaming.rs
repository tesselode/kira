use std::{ops::Range, sync::Arc, time::Duration};

use ringbuf::{Consumer, Producer, RingBuffer};
use triple_buffer::{Input, Output, TripleBuffer};

use crate::{util, Frame};

use super::{PlaybackInfo, Sound};

const BLOCK_SIZE: usize = 10000;
const PLAYBACK_INFO_BUFFER_CAPACITY: usize = 100;
const SCHEDULER_SLEEP_DURATION: Duration = Duration::from_millis(10);
const STALE_DATA_THRESHOLD: Duration = Duration::from_secs(5);

type DecodedFramesInputs = Vec<Input<Option<Arc<Vec<Frame>>>>>;
type DecodedFramesOutputs = Vec<Output<Option<Arc<Vec<Frame>>>>>;
type StaleDataTimers = Vec<Option<Duration>>;

fn block_index_at_position(position: f64, sample_rate: u32) -> usize {
	let frame_index = (position * sample_rate as f64) as usize;
	frame_index / BLOCK_SIZE
}

pub trait Decoder: Send + Sync {
	fn sample_rate(&mut self) -> u32;

	fn frame_count(&mut self) -> usize;

	fn decode(&mut self, frame_indices: Range<usize>) -> Vec<Frame>;
}

struct DecodeScheduler {
	decoder: Box<dyn Decoder>,
	playback_info_consumer: Consumer<PlaybackInfo>,
	decoded_frames_inputs: DecodedFramesInputs,
	stale_data_timers: StaleDataTimers,
	quit_signal_consumer: Consumer<()>,
}

impl DecodeScheduler {
	fn start(mut self) {
		std::thread::spawn(move || loop {
			std::thread::sleep(SCHEDULER_SLEEP_DURATION);
			self.run();
			if self.quit_signal_consumer.pop().is_some() {
				break;
			}
		});
	}

	fn run(&mut self) {
		let sample_rate = self.decoder.sample_rate();
		while let Some(PlaybackInfo { position, .. }) = self.playback_info_consumer.pop() {
			let current_block_index = block_index_at_position(position, sample_rate);
			let mut block_indices = vec![current_block_index];
			if current_block_index > 0 {
				block_indices.insert(0, current_block_index - 1);
			}
			if current_block_index < self.stale_data_timers.len() - 1 {
				block_indices.push(current_block_index + 1);
			}
			for block_index in block_indices {
				if self.stale_data_timers[block_index].is_none() {
					let frame_indices =
						(block_index * BLOCK_SIZE)..((block_index + 1) * BLOCK_SIZE);
					println!("decoding block {} ({:?})", block_index, frame_indices);
					self.decoded_frames_inputs[block_index]
						.write(Some(Arc::new(self.decoder.decode(frame_indices))));
				}
				self.stale_data_timers[block_index] = Some(STALE_DATA_THRESHOLD);
			}
		}
	}
}

pub struct StreamingSound {
	duration: Duration,
	sample_rate: u32,
	playback_info_producer: Producer<PlaybackInfo>,
	decoded_frames_outputs: DecodedFramesOutputs,
	quit_signal_producer: Producer<()>,
}

impl StreamingSound {
	pub fn new(mut decoder: impl Decoder + 'static) -> Self {
		let sample_rate = decoder.sample_rate();
		let duration = Duration::from_secs_f64(decoder.frame_count() as f64 / sample_rate as f64);
		let num_blocks = (decoder.frame_count() as f64 / BLOCK_SIZE as f64).ceil() as usize;
		let mut decoded_frames_inputs = vec![];
		let mut decoded_frames_outputs = vec![];
		for _ in 0..num_blocks {
			let (input, output) = TripleBuffer::new(None).split();
			decoded_frames_inputs.push(input);
			decoded_frames_outputs.push(output);
		}
		let stale_data_timers = vec![None; num_blocks];
		let (playback_info_producer, playback_info_consumer) =
			RingBuffer::new(PLAYBACK_INFO_BUFFER_CAPACITY).split();
		let (quit_signal_producer, quit_signal_consumer) = RingBuffer::new(1).split();
		let decode_scheduler = DecodeScheduler {
			decoder: Box::new(decoder),
			playback_info_consumer,
			decoded_frames_inputs,
			stale_data_timers,
			quit_signal_consumer,
		};
		decode_scheduler.start();
		Self {
			duration,
			sample_rate,
			playback_info_producer,
			decoded_frames_outputs,
			quit_signal_producer,
		}
	}

	fn frame_at_index(&mut self, index: usize) -> Frame {
		let block_index = index / BLOCK_SIZE;
		let relative_index = index % BLOCK_SIZE;
		self.decoded_frames_outputs
			.get_mut(block_index)
			.map(|output| output.read().as_ref().map(|frames| frames[relative_index]))
			.flatten()
			.unwrap_or(Frame::ZERO)
	}
}

impl Sound for StreamingSound {
	fn duration(&mut self) -> Duration {
		self.duration
	}

	fn frame_at_position(&mut self, position: f64) -> Frame {
		let sample_position = self.sample_rate as f64 * position;
		let fraction = (sample_position % 1.0) as f32;
		let current_sample_index = sample_position as usize;
		let previous = if current_sample_index == 0 {
			Frame::ZERO
		} else {
			self.frame_at_index(current_sample_index - 1)
		};
		let current = self.frame_at_index(current_sample_index);
		let next_1 = self.frame_at_index(current_sample_index + 1);
		let next_2 = self.frame_at_index(current_sample_index + 2);
		util::interpolate_frame(previous, current, next_1, next_2, fraction)
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

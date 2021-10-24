use std::{sync::Arc, time::Duration};

use ringbuf::{Consumer, Producer, RingBuffer};
use triple_buffer::TripleBuffer;

use crate::sound::{streaming::BLOCK_SIZE, PlaybackInfo};

use super::{DecodedFramesInputs, DecodedFramesOutputs, Decoder, StaleDataTimers};

const SCHEDULER_SLEEP_DURATION: Duration = Duration::from_millis(10);
const STALE_DATA_THRESHOLD: Duration = Duration::from_secs(5);
const PLAYBACK_INFO_BUFFER_CAPACITY: usize = 100;

pub struct DecodeSchedulerOutputs {
	pub sample_rate: u32,
	pub duration: Duration,
	pub playback_info_producer: Producer<PlaybackInfo>,
	pub decoded_frames_outputs: DecodedFramesOutputs,
	pub quit_signal_producer: Producer<()>,
}

pub struct DecodeScheduler {
	decoder: Box<dyn Decoder>,
	playback_info_consumer: Consumer<PlaybackInfo>,
	decoded_frames_inputs: DecodedFramesInputs,
	stale_data_timers: StaleDataTimers,
}

impl DecodeScheduler {
	pub fn start(mut decoder: Box<dyn Decoder>) -> DecodeSchedulerOutputs {
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
		let (playback_info_producer, playback_info_consumer) =
			RingBuffer::new(PLAYBACK_INFO_BUFFER_CAPACITY).split();
		let mut scheduler = DecodeScheduler {
			decoder,
			playback_info_consumer,
			decoded_frames_inputs,
			stale_data_timers: vec![None; num_blocks],
		};
		let (quit_signal_producer, mut quit_signal_consumer) = RingBuffer::new(1).split();
		std::thread::spawn(move || loop {
			std::thread::sleep(SCHEDULER_SLEEP_DURATION);
			scheduler.run();
			if quit_signal_consumer.pop().is_some() {
				break;
			}
		});
		DecodeSchedulerOutputs {
			sample_rate,
			duration,
			playback_info_producer,
			decoded_frames_outputs,
			quit_signal_producer,
		}
	}

	fn run(&mut self) {
		let sample_rate = self.decoder.sample_rate();
		while let Some(PlaybackInfo { position, .. }) = self.playback_info_consumer.pop() {
			let current_block_index = block_index_at_position(position, sample_rate);
			self.refresh_block(current_block_index);
			if current_block_index > 0 {
				self.refresh_block(current_block_index - 1);
			}
			if current_block_index < self.stale_data_timers.len() - 1 {
				self.refresh_block(current_block_index + 1);
			}
		}
	}

	fn refresh_block(&mut self, index: usize) {
		if self.stale_data_timers[index].is_none() {
			let frame_indices = (index * BLOCK_SIZE)..((index + 1) * BLOCK_SIZE);
			println!("decoding block {} ({:?})", index, frame_indices);
			self.decoded_frames_inputs[index]
				.write(Some(Arc::new(self.decoder.decode(frame_indices))));
		}
		self.stale_data_timers[index] = Some(STALE_DATA_THRESHOLD);
	}
}

fn block_index_at_position(position: f64, sample_rate: u32) -> usize {
	let frame_index = (position * sample_rate as f64) as usize;
	frame_index / BLOCK_SIZE
}

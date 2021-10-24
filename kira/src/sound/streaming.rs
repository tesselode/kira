use std::{
	ops::Range,
	sync::{
		atomic::{AtomicBool, Ordering},
		Arc,
	},
	time::Duration,
};

use ringbuf::{Consumer, Producer, RingBuffer};

use crate::{util, Frame};

use super::{PlaybackInfo, Sound};

const BLOCK_SIZE: usize = 16384;
const DECODER_THREAD_SLEEP_DURATION: Duration = Duration::from_millis(10);

pub trait Decoder: Send + Sync {
	fn sample_rate(&mut self) -> u32;

	fn frame_count(&mut self) -> usize;

	fn decode(&mut self, frame_indices: Range<usize>) -> Vec<Frame>;
}

pub struct StreamingSound {
	duration: Duration,
	sample_rate: u32,
	blocks_needed: Arc<Vec<AtomicBool>>,
	block_consumers: Vec<Consumer<Vec<Frame>>>,
	stale_block_producers: Vec<Producer<Vec<Frame>>>,
	dropped: Arc<AtomicBool>,
}

impl StreamingSound {
	pub fn new(mut decoder: impl Decoder + 'static) -> Self {
		let frame_count = decoder.frame_count();
		let sample_rate = decoder.sample_rate();
		let duration = Duration::from_secs_f64(frame_count as f64 / sample_rate as f64);
		let num_blocks = (frame_count as f64 / BLOCK_SIZE as f64).ceil() as usize;
		let blocks_needed = Arc::new(
			(0..num_blocks)
				.map(|_| AtomicBool::new(false))
				.collect::<Vec<AtomicBool>>(),
		);
		let mut block_producers = vec![];
		let mut block_consumers = vec![];
		for _ in 0..num_blocks {
			let (producer, consumer) = RingBuffer::new(1).split();
			block_producers.push(producer);
			block_consumers.push(consumer);
		}
		let mut stale_block_producers = vec![];
		let mut stale_block_consumers = vec![];
		for _ in 0..num_blocks {
			let (producer, consumer) = RingBuffer::new(1).split();
			stale_block_producers.push(producer);
			stale_block_consumers.push(consumer);
		}
		let dropped = Arc::new(AtomicBool::new(false));
		Self::spawn_decoder_thread(
			Box::new(decoder),
			blocks_needed.clone(),
			block_producers,
			stale_block_consumers,
			dropped.clone(),
		);
		Self {
			duration,
			sample_rate,
			blocks_needed,
			block_consumers,
			stale_block_producers,
			dropped,
		}
	}

	fn spawn_decoder_thread(
		mut decoder: Box<dyn Decoder>,
		blocks_needed: Arc<Vec<AtomicBool>>,
		mut block_producers: Vec<Producer<Vec<Frame>>>,
		mut stale_block_consumers: Vec<Consumer<Vec<Frame>>>,
		dropped: Arc<AtomicBool>,
	) {
		std::thread::spawn(move || {
			loop {
				std::thread::sleep(DECODER_THREAD_SLEEP_DURATION);
				for stale_block_consumer in &mut stale_block_consumers {
					stale_block_consumer.pop();
				}
				for (i, block_producer) in block_producers.iter_mut().enumerate() {
					if block_producer.is_empty() && blocks_needed[i].load(Ordering::SeqCst) {
						let frame_indices = (i * BLOCK_SIZE)..((i + 1) * BLOCK_SIZE);
						block_producer
							.push(decoder.decode(frame_indices.clone()))
							.expect("Block ringbuffer is already full");
						println!("decoded block {} ({:?})", i, frame_indices);
					}
				}
				if dropped.load(Ordering::SeqCst) {
					break;
				}
			}
			println!("decoder thread finished");
		});
	}

	fn frame_at_index(&mut self, index: usize) -> Option<Frame> {
		let block_index = index / BLOCK_SIZE;
		let relative_index = index % BLOCK_SIZE;
		let block_consumer = match self.block_consumers.get(block_index) {
			Some(block_consumer) => block_consumer,
			None => return Some(Frame::ZERO),
		};
		let mut frame = None;
		block_consumer.access(|first_slice, _| {
			let frames = first_slice.get(0);
			frame = frames.map(|frames| frames.get(relative_index).copied().unwrap_or(Frame::ZERO))
		});
		frame
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

	fn report_playback_info(&mut self, PlaybackInfo { position, .. }: PlaybackInfo) {
		let current_block_index = block_index_at_position(position, self.sample_rate);
		if let Some(needed) = self.blocks_needed.get(current_block_index) {
			needed.store(true, Ordering::SeqCst);
		}
		if current_block_index > 0 {
			if let Some(needed) = self.blocks_needed.get(current_block_index - 1) {
				needed.store(true, Ordering::SeqCst);
			}
		}
		if let Some(needed) = self.blocks_needed.get(current_block_index + 1) {
			needed.store(true, Ordering::SeqCst);
		}
	}
}

impl Drop for StreamingSound {
	fn drop(&mut self) {
		self.dropped.store(true, Ordering::SeqCst);
	}
}

fn block_index_at_position(position: f64, sample_rate: u32) -> usize {
	let frame_index = (position * sample_rate as f64) as usize;
	frame_index / BLOCK_SIZE
}

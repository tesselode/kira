use std::{
	sync::{atomic::Ordering, Arc},
	time::Duration,
};

use crate::{
	dsp::Frame,
	sound::{
		streaming::{decoder::Decoder, DecodeSchedulerCommand, StreamingSoundSettings},
		transport::Transport,
	},
};
use ringbuf::{HeapConsumer, HeapProducer, HeapRb};

use super::{Shared, TimestampedFrame};

const BUFFER_SIZE: usize = 16_384;
const DECODER_THREAD_SLEEP_DURATION: Duration = Duration::from_millis(1);

pub(crate) enum NextStep {
	Continue,
	Wait,
	End,
}

pub(crate) struct DecodeScheduler<Error: Send + 'static> {
	decoder: Box<dyn Decoder<Error = Error>>,
	sample_rate: u32,
	num_frames: usize,
	slice: Option<(usize, usize)>,
	transport: Transport,
	decoder_current_frame_index: usize,
	decoded_chunk: Option<DecodedChunk>,
	command_consumer: HeapConsumer<DecodeSchedulerCommand>,
	frame_producer: HeapProducer<TimestampedFrame>,
	error_producer: HeapProducer<Error>,
	shared: Arc<Shared>,
}

impl<Error: Send + 'static> DecodeScheduler<Error> {
	pub fn new(
		mut decoder: Box<dyn Decoder<Error = Error>>,
		slice: Option<(usize, usize)>,
		settings: StreamingSoundSettings,
		shared: Arc<Shared>,
		command_consumer: HeapConsumer<DecodeSchedulerCommand>,
		error_producer: HeapProducer<Error>,
	) -> Result<(Self, HeapConsumer<TimestampedFrame>), Error> {
		let (frame_producer, frame_consumer) = HeapRb::new(BUFFER_SIZE).split();
		let sample_rate = decoder.sample_rate();
		let num_frames = decoder.num_frames();
		let start_sample = settings.start_position.into_samples(sample_rate);
		let decoder_current_frame_index = decoder.seek(start_sample)?;
		let scheduler = Self {
			decoder,
			sample_rate,
			num_frames,
			slice,
			transport: Transport::new(
				num_frames,
				settings.loop_region,
				false,
				sample_rate,
				start_sample,
			),
			decoder_current_frame_index,
			decoded_chunk: None,
			command_consumer,
			frame_producer,
			error_producer,
			shared,
		};
		Ok((scheduler, frame_consumer))
	}

	pub fn current_frame(&self) -> usize {
		self.transport.position
	}

	pub fn start(mut self) {
		std::thread::spawn(move || loop {
			match self.run() {
				Ok(result) => match result {
					NextStep::Continue => {}
					NextStep::Wait => std::thread::sleep(DECODER_THREAD_SLEEP_DURATION),
					NextStep::End => break,
				},
				Err(error) => {
					self.error_producer.push(error).ok();
				}
			}
		});
	}

	pub fn run(&mut self) -> Result<NextStep, Error> {
		// if the sound was manually stopped, end the thread
		if self.shared.stopped.load(Ordering::SeqCst) {
			return Ok(NextStep::End);
		}
		// if the frame ringbuffer is full, sleep for a bit
		if self.frame_producer.is_full() {
			return Ok(NextStep::Wait);
		}
		// check for seek commands
		while let Some(command) = self.command_consumer.pop() {
			match command {
				DecodeSchedulerCommand::SetLoopRegion(loop_region) => self
					.transport
					.set_loop_region(loop_region, self.sample_rate, self.num_frames),
				DecodeSchedulerCommand::SeekBy(amount) => self.seek_by(amount)?,
				DecodeSchedulerCommand::SeekTo(position) => self.seek_to(position)?,
			}
		}
		let frame = self.frame_at_index(self.transport.position)?;
		self.frame_producer
			.push(TimestampedFrame {
				frame,
				index: self.transport.position,
			})
			.expect("could not push frame to frame producer");
		self.transport.increment_position();
		if !self.transport.playing {
			self.shared.reached_end.store(true, Ordering::SeqCst);
			return Ok(NextStep::End);
		}
		Ok(NextStep::Continue)
	}

	pub fn seek_to(&mut self, position: f64) -> Result<(), Error> {
		let index = (position * self.sample_rate as f64).round() as usize;
		self.seek_to_index(index)?;
		Ok(())
	}

	pub fn seek_by(&mut self, amount: f64) -> Result<(), Error> {
		let position = self.shared.position() + amount;
		self.seek_to(position)?;
		Ok(())
	}

	fn frame_at_index(&mut self, index: usize) -> Result<Frame, Error> {
		let start = self.slice.map(|(start, _)| start).unwrap_or(0);
		let end = self.slice.map(|(_, end)| end).unwrap_or(self.num_frames);
		if index >= end - start {
			return Ok(Frame::ZERO);
		}
		let index = start + index;
		// if the requested frame is already loaded, return it
		if let Some(chunk) = &self.decoded_chunk {
			if let Some(frame) = chunk.frame_at_index(index) {
				return Ok(frame);
			}
		}
		/*
			otherwise, seek to the requested index and decode chunks sequentially
			until we get the frame we want. just because we seek to an index does
			not mean the next decoded chunk will have the frame we want (or any frame
			at all, for that matter), so we may need to decode multiple chunks to
			get the frame we care about.
		*/
		if index < self.decoder_current_frame_index {
			self.decoder_current_frame_index = self.decoder.seek(index)?;
		}
		loop {
			let decoded_chunk = DecodedChunk {
				start_index: self.decoder_current_frame_index,
				frames: self.decoder.decode()?,
			};
			self.decoder_current_frame_index += decoded_chunk.frames.len();
			self.decoded_chunk = Some(decoded_chunk);
			if let Some(chunk) = &self.decoded_chunk {
				if let Some(frame) = chunk.frame_at_index(index) {
					return Ok(frame);
				}
			}
		}
	}

	fn seek_to_index(&mut self, index: usize) -> Result<(), Error> {
		self.transport.seek_to(index);
		self.decoder_current_frame_index = self.decoder.seek(index)?;
		Ok(())
	}
}

struct DecodedChunk {
	pub start_index: usize,
	pub frames: Vec<Frame>,
}

impl DecodedChunk {
	fn frame_at_index(&self, index: usize) -> Option<Frame> {
		if index < self.start_index {
			return None;
		}
		self.frames.get(index - self.start_index).copied()
	}
}

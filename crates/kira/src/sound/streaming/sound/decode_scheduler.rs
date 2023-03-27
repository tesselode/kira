use std::{
	collections::VecDeque,
	convert::TryInto,
	sync::{atomic::Ordering, Arc},
	time::Duration,
};

use crate::{
	dsp::Frame,
	sound::{
		static_sound::PlaybackState,
		streaming::{
			decoder::{DecodeResponse, Decoder},
			DecodeSchedulerCommand, StreamingSoundSettings,
		},
	},
	LoopBehavior,
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
	command_consumer: HeapConsumer<DecodeSchedulerCommand>,
	sample_rate: u32,
	loop_behavior: Option<LoopBehavior>,
	frame_producer: HeapProducer<TimestampedFrame>,
	decoded_frames: VecDeque<Frame>,
	current_frame_index: i64,
	error_producer: HeapProducer<Error>,
	shared: Arc<Shared>,
}

impl<Error: Send + 'static> DecodeScheduler<Error> {
	pub fn new(
		decoder: Box<dyn Decoder<Error = Error>>,
		settings: StreamingSoundSettings,
		shared: Arc<Shared>,
		command_consumer: HeapConsumer<DecodeSchedulerCommand>,
		error_producer: HeapProducer<Error>,
	) -> Result<(Self, HeapConsumer<TimestampedFrame>), Error> {
		let (mut frame_producer, frame_consumer) = HeapRb::new(BUFFER_SIZE).split();
		// pre-seed the frame ringbuffer with a zero frame. this is the "previous" frame
		// when the sound just started.
		frame_producer
			.push(TimestampedFrame {
				frame: Frame::ZERO,
				index: 0,
			})
			.expect("The frame producer shouldn't be full because we just created it");
		let sample_rate = decoder.sample_rate();
		let mut scheduler = Self {
			decoder,
			command_consumer,
			sample_rate,
			loop_behavior: settings.loop_behavior,
			frame_producer,
			decoded_frames: VecDeque::new(),
			current_frame_index: 0,
			error_producer,
			shared,
		};
		scheduler.seek_to(settings.start_position)?;
		Ok((scheduler, frame_consumer))
	}

	pub fn current_frame(&self) -> i64 {
		self.current_frame_index
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
		if self.shared.state() == PlaybackState::Stopped {
			return Ok(NextStep::End);
		}
		// if the frame ringbuffer is full, sleep for a bit
		if self.frame_producer.is_full() {
			return Ok(NextStep::Wait);
		}
		// check for seek commands
		while let Some(command) = self.command_consumer.pop() {
			match command {
				DecodeSchedulerCommand::SeekBy(amount) => self.seek_by(amount)?,
				DecodeSchedulerCommand::SeekTo(position) => self.seek_to(position)?,
			}
		}
		// if we have leftover frames from the last decode, push
		// those first
		if let Some(frame) = self.decoded_frames.pop_front() {
			self.frame_producer
				.push(TimestampedFrame {
					frame,
					index: self.current_frame_index,
				})
				.expect("Frame producer should not be full because we just checked that");
			self.current_frame_index += 1;
		// otherwise, if the current position is negative, push silence
		} else if self.current_frame_index < 0 {
			self.frame_producer
				.push(TimestampedFrame {
					frame: Frame::ZERO,
					index: self.current_frame_index,
				})
				.expect("Frame producer should not be full because we just checked that");
			self.current_frame_index += 1;
		// otherwise, decode some new frames
		} else {
			match self.decoder.decode()? {
				DecodeResponse::DecodedFrames(frames) => {
					self.decoded_frames.extend(frames.iter().copied());
				}
				DecodeResponse::ReachedEndOfAudio => {
					// if there aren't any new frames and the sound is looping,
					// seek back to the loop position
					if let Some(LoopBehavior { start_position }) = self.loop_behavior {
						self.seek_to(start_position)?;
					// otherwise, tell the sound to finish and end the thread
					} else {
						self.shared.reached_end.store(true, Ordering::SeqCst);
						return Ok(NextStep::End);
					}
				}
			}
		}
		Ok(NextStep::Continue)
	}

	fn seek_to(&mut self, position: f64) -> Result<(), Error> {
		let index = (position * self.sample_rate as f64).round() as i64;
		self.seek_to_index(index)?;
		Ok(())
	}

	fn seek_by(&mut self, amount: f64) -> Result<(), Error> {
		let position = self.shared.position() + amount;
		self.seek_to(position)?;
		Ok(())
	}

	fn seek_to_index(&mut self, index: i64) -> Result<(), Error> {
		if index < 0 {
			self.current_frame_index = index;
			self.decoder.seek(0)?;
		} else {
			let desired_index = index.try_into().expect("can't convert i64 to u64");
			self.current_frame_index = self
				.decoder
				.seek(desired_index)?
				.try_into()
				.expect("sound is too long, cannot convert u64 to i64");
		}
		Ok(())
	}
}

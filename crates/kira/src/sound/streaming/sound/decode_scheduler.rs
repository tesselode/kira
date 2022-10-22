use std::{
	collections::VecDeque,
	convert::TryInto,
	sync::{
		atomic::{AtomicBool, AtomicI64, Ordering},
		Arc,
	},
	time::Duration,
};

use crate::{
	dsp::Frame,
	sound::streaming::{decoder::Decoder, StreamingSoundSettings},
	LoopBehavior,
};
use ringbuf::{HeapConsumer, HeapProducer, HeapRb};

const BUFFER_SIZE: usize = 16_384;
const SEEK_DESTINATION_NONE: i64 = i64::MAX;
const DECODER_THREAD_SLEEP_DURATION: Duration = Duration::from_millis(1);

pub(crate) enum NextStep {
	Continue,
	Wait,
	End,
}

pub(crate) struct DecodeSchedulerController {
	frame_consumer: HeapConsumer<(i64, Frame)>,
	seek_destination_sender: Arc<AtomicI64>,
	stopped_signal_sender: Arc<AtomicBool>,
	finished_signal_receiver: Arc<AtomicBool>,
}

impl DecodeSchedulerController {
	pub fn frame_consumer_mut(&mut self) -> &mut HeapConsumer<(i64, Frame)> {
		&mut self.frame_consumer
	}

	pub fn seek(&self, index: i64) {
		self.seek_destination_sender.store(index, Ordering::SeqCst);
	}

	pub fn finished(&self) -> bool {
		self.finished_signal_receiver.load(Ordering::SeqCst)
	}
}

impl Drop for DecodeSchedulerController {
	fn drop(&mut self) {
		self.stopped_signal_sender.store(true, Ordering::SeqCst);
	}
}

pub(crate) struct DecodeScheduler<Error: Send + 'static> {
	decoder: Box<dyn Decoder<Error = Error>>,
	sample_rate: u32,
	loop_behavior: Option<LoopBehavior>,
	frame_producer: HeapProducer<(i64, Frame)>,
	seek_destination_receiver: Arc<AtomicI64>,
	stopped_signal_receiver: Arc<AtomicBool>,
	finished_signal_sender: Arc<AtomicBool>,
	decoded_frames: VecDeque<Frame>,
	current_frame: i64,
	error_producer: HeapProducer<Error>,
}

impl<Error: Send + 'static> DecodeScheduler<Error> {
	pub fn new(
		decoder: Box<dyn Decoder<Error = Error>>,
		settings: StreamingSoundSettings,
		error_producer: HeapProducer<Error>,
	) -> Result<(Self, DecodeSchedulerController), Error> {
		let (mut frame_producer, frame_consumer) = HeapRb::new(BUFFER_SIZE).split();
		// pre-seed the frame ringbuffer with a zero frame. this is the "previous" frame
		// when the sound just started.
		frame_producer
			.push((0, Frame::ZERO))
			.expect("The frame producer shouldn't be full because we just created it");
		let seek_destination_sender = Arc::new(AtomicI64::new(SEEK_DESTINATION_NONE));
		let seek_destination_receiver = seek_destination_sender.clone();
		let stopped_signal_sender = Arc::new(AtomicBool::new(false));
		let stopped_signal_receiver = stopped_signal_sender.clone();
		let finished_signal_sender = Arc::new(AtomicBool::new(false));
		let finished_signal_receiver = finished_signal_sender.clone();
		let sample_rate = decoder.sample_rate();
		let mut scheduler = Self {
			decoder,
			sample_rate,
			loop_behavior: settings.loop_behavior,
			frame_producer,
			seek_destination_receiver,
			stopped_signal_receiver,
			finished_signal_sender,
			decoded_frames: VecDeque::new(),
			current_frame: 0,
			error_producer,
		};
		scheduler.seek(settings.start_position)?;
		let controller = DecodeSchedulerController {
			frame_consumer,
			seek_destination_sender,
			stopped_signal_sender,
			finished_signal_receiver,
		};
		Ok((scheduler, controller))
	}

	pub fn current_frame(&self) -> i64 {
		self.current_frame
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
		if self.stopped_signal_receiver.load(Ordering::SeqCst) {
			return Ok(NextStep::End);
		}
		// if the frame ringbuffer is full, sleep for a bit
		if self.frame_producer.is_full() {
			return Ok(NextStep::Wait);
		}
		// check for seek commands
		let seek_destination = self.seek_destination_receiver.load(Ordering::SeqCst);
		if seek_destination != SEEK_DESTINATION_NONE {
			self.seek_destination_receiver
				.store(SEEK_DESTINATION_NONE, Ordering::SeqCst);
			self.seek_to_index(seek_destination)?;
		}
		// if we have leftover frames from the last decode, push
		// those first
		if let Some(frame) = self.decoded_frames.pop_front() {
			self.frame_producer
				.push((self.current_frame, frame))
				.expect("Frame producer should not be full because we just checked that");
			self.current_frame += 1;
		// otherwise, if the current position is negative, push silence
		} else if self.current_frame < 0 {
			self.frame_producer
				.push((self.current_frame, Frame::ZERO))
				.expect("Frame producer should not be full because we just checked that");
			self.current_frame += 1;
		// otherwise, decode some new frames
		} else {
			let reached_end_of_file = self.decoder.decode(&mut self.decoded_frames)?;
			if reached_end_of_file {
				// if there aren't any new frames and the sound is looping,
				// seek back to the loop position
				if let Some(LoopBehavior { start_position }) = self.loop_behavior {
					self.seek(start_position)?;
				// otherwise, tell the sound to finish and end the thread
				} else {
					self.finished_signal_sender.store(true, Ordering::SeqCst);
					return Ok(NextStep::End);
				}
			}
		}
		Ok(NextStep::Continue)
	}

	fn seek(&mut self, position: f64) -> Result<(), Error> {
		let index = (position * self.sample_rate as f64).round() as i64;
		self.seek_to_index(index)?;
		Ok(())
	}

	fn seek_to_index(&mut self, index: i64) -> Result<(), Error> {
		if index < 0 {
			self.current_frame = index;
			self.decoder.seek(0)?;
		} else {
			let desired_index = index.try_into().expect("can't convert i64 to u64");
			self.current_frame = self
				.decoder
				.seek(desired_index)?
				.try_into()
				.expect("sound is too long, cannot convert u64 to i64");
		}
		Ok(())
	}
}

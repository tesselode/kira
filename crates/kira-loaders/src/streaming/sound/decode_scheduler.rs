use std::{
	collections::VecDeque,
	sync::{
		atomic::{AtomicBool, AtomicU64, Ordering},
		Arc,
	},
	time::Duration,
};

use kira::{dsp::Frame, LoopBehavior};
use ringbuf::Producer;

use crate::{
	streaming::{decoder::Decoder, sound::SEEK_DESTINATION_NONE},
	StreamingSoundData,
};

const DECODER_THREAD_SLEEP_DURATION: Duration = Duration::from_millis(1);

pub struct DecodeScheduler<Error: Send + 'static> {
	decoder: Box<dyn Decoder<Error = Error>>,
	sample_rate: u32,
	loop_behavior: Option<LoopBehavior>,
	frame_producer: Producer<(u64, Frame)>,
	seek_destination_receiver: Arc<AtomicU64>,
	stopped_signal_receiver: Arc<AtomicBool>,
	finished_signal_sender: Arc<AtomicBool>,
	decoded_frames: VecDeque<Frame>,
	current_frame: u64,
}

impl<Error: Send + 'static> DecodeScheduler<Error> {
	pub fn new(
		data: StreamingSoundData<Error>,
		frame_producer: Producer<(u64, Frame)>,
		seek_destination_receiver: Arc<AtomicU64>,
		stopped_signal_receiver: Arc<AtomicBool>,
		finished_signal_sender: Arc<AtomicBool>,
	) -> Result<Self, Error> {
		let sample_rate = data.decoder.sample_rate();
		let mut scheduler = Self {
			decoder: data.decoder,
			sample_rate,
			loop_behavior: data.settings.loop_behavior,
			frame_producer,
			seek_destination_receiver,
			stopped_signal_receiver,
			finished_signal_sender,
			decoded_frames: VecDeque::new(),
			current_frame: 0,
		};
		scheduler.seek(data.settings.start_position)?;
		Ok(scheduler)
	}

	pub fn current_frame(&self) -> u64 {
		self.current_frame
	}

	pub fn start(mut self, mut error_producer: Producer<Error>) {
		std::thread::spawn(move || loop {
			match self.run() {
				Ok(should_end_thread) => {
					if should_end_thread {
						break;
					}
				}
				Err(error) => {
					error_producer.push(error).ok();
					break;
				}
			}
		});
	}

	fn run(&mut self) -> Result<bool, Error> {
		// if the sound was manually stopped, end the thread
		if self.stopped_signal_receiver.load(Ordering::SeqCst) {
			return Ok(true);
		}
		// if the frame ringbuffer is full, sleep for a bit
		if self.frame_producer.is_full() {
			std::thread::sleep(DECODER_THREAD_SLEEP_DURATION);
			return Ok(false);
		}
		// check for seek commands
		let seek_destination = self.seek_destination_receiver.load(Ordering::SeqCst);
		if seek_destination != SEEK_DESTINATION_NONE {
			self.current_frame = self.decoder.seek(seek_destination)?;
			self.seek_destination_receiver
				.store(SEEK_DESTINATION_NONE, Ordering::SeqCst);
		}
		// if we have leftover frames from the last decode, push
		// those first
		if let Some(frame) = self.decoded_frames.pop_front() {
			self.frame_producer
				.push((self.current_frame, frame))
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
					return Ok(true);
				}
			}
		}
		Ok(false)
	}

	fn seek(&mut self, position: f64) -> Result<(), Error> {
		let index = (position * self.sample_rate as f64).round() as u64;
		self.current_frame = self.decoder.seek(index)?;
		Ok(())
	}
}

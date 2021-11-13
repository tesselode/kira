use std::{
	collections::VecDeque,
	sync::{
		atomic::{AtomicBool, AtomicUsize, Ordering},
		Arc,
	},
	time::Duration,
};

use kira::{dsp::Frame, LoopBehavior};
use ringbuf::Producer;

use crate::Decoder;

use super::SEEK_DESTINATION_NONE;

const DECODER_THREAD_SLEEP_DURATION: Duration = Duration::from_millis(1);

type ShouldEndThread = bool;

pub struct DecoderWrapper<E: Send + Sync + 'static> {
	decoder: Box<dyn Decoder<Error = E>>,
	loop_behavior: Option<LoopBehavior>,
	frame_producer: Producer<(usize, Frame)>,
	seek_destination_receiver: Arc<AtomicUsize>,
	stopped_signal_receiver: Arc<AtomicBool>,
	finished_signal_sender: Arc<AtomicBool>,
	decoded_frames: VecDeque<Frame>,
	current_frame: usize,
}

impl<E: Send + Sync + 'static> DecoderWrapper<E> {
	pub fn new(
		decoder: Box<dyn Decoder<Error = E>>,
		start_position: f64,
		loop_behavior: Option<LoopBehavior>,
		frame_producer: Producer<(usize, Frame)>,
		seek_destination_receiver: Arc<AtomicUsize>,
		stopped_signal_receiver: Arc<AtomicBool>,
		finished_signal_sender: Arc<AtomicBool>,
	) -> Result<Self, E> {
		let mut wrapper = Self {
			decoder,
			loop_behavior,
			frame_producer,
			seek_destination_receiver,
			stopped_signal_receiver,
			finished_signal_sender,
			decoded_frames: VecDeque::new(),
			current_frame: 0,
		};
		wrapper.seek(start_position)?;
		Ok(wrapper)
	}

	pub fn current_frame(&self) -> usize {
		self.current_frame
	}

	pub fn start(mut self, mut error_producer: Producer<E>) {
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

	fn run(&mut self) -> Result<ShouldEndThread, E> {
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
			self.seek_to_index(seek_destination)?;
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
		} else if let Some(frames) = self.decoder.decode()? {
			self.decoded_frames = frames;
		// if there aren't any new frames and the sound is looping,
		// seek back to the loop position
		} else if let Some(LoopBehavior { start_position }) = self.loop_behavior {
			self.seek(start_position)?;
		// otherwise, tell the sound to finish and end the thread
		} else {
			self.finished_signal_sender.store(true, Ordering::SeqCst);
			return Ok(true);
		}
		Ok(false)
	}

	fn reset(&mut self) -> Result<(), E> {
		self.decoder.reset()?;
		self.current_frame = 0;
		self.decoded_frames.clear();
		Ok(())
	}

	fn seek_to_index(&mut self, index: usize) -> Result<(), E> {
		if self.current_frame > index {
			self.reset()?;
		}
		while self.current_frame + self.decoded_frames.len() < index {
			self.current_frame += self.decoded_frames.len();
			if let Some(frames) = self.decoder.decode()? {
				self.decoded_frames = frames;
			} else {
				// if we've reached the end of the audio data and a loop behavior
				// is set, wrap around to the start position
				if let Some(LoopBehavior { start_position }) = self.loop_behavior {
					let start_index =
						(start_position * self.decoder.sample_rate() as f64).round() as usize;
					self.seek_to_index(start_index + index - self.current_frame)?;
				}
				break;
			}
		}
		Ok(())
	}

	fn seek(&mut self, position: f64) -> Result<(), E> {
		let index = (position * self.decoder.sample_rate() as f64).round() as usize;
		self.seek_to_index(index)?;
		Ok(())
	}
}

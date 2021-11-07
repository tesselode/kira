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

pub struct DecoderWrapper {
	decoder: Box<dyn Decoder>,
	loop_behavior: Option<LoopBehavior>,
	frame_producer: Producer<(usize, Frame)>,
	seek_destination_receiver: Arc<AtomicUsize>,
	stopped_signal_receiver: Arc<AtomicBool>,
	finished_signal_sender: Arc<AtomicBool>,
	decoded_frames: VecDeque<Frame>,
	current_frame: usize,
}

impl DecoderWrapper {
	pub fn new(
		decoder: Box<dyn Decoder>,
		loop_behavior: Option<LoopBehavior>,
		frame_producer: Producer<(usize, Frame)>,
		seek_destination_receiver: Arc<AtomicUsize>,
		stopped_signal_receiver: Arc<AtomicBool>,
		finished_signal_sender: Arc<AtomicBool>,
	) -> Self {
		Self {
			decoder,
			loop_behavior,
			frame_producer,
			seek_destination_receiver,
			stopped_signal_receiver,
			finished_signal_sender,
			decoded_frames: VecDeque::new(),
			current_frame: 0,
		}
	}

	pub fn start(mut self) {
		std::thread::spawn(move || loop {
			let should_end_thread = self.run();
			if should_end_thread {
				break;
			}
		});
	}

	fn run(&mut self) -> ShouldEndThread {
		// if the sound was manually stopped, end the thread
		if self.stopped_signal_receiver.load(Ordering::SeqCst) {
			return true;
		}
		// if the frame ringbuffer is full, sleep for a bit
		if self.frame_producer.is_full() {
			std::thread::sleep(DECODER_THREAD_SLEEP_DURATION);
			return false;
		}
		// check for seek commands
		let seek_destination = self.seek_destination_receiver.load(Ordering::SeqCst);
		if seek_destination != SEEK_DESTINATION_NONE {
			self.seek(seek_destination);
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
		} else if let Some(frames) = self.decoder.decode() {
			self.decoded_frames = frames;
		// if there aren't any new frames and the sound is looping,
		// seek back to the loop position
		} else if let Some(LoopBehavior { start_position }) = self.loop_behavior {
			let destination = (start_position * self.decoder.sample_rate() as f64).round() as usize;
			self.seek(destination);
		// otherwise, tell the sound to finish and end the thread
		} else {
			self.finished_signal_sender.store(true, Ordering::SeqCst);
			return true;
		}
		false
	}

	fn reset(&mut self) {
		self.decoder.reset();
		self.current_frame = 0;
		self.decoded_frames.clear();
	}

	fn seek(&mut self, destination: usize) {
		if self.current_frame > destination {
			self.reset();
		}
		while self.current_frame + self.decoded_frames.len() < destination {
			self.current_frame += self.decoded_frames.len();
			if let Some(frames) = self.decoder.decode() {
				self.decoded_frames = frames;
			} else {
				// if we've reached the end of the audio data and a loop behavior
				// is set, wrap around to the start position
				if let Some(LoopBehavior { start_position }) = self.loop_behavior {
					let start_index =
						(start_position * self.decoder.sample_rate() as f64).round() as usize;
					self.seek(start_index + destination - self.current_frame);
				}
				break;
			}
		}
	}
}

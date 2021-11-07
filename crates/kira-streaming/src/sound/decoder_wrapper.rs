use std::{
	collections::VecDeque,
	sync::{
		atomic::{AtomicBool, Ordering},
		Arc,
	},
	time::Duration,
};

use kira::{dsp::Frame, LoopBehavior};
use ringbuf::Producer;

use crate::Decoder;

const DECODER_THREAD_SLEEP_DURATION: Duration = Duration::from_millis(1);

type ShouldEndThread = bool;

pub struct DecoderWrapper {
	decoder: Box<dyn Decoder>,
	loop_behavior: Option<LoopBehavior>,
	frame_producer: Producer<(usize, Frame)>,
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
		stopped_signal_receiver: Arc<AtomicBool>,
		finished_signal_sender: Arc<AtomicBool>,
	) -> Self {
		Self {
			decoder,
			loop_behavior,
			frame_producer,
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
			if let Some((sample_index, frames)) = self.seek(start_position) {
				self.current_frame = sample_index;
				self.decoded_frames = frames;
			}
		// otherwise, tell the sound to finish and end the thread
		} else {
			self.finished_signal_sender.store(true, Ordering::SeqCst);
			return true;
		}
		false
	}

	fn seek(&mut self, position: f64) -> Option<(usize, VecDeque<Frame>)> {
		let mut samples_to_skip = (position * self.decoder.sample_rate() as f64).round() as usize;
		let mut current_sample = 0;
		self.decoder.reset();
		while let Some(frames) = self.decoder.decode() {
			if samples_to_skip < frames.len() {
				return Some((current_sample, frames));
			}
			samples_to_skip -= frames.len();
			current_sample += frames.len();
		}
		None
	}
}

use std::{
	collections::VecDeque,
	sync::{
		atomic::{AtomicBool, AtomicU64, Ordering},
		Arc,
	},
	time::Duration,
};

use ringbuf::{Consumer, RingBuffer};

use kira::{
	dsp::Frame,
	manager::resources::{Clocks, Parameters},
	sound::{static_sound::PlaybackState, Sound},
	track::TrackId,
	util, LoopBehavior,
};

use super::data::StreamingSoundData;

const BUFFER_SIZE: usize = 16_384;
const DECODER_THREAD_SLEEP_DURATION: Duration = Duration::from_millis(1);

pub(crate) struct Shared {
	position: AtomicU64,
}

impl Shared {
	pub fn position(&self) -> f64 {
		f64::from_bits(self.position.load(Ordering::SeqCst))
	}
}

pub(crate) struct StreamingSound {
	sample_rate: u32,
	frame_consumer: Consumer<(usize, Frame)>,
	stopped_signal_sender: Arc<AtomicBool>,
	finished_signal_receiver: Arc<AtomicBool>,
	state: PlaybackState,
	current_frame: usize,
	fractional_position: f64,
	shared: Arc<Shared>,
}

impl StreamingSound {
	pub fn new(mut data: StreamingSoundData) -> Self {
		let sample_rate = data.decoder.sample_rate();
		let loop_behavior = data.settings.loop_behavior;
		let (mut frame_producer, frame_consumer) = RingBuffer::new(BUFFER_SIZE).split();
		frame_producer
			.push((0, Frame::ZERO))
			.expect("The frame producer shouldn't be full because we just created it");
		let stopped_signal_sender = Arc::new(AtomicBool::new(false));
		let stopped_signal_receiver = stopped_signal_sender.clone();
		let finished_signal_sender = Arc::new(AtomicBool::new(false));
		let finished_signal_receiver = finished_signal_sender.clone();
		std::thread::spawn(move || {
			let mut decoded_frames = VecDeque::new();
			let mut current_frame = 0;
			loop {
				if stopped_signal_receiver.load(Ordering::SeqCst) {
					break;
				}
				if frame_producer.is_full() {
					std::thread::sleep(DECODER_THREAD_SLEEP_DURATION);
					continue;
				}
				if let Some(frame) = decoded_frames.pop_front() {
					frame_producer
						.push((current_frame, frame))
						.expect("Frame producer should not be full because we just checked that");
					current_frame += 1;
				} else if let Some(frames) = data.decoder.decode() {
					decoded_frames = frames;
				} else if let Some(LoopBehavior { start_position }) = loop_behavior {
					if let Some((sample_index, frames)) = seek(start_position, &mut data) {
						current_frame = sample_index;
						decoded_frames = frames;
					}
				} else {
					finished_signal_sender.store(true, Ordering::SeqCst);
					break;
				}
			}
		});
		Self {
			sample_rate,
			frame_consumer,
			stopped_signal_sender,
			finished_signal_receiver,
			state: PlaybackState::Playing,
			current_frame: 0,
			fractional_position: 0.0,
			shared: Arc::new(Shared {
				position: AtomicU64::new(0.0f64.to_bits()),
			}),
		}
	}

	pub fn shared(&self) -> Arc<Shared> {
		self.shared.clone()
	}

	fn update_current_frame(&mut self) {
		self.frame_consumer.access(|a, b| {
			let mut iter = a.iter().chain(b.iter());
			if let Some((index, _)) = iter.nth(1) {
				self.current_frame = *index;
			}
		});
	}

	fn next_frames(&self) -> [Frame; 4] {
		let mut frames = [Frame::ZERO; 4];
		self.frame_consumer.access(|a, b| {
			let mut iter = a.iter().chain(b.iter());
			for frame in &mut frames {
				*frame = iter
					.next()
					.copied()
					.map(|(_, frame)| frame)
					.unwrap_or(Frame::ZERO);
			}
		});
		frames
	}

	fn position(&self) -> f64 {
		(self.current_frame as f64 + self.fractional_position) / self.sample_rate as f64
	}
}

fn seek(position: f64, data: &mut StreamingSoundData) -> Option<(usize, VecDeque<Frame>)> {
	let mut samples_to_skip = (position * data.decoder.sample_rate() as f64).round() as usize;
	let mut current_sample = 0;
	data.decoder.reset();
	while let Some(frames) = data.decoder.decode() {
		if samples_to_skip < frames.len() {
			return Some((current_sample, frames));
		}
		samples_to_skip -= frames.len();
		current_sample += frames.len();
	}
	None
}

impl Sound for StreamingSound {
	fn track(&mut self) -> TrackId {
		TrackId::Main
	}

	fn process(&mut self, dt: f64, _parameters: &Parameters, _clocks: &Clocks) -> Frame {
		// pause playback while waiting for audio data. the first frame
		// in the ringbuffer is the previous frame, so we need to make
		// sure there's at least 2 before we continue playing.
		if self.frame_consumer.len() < 2 && !self.finished_signal_receiver.load(Ordering::SeqCst) {
			return Frame::ZERO;
		}
		self.update_current_frame();
		let next_frames = self.next_frames();
		let out = util::interpolate_frame(
			next_frames[0],
			next_frames[1],
			next_frames[2],
			next_frames[3],
			self.fractional_position as f32,
		);
		self.fractional_position += self.sample_rate as f64 * dt;
		while self.fractional_position >= 1.0 {
			self.fractional_position -= 1.0;
			self.frame_consumer.pop();
		}
		if self.finished_signal_receiver.load(Ordering::SeqCst) && self.frame_consumer.is_empty() {
			self.state = PlaybackState::Stopped;
		}
		out
	}

	fn on_start_processing(&mut self) {
		self.shared
			.position
			.store(self.position().to_bits(), Ordering::SeqCst);
	}

	fn finished(&self) -> bool {
		self.state == PlaybackState::Stopped
	}
}

impl Drop for StreamingSound {
	fn drop(&mut self) {
		self.stopped_signal_sender.store(true, Ordering::SeqCst);
		println!("dropped sound");
	}
}

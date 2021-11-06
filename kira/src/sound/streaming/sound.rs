use std::{
	collections::VecDeque,
	sync::{
		atomic::{AtomicBool, Ordering},
		Arc,
	},
	time::Duration,
};

use ringbuf::{Consumer, RingBuffer};

use crate::{
	dsp::Frame,
	manager::resources::{Clocks, Parameters},
	sound::{static_sound::PlaybackState, Sound},
	track::TrackId,
	util,
};

use super::data::StreamingSoundData;

const BUFFER_SIZE: usize = 16_384;
const DECODER_THREAD_SLEEP_DURATION: Duration = Duration::from_millis(1);

pub struct StreamingSound {
	sample_rate: u32,
	frame_consumer: Consumer<Frame>,
	quit_signal_sender: Arc<AtomicBool>,
	state: PlaybackState,
	fractional_position: f64,
}

impl StreamingSound {
	pub fn new(mut data: StreamingSoundData) -> Self {
		let sample_rate = data.decoder.sample_rate();
		let (mut frame_producer, frame_consumer) = RingBuffer::new(BUFFER_SIZE).split();
		frame_producer
			.push(Frame::ZERO)
			.expect("The frame producer shouldn't be full because we just created it");
		let quit_signal_sender = Arc::new(AtomicBool::new(false));
		let quit_signal_receiver = quit_signal_sender.clone();
		std::thread::spawn(move || {
			let mut decoded_frames = VecDeque::new();
			loop {
				if quit_signal_receiver.load(Ordering::SeqCst) {
					break;
				}
				if frame_producer.is_full() {
					std::thread::sleep(DECODER_THREAD_SLEEP_DURATION);
					continue;
				}
				if let Some(frame) = decoded_frames.pop_front() {
					frame_producer
						.push(frame)
						.expect("Frame producer should not be full because we just checked that");
				} else {
					decoded_frames = data.decoder.decode();
				}
			}
			println!("stopping decoder thread");
		});
		Self {
			sample_rate,
			frame_consumer,
			quit_signal_sender,
			state: PlaybackState::Playing,
			fractional_position: 0.0,
		}
	}

	fn next_frames(&self) -> [Frame; 4] {
		let mut frames = [Frame::ZERO; 4];
		self.frame_consumer.access(|a, b| {
			let mut iter = a.iter().chain(b.iter());
			for frame in &mut frames {
				*frame = iter.next().copied().unwrap_or(Frame::ZERO);
			}
		});
		frames
	}
}

impl Sound for StreamingSound {
	fn track(&mut self) -> TrackId {
		TrackId::Main
	}

	fn process(&mut self, dt: f64, _parameters: &Parameters, _clocks: &Clocks) -> Frame {
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
		out
	}

	fn finished(&self) -> bool {
		self.state == PlaybackState::Stopped
	}
}

impl Drop for StreamingSound {
	fn drop(&mut self) {
		self.quit_signal_sender.store(true, Ordering::SeqCst);
	}
}

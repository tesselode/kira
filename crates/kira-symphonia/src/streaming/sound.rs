mod decoder_wrapper;

use std::sync::{
	atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering},
	Arc,
};

use kira::{
	clock::Clocks,
	dsp::{interpolate_frame, Frame},
	parameter::Parameters,
	sound::{static_sound::PlaybackState, Sound},
	track::TrackId,
};
use ringbuf::{Consumer, Producer, RingBuffer};

use crate::{Command, Error, StreamingSoundData};

use self::decoder_wrapper::DecoderWrapper;

const BUFFER_SIZE: usize = 16_384;
const SEEK_DESTINATION_NONE: u64 = u64::MAX;

pub(crate) struct Shared {
	state: AtomicU8,
	position: AtomicU64,
}

impl Shared {
	pub fn state(&self) -> PlaybackState {
		match self.state.load(Ordering::SeqCst) {
			0 => PlaybackState::Playing,
			1 => PlaybackState::Pausing,
			2 => PlaybackState::Paused,
			3 => PlaybackState::Stopping,
			4 => PlaybackState::Stopped,
			_ => panic!("Invalid playback state"),
		}
	}

	pub fn position(&self) -> f64 {
		f64::from_bits(self.position.load(Ordering::SeqCst))
	}
}

pub(crate) struct StreamingSound {
	command_consumer: Consumer<Command>,
	sample_rate: u32,
	frame_consumer: Consumer<(u64, Frame)>,
	seek_destination_sender: Arc<AtomicU64>,
	stopped_signal_sender: Arc<AtomicBool>,
	finished_signal_receiver: Arc<AtomicBool>,
	state: PlaybackState,
	current_frame: u64,
	fractional_position: f64,
	shared: Arc<Shared>,
}

impl StreamingSound {
	pub fn new(
		data: StreamingSoundData,
		command_consumer: Consumer<Command>,
		error_producer: Producer<Error>,
	) -> Result<Self, Error> {
		let (mut frame_producer, frame_consumer) = RingBuffer::new(BUFFER_SIZE).split();
		// pre-seed the frame ringbuffer with a zero frame. this is the "previous" frame
		// when the sound just started.
		frame_producer
			.push((0, Frame::ZERO))
			.expect("The frame producer shouldn't be full because we just created it");
		let seek_destination_sender = Arc::new(AtomicU64::new(SEEK_DESTINATION_NONE));
		let seek_destination_receiver = seek_destination_sender.clone();
		let stopped_signal_sender = Arc::new(AtomicBool::new(false));
		let stopped_signal_receiver = stopped_signal_sender.clone();
		let finished_signal_sender = Arc::new(AtomicBool::new(false));
		let finished_signal_receiver = finished_signal_sender.clone();
		let sample_rate = data.sample_rate;
		let decoder_wrapper = DecoderWrapper::new(
			data.format_reader,
			data.decoder,
			data.sample_rate,
			data.track_id,
			frame_producer,
			seek_destination_receiver,
			stopped_signal_receiver,
			finished_signal_sender,
		);
		let current_frame = 0;
		decoder_wrapper.start(error_producer);
		Ok(Self {
			command_consumer,
			sample_rate,
			frame_consumer,
			seek_destination_sender,
			stopped_signal_sender,
			finished_signal_receiver,
			state: PlaybackState::Playing,
			current_frame,
			fractional_position: 0.0,
			shared: Arc::new(Shared {
				position: AtomicU64::new(0.0f64.to_bits()),
				state: AtomicU8::new(PlaybackState::Playing as u8),
			}),
		})
	}

	fn set_state(&mut self, state: PlaybackState) {
		self.state = state;
		// self.shared.state.store(state as u8, Ordering::SeqCst);
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

	pub fn shared(&self) -> Arc<Shared> {
		self.shared.clone()
	}

	fn position(&self) -> f64 {
		(self.current_frame as f64 + self.fractional_position) / self.sample_rate as f64
	}

	fn seek_to_index(&mut self, index: u64) {
		self.seek_destination_sender.store(index, Ordering::SeqCst);
	}

	fn seek_to(&mut self, position: f64) {
		self.seek_to_index((position * self.sample_rate as f64).round() as u64);
	}

	fn seek_by(&mut self, amount: f64) {
		self.seek_to(self.position() + amount);
	}
}

impl Sound for StreamingSound {
	fn track(&mut self) -> TrackId {
		TrackId::Main
	}

	fn on_start_processing(&mut self) {
		/* self.shared
		.position
		.store(self.position().to_bits(), Ordering::SeqCst); */
		while let Some(command) = self.command_consumer.pop() {
			match command {
				/* Command::SetVolume(volume) => self.volume.set(volume),
				Command::SetPlaybackRate(playback_rate) => self.playback_rate.set(playback_rate),
				Command::SetPanning(panning) => self.panning.set(panning),
				Command::Pause(tween) => self.pause(tween),
				Command::Resume(tween) => self.resume(tween),
				Command::Stop(tween) => self.stop(tween), */
				Command::SeekBy(amount) => self.seek_by(amount),
				Command::SeekTo(position) => self.seek_to(position),
				_ => todo!(),
			}
		}
	}

	fn process(&mut self, dt: f64, parameters: &Parameters, clocks: &Clocks) -> Frame {
		if matches!(self.state, PlaybackState::Paused | PlaybackState::Stopped) {
			return Frame::ZERO;
		}
		// pause playback while waiting for audio data. the first frame
		// in the ringbuffer is the previous frame, so we need to make
		// sure there's at least 2 before we continue playing.
		if self.frame_consumer.len() < 2 && !self.finished_signal_receiver.load(Ordering::SeqCst) {
			return Frame::ZERO;
		}
		self.update_current_frame();
		let next_frames = self.next_frames();
		let out = interpolate_frame(
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
			self.set_state(PlaybackState::Stopped);
		}
		out
	}

	fn finished(&self) -> bool {
		self.state == PlaybackState::Stopped
	}
}

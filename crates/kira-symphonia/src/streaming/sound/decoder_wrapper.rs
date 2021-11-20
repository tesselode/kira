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
use symphonia::core::{
	audio::{AudioBuffer, AudioBufferRef, Signal},
	codecs::Decoder,
	conv::{FromSample, IntoSample},
	formats::{FormatReader, SeekMode, SeekTo},
	sample::Sample,
};

use crate::{Error, StreamingSoundData};

use super::SEEK_DESTINATION_NONE;

const DECODER_THREAD_SLEEP_DURATION: Duration = Duration::from_millis(1);

pub struct DecoderWrapper {
	format_reader: Box<dyn FormatReader>,
	decoder: Box<dyn Decoder>,
	sample_rate: u32,
	track_id: u32,
	loop_behavior: Option<LoopBehavior>,
	frame_producer: Producer<(u64, Frame)>,
	seek_destination_receiver: Arc<AtomicU64>,
	stopped_signal_receiver: Arc<AtomicBool>,
	finished_signal_sender: Arc<AtomicBool>,
	decoded_frames: VecDeque<Frame>,
	current_frame: u64,
}

impl DecoderWrapper {
	pub fn new(
		data: StreamingSoundData,
		frame_producer: Producer<(u64, Frame)>,
		seek_destination_receiver: Arc<AtomicU64>,
		stopped_signal_receiver: Arc<AtomicBool>,
		finished_signal_sender: Arc<AtomicBool>,
	) -> Result<Self, Error> {
		let mut wrapper = Self {
			format_reader: data.format_reader,
			decoder: data.decoder,
			sample_rate: data.sample_rate,
			track_id: data.track_id,
			loop_behavior: data.settings.loop_behavior,
			frame_producer,
			seek_destination_receiver,
			stopped_signal_receiver,
			finished_signal_sender,
			decoded_frames: VecDeque::new(),
			current_frame: 0,
		};
		wrapper.seek(data.settings.start_position)?;
		Ok(wrapper)
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
		} else {
			let reached_end_of_file = self.decode()?;
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

	fn decode(&mut self) -> Result<bool, Error> {
		match self.format_reader.next_packet() {
			Ok(packet) => {
				let buffer = self.decoder.decode(&packet)?;
				load_frames_from_buffer_ref(&mut self.decoded_frames, &buffer)?;
			}
			Err(error) => match error {
				symphonia::core::errors::Error::IoError(error) => {
					if error.kind() == std::io::ErrorKind::UnexpectedEof {
						return Ok(true);
					}
					return Err(symphonia::core::errors::Error::IoError(error).into());
				}
				error => return Err(error.into()),
			},
		}
		Ok(false)
	}

	fn seek_to_index(&mut self, index: u64) -> Result<(), Error> {
		let seeked_to = self.format_reader.seek(
			SeekMode::Accurate,
			SeekTo::TimeStamp {
				ts: index,
				track_id: self.track_id,
			},
		)?;
		self.current_frame = seeked_to.actual_ts;
		Ok(())
	}

	fn seek(&mut self, position: f64) -> Result<(), Error> {
		let index = (position * self.sample_rate as f64).round() as u64;
		self.seek_to_index(index)?;
		Ok(())
	}
}

fn load_frames_from_buffer_ref(
	frames: &mut VecDeque<Frame>,
	buffer: &AudioBufferRef,
) -> Result<(), Error> {
	match buffer {
		AudioBufferRef::U8(buffer) => load_frames_from_buffer(frames, buffer),
		AudioBufferRef::U16(buffer) => load_frames_from_buffer(frames, buffer),
		AudioBufferRef::U24(buffer) => load_frames_from_buffer(frames, buffer),
		AudioBufferRef::U32(buffer) => load_frames_from_buffer(frames, buffer),
		AudioBufferRef::S8(buffer) => load_frames_from_buffer(frames, buffer),
		AudioBufferRef::S16(buffer) => load_frames_from_buffer(frames, buffer),
		AudioBufferRef::S24(buffer) => load_frames_from_buffer(frames, buffer),
		AudioBufferRef::S32(buffer) => load_frames_from_buffer(frames, buffer),
		AudioBufferRef::F32(buffer) => load_frames_from_buffer(frames, buffer),
		AudioBufferRef::F64(buffer) => load_frames_from_buffer(frames, buffer),
	}
}

fn load_frames_from_buffer<S: Sample>(
	frames: &mut VecDeque<Frame>,
	buffer: &AudioBuffer<S>,
) -> Result<(), Error>
where
	f32: FromSample<S>,
{
	match buffer.spec().channels.count() {
		1 => {
			for sample in buffer.chan(0) {
				frames.push_back(Frame::from_mono((*sample).into_sample()));
			}
		}
		2 => {
			for (left, right) in buffer.chan(0).iter().zip(buffer.chan(1).iter()) {
				frames.push_back(Frame::new((*left).into_sample(), (*right).into_sample()));
			}
		}
		_ => return Err(Error::UnsupportedChannelConfiguration),
	}
	Ok(())
}

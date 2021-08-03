use atomic_arena::{Arena, Controller};
use ringbuf::Producer;

use crate::{audio_stream::AudioStreamWrapper, manager::command::AudioStreamCommand};

use super::{mixer::Mixer, Parameters};

pub(crate) struct AudioStreams {
	audio_streams: Arena<AudioStreamWrapper>,
	unused_audio_stream_producer: Producer<AudioStreamWrapper>,
}

impl AudioStreams {
	pub fn new(
		capacity: usize,
		unused_audio_stream_producer: Producer<AudioStreamWrapper>,
	) -> Self {
		Self {
			audio_streams: Arena::new(capacity),
			unused_audio_stream_producer,
		}
	}

	pub fn controller(&self) -> Controller {
		self.audio_streams.controller()
	}

	pub fn on_start_processing(&mut self) {
		if self.unused_audio_stream_producer.is_full() {
			return;
		}
		for (_, audio_stream) in self
			.audio_streams
			.drain_filter(|audio_stream| audio_stream.shared().is_marked_for_removal())
		{
			if self
				.unused_audio_stream_producer
				.push(audio_stream)
				.is_err()
			{
				panic!("Unused audio_stream producer is full")
			}
			if self.unused_audio_stream_producer.is_full() {
				return;
			}
		}
	}

	pub fn run_command(&mut self, command: AudioStreamCommand) {
		match command {
			AudioStreamCommand::Add(id, audio_stream) => self
				.audio_streams
				.insert_with_index(id.0, audio_stream)
				.expect("AudioStream arena is full"),
		}
	}

	pub fn process(&mut self, dt: f64, parameters: &Parameters, mixer: &mut Mixer) {
		for (_, stream) in &mut self.audio_streams {
			stream.process(dt, parameters, mixer);
		}
	}
}

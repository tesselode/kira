//! Adds echoes to a sound.

mod builder;
mod handle;

pub use builder::*;
pub use handle::*;

use std::time::Duration;

use crate::{
	command::{read_commands_into_parameters, ValueChangeCommand},
	command_writers_and_readers,
	frame::Frame,
	info::Info,
	Decibels, Mix, Parameter,
};

use super::Effect;

struct Delay {
	command_readers: CommandReaders,
	delay_time: Duration,
	feedback: Parameter<Decibels>,
	mix: Parameter<Mix>,
	buffer: Vec<Frame>,
	feedback_effects: Vec<Box<dyn Effect>>,
	temp_buffer: Vec<Frame>,
}

impl Delay {
	/// Creates a new delay effect.
	#[must_use]
	fn new(builder: DelayBuilder, command_readers: CommandReaders) -> Self {
		Self {
			command_readers,
			delay_time: builder.delay_time,
			feedback: Parameter::new(builder.feedback, Decibels(-6.0)),
			mix: Parameter::new(builder.mix, Mix(0.5)),
			buffer: Vec::with_capacity(0),
			feedback_effects: builder.feedback_effects,
			temp_buffer: vec![],
		}
	}
}

impl Effect for Delay {
	fn init(&mut self, sample_rate: u32, internal_buffer_size: usize) {
		let delay_time_frames = (self.delay_time.as_secs_f64() * sample_rate as f64) as usize;
		self.buffer = vec![Frame::ZERO; delay_time_frames];
		self.temp_buffer = vec![Frame::ZERO; internal_buffer_size];
		for effect in &mut self.feedback_effects {
			effect.init(sample_rate, internal_buffer_size);
		}
	}

	fn on_change_sample_rate(&mut self, sample_rate: u32) {
		let delay_time_frames = (self.delay_time.as_secs_f64() * sample_rate as f64) as usize;
		self.buffer = vec![Frame::ZERO; delay_time_frames];
		for effect in &mut self.feedback_effects {
			effect.on_change_sample_rate(sample_rate);
		}
	}

	fn on_start_processing(&mut self) {
		read_commands_into_parameters!(self, feedback, mix);
		for effect in &mut self.feedback_effects {
			effect.on_start_processing();
		}
	}

	fn process(&mut self, input: &mut [Frame], dt: f64, info: &Info) {
		self.feedback.update(dt * input.len() as f64, info);
		self.mix.update(dt * input.len() as f64, info);

		for input in input.chunks_mut(self.buffer.len()) {
			let num_frames = input.len();

			// read from the beginning of the buffer and apply effects and feedback gain
			self.temp_buffer[..input.len()].copy_from_slice(&self.buffer[..input.len()]);
			for effect in &mut self.feedback_effects {
				effect.process(&mut self.temp_buffer[..input.len()], dt, info);
			}
			for (i, frame) in self.temp_buffer[..input.len()].iter_mut().enumerate() {
				let time_in_chunk = (i + 1) as f64 / num_frames as f64;
				let feedback = self.feedback.interpolated_value(time_in_chunk);
				*frame *= feedback.as_amplitude();
			}

			// write input + read buffer to the end of the buffer
			self.buffer.copy_within(input.len().., 0);
			let write_range = self.buffer.len() - input.len()..;
			for ((out, input), read) in self.buffer[write_range]
				.iter_mut()
				.zip(input.iter())
				.zip(&mut self.temp_buffer[..input.len()])
			{
				*out = *input + *read;
			}

			// output mix of input and read buffer
			for (i, frame) in input.iter_mut().enumerate() {
				let time_in_chunk = (i + 1) as f64 / num_frames as f64;
				let mix = self.mix.interpolated_value(time_in_chunk).0.clamp(0.0, 1.0);
				*frame = self.temp_buffer[i] * mix.sqrt() + *frame * (1.0 - mix).sqrt()
			}
		}
	}
}

command_writers_and_readers! {
	set_feedback: ValueChangeCommand<Decibels>,
	set_mix: ValueChangeCommand<Mix>,
}

//! Adds echoes to a sound.

mod builder;
mod handle;

pub use builder::*;
pub use handle::*;

use crate::{
	clock::clock_info::ClockInfoProvider,
	command::ValueChangeCommand,
	command_writers_and_readers,
	dsp::{interpolate_frame, Frame},
	modulator::value_provider::ModulatorValueProvider,
	read_commands_into_parameters,
	track::Effect,
	tween::Parameter,
	Volume,
};

#[derive(Debug, Clone)]
enum DelayState {
	Uninitialized {
		buffer_length: f64,
	},
	Initialized {
		buffer: Vec<Frame>,
		buffer_length: f64,
		write_position: usize,
	},
}

struct Delay {
	command_readers: CommandReaders,
	delay_time: Parameter,
	feedback: Parameter<Volume>,
	mix: Parameter,
	state: DelayState,
	feedback_effects: Vec<Box<dyn Effect>>,
}

impl Delay {
	/// Creates a new delay effect.
	fn new(builder: DelayBuilder, command_readers: CommandReaders) -> Self {
		Self {
			command_readers,
			delay_time: Parameter::new(builder.delay_time, 0.5),
			feedback: Parameter::new(builder.feedback, Volume::Amplitude(0.5)),
			mix: Parameter::new(builder.mix, 0.5),
			state: DelayState::Uninitialized {
				buffer_length: builder.buffer_length,
			},
			feedback_effects: builder.feedback_effects,
		}
	}
}

impl Effect for Delay {
	fn init(&mut self, sample_rate: u32) {
		if let DelayState::Uninitialized { buffer_length } = &self.state {
			self.state = DelayState::Initialized {
				buffer: vec![Frame::ZERO; (buffer_length * sample_rate as f64) as usize],
				buffer_length: *buffer_length,
				write_position: 0,
			};
			for effect in &mut self.feedback_effects {
				effect.init(sample_rate);
			}
		} else {
			panic!("The delay should be in the uninitialized state before init")
		}
	}

	fn on_change_sample_rate(&mut self, sample_rate: u32) {
		if let DelayState::Initialized {
			buffer,
			buffer_length,
			write_position,
		} = &mut self.state
		{
			*buffer = vec![Frame::ZERO; (*buffer_length * sample_rate as f64) as usize];
			*write_position = 0;
			for effect in &mut self.feedback_effects {
				effect.on_change_sample_rate(sample_rate);
			}
		} else {
			panic!("The delay should be initialized when the change sample rate callback is called")
		}
	}

	fn on_start_processing(&mut self) {
		read_commands_into_parameters!(self, delay_time, feedback, mix);
		for effect in &mut self.feedback_effects {
			effect.on_start_processing();
		}
	}

	fn process(
		&mut self,
		input: Frame,
		dt: f64,
		clock_info_provider: &ClockInfoProvider,
		modulator_value_provider: &ModulatorValueProvider,
	) -> Frame {
		if let DelayState::Initialized {
			buffer,
			write_position,
			..
		} = &mut self.state
		{
			self.delay_time
				.update(dt, clock_info_provider, modulator_value_provider);
			self.feedback
				.update(dt, clock_info_provider, modulator_value_provider);
			self.mix
				.update(dt, clock_info_provider, modulator_value_provider);

			// get the read position (in samples)
			let mut read_position = *write_position as f32 - (self.delay_time.value() / dt) as f32;
			while read_position < 0.0 {
				read_position += buffer.len() as f32;
			}

			// read an interpolated sample
			let current_sample_index = read_position as usize;
			let previous_sample_index = if current_sample_index == 0 {
				buffer.len() - 2
			} else {
				current_sample_index - 1
			};
			let next_sample_index = (current_sample_index + 1) % buffer.len();
			let next_sample_index_2 = (current_sample_index + 2) % buffer.len();
			let fraction = read_position % 1.0;
			let mut output = interpolate_frame(
				buffer[previous_sample_index],
				buffer[current_sample_index],
				buffer[next_sample_index],
				buffer[next_sample_index_2],
				fraction,
			);
			for effect in &mut self.feedback_effects {
				output = effect.process(output, dt, clock_info_provider, modulator_value_provider);
			}

			// write output audio to the buffer
			*write_position += 1;
			*write_position %= buffer.len();
			buffer[*write_position] = input + output * self.feedback.value().as_amplitude() as f32;

			let mix = self.mix.value() as f32;
			output * mix.sqrt() + input * (1.0 - mix).sqrt()
		} else {
			panic!("The delay should be initialized by the first process call")
		}
	}
}

command_writers_and_readers! {
	set_delay_time: ValueChangeCommand<f64>,
	set_feedback: ValueChangeCommand<Volume>,
	set_mix: ValueChangeCommand<f64>,
}

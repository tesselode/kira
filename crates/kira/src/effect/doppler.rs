//! Adds doppler effects to a sound.
//! Useful for simulating the sound of moving audio sources.

mod builder;
mod handle;

pub use builder::*;
pub use handle::*;

use crate::{
	command::{read_commands_into_parameters, ValueChangeCommand},
	command_writers_and_readers,
	effect::Effect,
	info::Info,
	Frame, Parameter,
};
use glam::Vec3;

/// Universal gas constant in J/(mol·K)
pub const R: f32 = 8.314;

#[derive(Debug, PartialEq)]
struct Motion {
	velocity: f32,
	direction: MotionDirection,
}

impl Motion {
	fn new(velocity: f32, approaching: bool) -> Self {
		Self {
			velocity,
			direction: if approaching {
				MotionDirection::Approaching
			} else {
				MotionDirection::Departing
			},
		}
	}
}

#[derive(Debug, PartialEq)]
enum MotionState {
	BothStationary,
	OnlyEmitterMoving(Motion),
	OnlyListenerMoving(Motion),
	BothMoving { emitter: Motion, listener: Motion },
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum MotionDirection {
	Approaching,
	Departing,
}

/// This code is based on LibreTexts by OpenStax, found here:
/// https://phys.libretexts.org/Bookshelves/University_Physics/University_Physics_(OpenStax)/Book%3A_University_Physics_I_-_Mechanics_Sound_Oscillations_and_Waves_(OpenStax)/17%3A_Sound/17.08%3A_The_Doppler_Effect
struct Doppler {
	command_readers: CommandReaders,
	temperature: Parameter,
	mass: Parameter,
	index: Parameter,
	state: Option<MotionState>,
}

impl Doppler {
	/// Creates a new `Doppler` effect.
	#[must_use]
	fn new(settings: DopplerBuilder, command_readers: CommandReaders) -> Self {
		Self {
			command_readers,
			temperature: Parameter::new(settings.temperature, 0.9),
			mass: Parameter::new(settings.mass, 0.1),
			index: Parameter::new(settings.index, 0.1),
			state: None,
		}
	}

	fn speed_of_sound(&self, time_in_chunk: f64) -> f32 {
		let temperature = self.temperature.interpolated_value(time_in_chunk) as f32;
		let mass = self.mass.interpolated_value(time_in_chunk) as f32;
		let index = self.index.interpolated_value(time_in_chunk) as f32;

		// Convert temperature from Celsius to Kelvin
		let kelvin = temperature + 273.15;

		// Calculate speed of sound
		(index * R * kelvin / mass).sqrt()
	}
}

impl Effect for Doppler {
	fn on_start_processing(&mut self) {
		read_commands_into_parameters!(self, temperature, mass);
	}

	fn process(&mut self, input: &mut [Frame], dt: f64, info: &Info) {
		self.temperature.update(dt * input.len() as f64, info);
		self.mass.update(dt * input.len() as f64, info);
		self.index.update(dt * input.len() as f64, info);

		if let (Some(listener), Some(spatial)) = (info.listener_info(), info.spatial_track_info()) {
			match (listener.stationary(), spatial.stationary()) {
				(true, true) => {
					self.state = Some(MotionState::BothStationary);
				}
				(false, false) => {
					let emitter_velocity = spatial.velocity();
					let listener_velocity = listener.velocity();
					let relative_position =
						Vec3::from(spatial.position) - Vec3::from(listener.position);
					let listener_motion = listener_velocity.dot(relative_position);
					let emitter_motion = emitter_velocity.dot(relative_position);
					let listener_approaching = listener_motion < 0.0;
					let emitter_approaching = emitter_motion < 0.0;
					self.state = Some(MotionState::BothMoving {
						emitter: Motion::new(emitter_velocity.length(), emitter_approaching),
						listener: Motion::new(listener_velocity.length(), listener_approaching),
					});
				}
				(true, false) => {
					let velocity = spatial.velocity();
					let relative_position =
						Vec3::from(spatial.position) - Vec3::from(listener.position);
					let motion = velocity.dot(relative_position);
					let approaching = motion < 0.0;
					self.state = Some(MotionState::OnlyEmitterMoving(Motion::new(
						velocity.length(),
						approaching,
					)));
				}
				(false, true) => {
					let velocity = listener.velocity();
					let relative_position =
						Vec3::from(spatial.position) - Vec3::from(listener.position);
					let motion = velocity.dot(relative_position);
					let approaching = motion < 0.0;
					self.state = Some(MotionState::OnlyListenerMoving(Motion::new(
						velocity.length(),
						approaching,
					)));
				}
			};

			if let Some(state) = self
				.state
				.as_ref()
				.filter(|x| **x != MotionState::BothStationary)
			{
				let num_frames = input.len();
				for (i, frame) in input.iter_mut().enumerate() {
					let time_in_chunk = (i + 1) as f64 / num_frames as f64;
					let speed_of_sound = self.speed_of_sound(time_in_chunk);

					let quotient = match state {
						MotionState::OnlyEmitterMoving(Motion {
							velocity,
							direction,
						}) => match direction {
							MotionDirection::Approaching => {
								speed_of_sound / (speed_of_sound - velocity)
							}
							MotionDirection::Departing => {
								speed_of_sound / (speed_of_sound + velocity)
							}
						},
						MotionState::OnlyListenerMoving(Motion {
							velocity,
							direction,
						}) => match direction {
							MotionDirection::Approaching => {
								(speed_of_sound + velocity) / speed_of_sound
							}
							MotionDirection::Departing => {
								(speed_of_sound - velocity) / speed_of_sound
							}
						},
						MotionState::BothMoving { emitter, listener } => {
							match (emitter.direction, listener.direction) {
								(MotionDirection::Approaching, MotionDirection::Approaching) => {
									(speed_of_sound + listener.velocity)
										/ (speed_of_sound - emitter.velocity)
								}
								(MotionDirection::Departing, MotionDirection::Departing) => {
									(speed_of_sound - listener.velocity)
										/ (speed_of_sound + emitter.velocity)
								}
								(MotionDirection::Approaching, MotionDirection::Departing) => {
									(speed_of_sound - listener.velocity)
										/ (speed_of_sound - emitter.velocity)
								}
								(MotionDirection::Departing, MotionDirection::Approaching) => {
									(speed_of_sound + listener.velocity)
										/ (speed_of_sound + emitter.velocity)
								}
							}
						}
						MotionState::BothStationary => unreachable!(),
					};

					*frame = Frame::new(frame.left * quotient, frame.right * quotient);
				}
			}
		}
	}
}

command_writers_and_readers! {
	set_temperature: ValueChangeCommand<f64>,
	set_mass: ValueChangeCommand<f64>,
	set_index: ValueChangeCommand<f64>,
}

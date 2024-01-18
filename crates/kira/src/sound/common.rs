use crate::{
	tween::{Tween, Value},
	OutputDestination, StartTime, Volume,
};

pub struct CommonSoundController {}

pub struct CommonSoundSettings {
	/// When the sound should start playing.
	pub start_time: StartTime,
	/// The volume of the sound.
	pub volume: Value<Volume>,
	/// The panning of the sound, where 0 is hard left
	/// and 1 is hard right.
	pub panning: Value<f64>,
	/// The destination that this sound should be routed to.
	pub output_destination: OutputDestination,
	/// An optional fade-in from silence.
	pub fade_in_tween: Option<Tween>,
}

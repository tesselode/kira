/// Describes how a sound should be looped.
///
/// The end of the loop is always at the end of the sound.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct LoopBehavior {
	/// The position that playback should jump to when the
	/// end of the sound has been reached.
	pub start_position: f64,
}

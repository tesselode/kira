/// Useful info about a `Sound`.
///
/// This is set entirely by the user when loading a sound
/// and can be accessed via `SoundId`s.
#[derive(Debug, Default, Copy, Clone)]
pub struct SoundMetadata {
	/// How long the sound is musically.
	///
	/// For example, a recording of a 2-bar drum fill
	/// in an echoey cathedral may have 5 seconds of actual
	/// drumming and then 10 seconds of reverberations from
	/// the building. So even though the audio is 15 seconds
	/// long, you might say the music only lasts for 5 seconds.
	///
	/// If set, the semantic duration of the sound will be
	/// used as the default end point when looping the sound.
	pub semantic_duration: Option<f64>,
}

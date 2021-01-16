use thiserror::Error;

#[derive(Debug, Error)]
pub enum SoundFromFileError {
	#[error("Only mono and stereo audio is supported")]
	UnsupportedChannelConfiguration,

	#[error("Only .mp3, .ogg, .flac, and .wav files are supported")]
	UnsupportedAudioFileFormat,

	#[error("{0}")]
	IoError(#[from] std::io::Error),

	#[cfg(feature = "mp3")]
	#[error("{0}")]
	Mp3Error(#[from] minimp3::Error),

	#[cfg(feature = "mp3")]
	#[error("mp3s with variable sample rates are not supported")]
	VariableMp3SampleRate,

	#[cfg(feature = "mp3")]
	#[error("Could not get the sample rate of the mp3")]
	UnknownMp3SampleRate,

	#[cfg(feature = "ogg")]
	#[error("{0}")]
	OggError(#[from] lewton::VorbisError),

	#[cfg(feature = "flac")]
	#[error("{0}")]
	FlacError(#[from] claxon::Error),

	#[cfg(feature = "wav")]
	#[error("{0}")]
	WavError(#[from] hound::Error),
}

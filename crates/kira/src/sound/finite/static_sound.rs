#[cfg(test)]
mod test;

use std::{
	fmt::{Debug, Formatter},
	sync::Arc,
	time::Duration,
};

use crate::{
	dsp::Frame,
	sound::{Sound, SoundData},
};

use super::{FiniteSound, FiniteSoundData, SoundHandle, SoundSettings};

/// A piece of audio loaded into memory all at once.
///
/// These can be cheaply cloned, as the audio data is shared
/// among all clones.
#[derive(Clone, PartialEq)]
pub struct StaticSoundData {
	/// The sample rate of the audio (in Hz).
	pub sample_rate: u32,
	/// The raw samples that make up the audio.
	pub frames: Arc<[Frame]>,
	/// Settings for the sound.
	pub settings: SoundSettings,
}

impl StaticSoundData {
	#[cfg(feature = "symphonia")]
	pub fn from_cursor<T: AsRef<[u8]> + Send + Sync + 'static>(
		cursor: std::io::Cursor<T>,
		settings: SoundSettings,
	) -> Result<StaticSoundData, super::error::LoadError> {
		Self::from_media_source(Box::new(cursor), settings)
	}

	#[cfg(all(feature = "symphonia", not(target_arch = "wasm32")))]
	pub fn from_file(
		path: impl AsRef<std::path::Path>,
		settings: SoundSettings,
	) -> Result<Self, super::error::FromFileError> {
		Ok(Self::from_media_source(
			Box::new(std::fs::File::open(path)?),
			settings,
		)?)
	}

	/// Returns the duration of the audio.
	pub fn duration(&self) -> Duration {
		Duration::from_secs_f64(self.frames.len() as f64 / self.sample_rate as f64)
	}

	/// Returns a clone of the `StaticSoundData` with the specified settings.
	pub fn with_settings(&self, settings: SoundSettings) -> Self {
		Self {
			settings,
			..self.clone()
		}
	}

	/// Returns a clone of the `StaticSoundData` with the modified settings from
	/// the given function.
	pub fn with_modified_settings(&self, f: impl FnOnce(SoundSettings) -> SoundSettings) -> Self {
		self.with_settings(f(self.settings))
	}

	#[cfg(feature = "symphonia")]
	fn from_media_source(
		media_source: Box<dyn symphonia::core::io::MediaSource>,
		settings: SoundSettings,
	) -> Result<Self, super::error::LoadError> {
		use symphonia::core::io::MediaSourceStream;

		use crate::sound::finite::{error::LoadError, symphonia::load_frames_from_buffer_ref};

		let codecs = symphonia::default::get_codecs();
		let probe = symphonia::default::get_probe();
		let mss = MediaSourceStream::new(media_source, Default::default());
		let mut format_reader = probe
			.format(
				&Default::default(),
				mss,
				&Default::default(),
				&Default::default(),
			)?
			.format;
		let codec_params = &format_reader
			.default_track()
			.ok_or(LoadError::NoDefaultTrack)?
			.codec_params;
		let sample_rate = codec_params
			.sample_rate
			.ok_or(LoadError::UnknownSampleRate)?;
		let mut decoder = codecs.make(codec_params, &Default::default())?;
		let mut frames = vec![];
		loop {
			match format_reader.next_packet() {
				Ok(packet) => {
					let buffer = decoder.decode(&packet)?;
					load_frames_from_buffer_ref(&mut frames, &buffer)?;
				}
				Err(error) => match error {
					symphonia::core::errors::Error::IoError(error) => {
						if error.kind() == std::io::ErrorKind::UnexpectedEof {
							break;
						}
						return Err(symphonia::core::errors::Error::IoError(error).into());
					}
					error => return Err(error.into()),
				},
			}
		}
		Ok(Self {
			sample_rate,
			frames: frames.into(),
			settings,
		})
	}

	pub(super) fn split(self) -> (FiniteSound, SoundHandle) {
		/* let (command_producer, command_consumer) = HeapRb::new(COMMAND_BUFFER_CAPACITY).split(); */
		let sound = FiniteSound::new(Box::new(self) /* command_consumer */);
		/* let shared = sound.shared(); */
		(
			sound,
			SoundHandle, /*  {
							 command_producer,
							 shared,
						 } */
		)
	}
}

impl SoundData for StaticSoundData {
	type Error = ();

	type Handle = SoundHandle;

	#[allow(clippy::type_complexity)]
	fn into_sound(self) -> Result<(Box<dyn Sound>, Self::Handle), Self::Error> {
		let (sound, handle) = self.split();
		Ok((Box::new(sound), handle))
	}
}

impl FiniteSoundData for StaticSoundData {
	fn sample_rate(&mut self) -> u32 {
		self.sample_rate
	}

	fn len(&mut self) -> usize {
		self.frames.len()
	}

	fn frame(&mut self, index: usize) -> Frame {
		self.frames[index]
	}

	fn buffer_len(&mut self) -> usize {
		4
	}
}

impl Debug for StaticSoundData {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("StaticSoundData")
			.field("sample_rate", &self.sample_rate)
			.field(
				"frames",
				&FramesDebug {
					len: self.frames.len(),
				},
			)
			.field("settings", &self.settings)
			.finish()
	}
}

struct FramesDebug {
	len: usize,
}

impl Debug for FramesDebug {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("[{} frames]", self.len))
	}
}

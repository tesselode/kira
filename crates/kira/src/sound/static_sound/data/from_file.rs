use std::io::Cursor;

use symphonia::core::io::{MediaSource, MediaSourceStream};

use crate::sound::{
	static_sound::StaticSoundSettings, symphonia::load_frames_from_buffer_ref, FromFileError,
};

use super::StaticSoundData;

impl StaticSoundData {
	/// Loads an audio file into a [`StaticSoundData`].
	#[cfg(not(target_arch = "wasm32"))]
	#[cfg_attr(docsrs, doc(cfg(all(feature = "symphonia", not(wasm32)))))]
	pub fn from_file(path: impl AsRef<std::path::Path>) -> Result<Self, FromFileError> {
		Self::from_media_source(std::fs::File::open(path)?)
	}

	/// Loads a cursor wrapping audio file data into a [`StaticSoundData`].
	#[cfg_attr(docsrs, doc(cfg(feature = "symphonia")))]
	pub fn from_cursor<T: AsRef<[u8]> + Send + Sync + 'static>(
		cursor: Cursor<T>,
	) -> Result<StaticSoundData, FromFileError> {
		Self::from_media_source(cursor)
	}

	/// Loads an audio file from a type that implements Symphonia's [`MediaSource`]
	/// trait.
	#[cfg_attr(docsrs, doc(cfg(feature = "symphonia")))]
	pub fn from_media_source(
		media_source: impl MediaSource + 'static,
	) -> Result<Self, FromFileError> {
		Self::from_boxed_media_source(Box::new(media_source))
	}

	fn from_boxed_media_source(media_source: Box<dyn MediaSource>) -> Result<Self, FromFileError> {
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
		let default_track = format_reader
			.default_track()
			.ok_or(FromFileError::NoDefaultTrack)?;
		let default_track_id = default_track.id;
		let codec_params = &default_track.codec_params;
		let sample_rate = codec_params
			.sample_rate
			.ok_or(FromFileError::UnknownSampleRate)?;
		let mut decoder = codecs.make(codec_params, &Default::default())?;
		let mut frames = vec![];
		loop {
			match format_reader.next_packet() {
				Ok(packet) => {
					if default_track_id == packet.track_id() {
						let buffer = decoder.decode(&packet)?;
						frames.append(&mut load_frames_from_buffer_ref(&buffer)?);
					}
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
			settings: StaticSoundSettings::default(),
			slice: None,
		})
	}
}

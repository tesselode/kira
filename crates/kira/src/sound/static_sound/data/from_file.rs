use std::io::Cursor;

use symphonia::core::io::{MediaSource, MediaSourceStream};
use symphonia::core::{codecs::CodecParameters, formats::TrackType};

use crate::sound::{
	FromFileError, static_sound::StaticSoundSettings, symphonia::load_frames_from_buffer_ref,
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
		let mut format_reader = probe.probe(
			&Default::default(),
			mss,
			Default::default(),
			Default::default(),
		)?;
		let default_track = format_reader
			.default_track(TrackType::Audio)
			.ok_or(FromFileError::NoDefaultTrack)?;
		let default_track_id = default_track.id;
		let audio_params = match default_track.codec_params.as_ref() {
			Some(CodecParameters::Audio(p)) => p,
			_ => return Err(FromFileError::NoDefaultTrack),
		};
		let sample_rate = audio_params
			.sample_rate
			.ok_or(FromFileError::UnknownSampleRate)?;
		let mut decoder = codecs.make_audio_decoder(audio_params, &Default::default())?;
		let mut frames = vec![];
		loop {
			match format_reader.next_packet() {
				Ok(Some(packet)) => {
					if default_track_id == packet.track_id {
						let buffer = decoder.decode(&packet)?;
						frames.append(&mut load_frames_from_buffer_ref(&buffer)?);
					}
				}
				Ok(None) => break,
				Err(error) => return Err(error.into()),
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

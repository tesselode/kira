/*!
# kira-loaders

Provides support for loading and streaming sounds from audio files
in Kira.

## Examples

### Loading a sound into memory all at once

```no_run
use kira::{
	manager::{backend::mock::MockBackend, AudioManager, AudioManagerSettings},
	sound::static_sound::StaticSoundSettings,
};

let mut manager = AudioManager::<MockBackend>::new(AudioManagerSettings::default()).unwrap();
manager.play(kira_loaders::load(
	"sound.ogg",
	StaticSoundSettings::default(),
)?)?;
# Result::<(), Box<dyn std::error::Error>>::Ok(())
```

### Streaming a sound from disk

```no_run
use kira::manager::{backend::mock::MockBackend, AudioManager, AudioManagerSettings};
use kira_loaders::StreamingSoundSettings;

let mut manager = AudioManager::<MockBackend>::new(AudioManagerSettings::default()).unwrap();
manager.play(kira_loaders::stream(
	"sound.ogg",
	StreamingSoundSettings::default(),
)?)?;
# Result::<(), Box<dyn std::error::Error>>::Ok(())
```

## Static vs. streaming sounds

`kira-loaders` can load entire sounds into memory, but it can also
stream them from the disk in real-time. This reduces the amount of
memory needed to play large audio files.

The [`stream`] function takes a [`StreamingSoundSettings`] argument,
which is almost the same as [`StaticSoundSettings`]. Similarly,
[`StreamingSoundHandle`]s are very similar to
[`StaticSoundHandle`](kira::sound::static_sound::StaticSoundHandle)s.

Streaming sounds have some disadvantages compared to static sounds:

- Streaming sounds require more CPU power.
- There may be a longer delay between when you call
  [`AudioManager::play`](kira::manager::AudioManager::play) and
  when the sound actually starts playing.
- Seeking the sound may also have a longer delay.
- If the file cannot be read from the disk fast enough, there will be hiccups in
  the sound playback. (This will not affect other sounds, though.)
- Backwards playback is not supported.
- [`StreamingSoundData`] cannot be cloned.
*/

#![warn(missing_docs)]
#![allow(clippy::tabs_in_doc_comments)]

mod error;
mod streaming;

use std::{fs::File, io::Cursor, path::Path, sync::Arc};

pub use error::*;
use kira::{
	dsp::Frame,
	sound::static_sound::{StaticSoundData, StaticSoundSettings},
};
use streaming::decoder::symphonia::SymphoniaDecoder;
pub use streaming::*;
use symphonia::core::{
	audio::{AudioBuffer, AudioBufferRef, Signal},
	conv::{FromSample, IntoSample},
	io::{MediaSource, MediaSourceStream},
	sample::Sample,
};

fn load_from_media_source(
	media_source: Box<dyn MediaSource>,
	settings: StaticSoundSettings,
) -> Result<StaticSoundData, Error> {
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
		.ok_or(Error::NoDefaultTrack)?
		.codec_params;
	let sample_rate = codec_params.sample_rate.ok_or(Error::UnknownSampleRate)?;
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
	Ok(StaticSoundData {
		sample_rate,
		frames: Arc::new(frames),
		settings,
	})
}

/// Loads an audio file into a [`StaticSoundData`].
pub fn load(
	path: impl AsRef<Path>,
	settings: StaticSoundSettings,
) -> Result<StaticSoundData, Error> {
	load_from_media_source(Box::new(File::open(path)?), settings)
}

/// Loads a cursor wrapping audio file data into a [`StaticSoundData`].
pub fn load_from_cursor<T: AsRef<[u8]> + Send + 'static>(
	cursor: Cursor<T>,
	settings: StaticSoundSettings,
) -> Result<StaticSoundData, Error> {
	load_from_media_source(Box::new(cursor), settings)
}

/// Creates a [`StreamingSoundData`] for an audio file.
pub fn stream(
	path: impl AsRef<Path>,
	settings: StreamingSoundSettings,
) -> Result<StreamingSoundData<Error>, Error> {
	Ok(StreamingSoundData {
		decoder: Box::new(SymphoniaDecoder::new(Box::new(File::open(path)?))?),
		settings,
	})
}

/// Creates a [`StreamingSoundData`] for a cursor wrapping audio file data.
pub fn stream_from_cursor<T: AsRef<[u8]> + Send + 'static>(
	cursor: Cursor<T>,
	settings: StreamingSoundSettings,
) -> Result<StreamingSoundData<Error>, Error> {
	Ok(StreamingSoundData {
		decoder: Box::new(SymphoniaDecoder::new(Box::new(cursor))?),
		settings,
	})
}

fn load_frames_from_buffer_ref(
	frames: &mut Vec<Frame>,
	buffer: &AudioBufferRef,
) -> Result<(), Error> {
	match buffer {
		AudioBufferRef::U8(buffer) => load_frames_from_buffer(frames, buffer),
		AudioBufferRef::U16(buffer) => load_frames_from_buffer(frames, buffer),
		AudioBufferRef::U24(buffer) => load_frames_from_buffer(frames, buffer),
		AudioBufferRef::U32(buffer) => load_frames_from_buffer(frames, buffer),
		AudioBufferRef::S8(buffer) => load_frames_from_buffer(frames, buffer),
		AudioBufferRef::S16(buffer) => load_frames_from_buffer(frames, buffer),
		AudioBufferRef::S24(buffer) => load_frames_from_buffer(frames, buffer),
		AudioBufferRef::S32(buffer) => load_frames_from_buffer(frames, buffer),
		AudioBufferRef::F32(buffer) => load_frames_from_buffer(frames, buffer),
		AudioBufferRef::F64(buffer) => load_frames_from_buffer(frames, buffer),
	}
}

fn load_frames_from_buffer<S: Sample>(
	frames: &mut Vec<Frame>,
	buffer: &AudioBuffer<S>,
) -> Result<(), Error>
where
	f32: FromSample<S>,
{
	match buffer.spec().channels.count() {
		1 => {
			for sample in buffer.chan(0) {
				frames.push(Frame::from_mono((*sample).into_sample()));
			}
		}
		2 => {
			for (left, right) in buffer.chan(0).iter().zip(buffer.chan(1).iter()) {
				frames.push(Frame::new((*left).into_sample(), (*right).into_sample()));
			}
		}
		_ => return Err(Error::UnsupportedChannelConfiguration),
	}
	Ok(())
}

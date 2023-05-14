/*!
Decodes data gradually from an audio file.

To play a streaming sound, pass a [`StreamingSoundData`] to
[`AudioManager::play`](crate::manager::AudioManager::play).

```no_run
use kira::{
	manager::{
		AudioManager, AudioManagerSettings,
		backend::cpal::CpalBackend,
	},
	sound::streaming::{StreamingSoundData, StreamingSoundSettings},
};

let mut manager = AudioManager::<CpalBackend>::new(AudioManagerSettings::default())?;
let sound_data = StreamingSoundData::from_file("sound.ogg", StreamingSoundSettings::default())?;
manager.play(sound_data)?;
# Result::<(), Box<dyn std::error::Error>>::Ok(())
```

Streaming sounds use less memory than static sounds, but they use more
CPU, and they can have delays when starting or seeking.
*/

#![cfg_attr(docsrs, doc(cfg(not(wasm32))))]

mod data;
mod decoder;
mod handle;
mod settings;
mod sound;

pub use data::*;
pub use decoder::*;
pub use handle::*;
pub use settings::*;

use crate::{
	tween::{Tween, Value},
	PlaybackRate, Volume,
};

use super::Region;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum SoundCommand {
	SetVolume(Value<Volume>, Tween),
	SetPlaybackRate(Value<PlaybackRate>, Tween),
	SetPanning(Value<f64>, Tween),
	Pause(Tween),
	Resume(Tween),
	Stop(Tween),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum DecodeSchedulerCommand {
	SetPlaybackRegion(Region),
	SetLoopRegion(Option<Region>),
	SeekBy(f64),
	SeekTo(f64),
}

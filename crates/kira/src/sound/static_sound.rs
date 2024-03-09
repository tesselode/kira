/*!
Playable chunks of audio that are loaded into memory all at once.

To play a static sound, pass a [`StaticSoundData`] to
[`AudioManager::play`](crate::manager::AudioManager::play).

```no_run
use kira::{
	manager::{
		AudioManager, AudioManagerSettings,
		backend::DefaultBackend,
	},
	sound::static_sound::{StaticSoundData, StaticSoundSettings},
};

let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
let sound_data = StaticSoundData::from_file("sound.ogg", StaticSoundSettings::default())?;
manager.play(sound_data)?;
# Result::<(), Box<dyn std::error::Error>>::Ok(())
```

Compared to streaming sounds, static sounds have lower CPU usage and shorter delays
when starting and seeking, but they use a lot more memory.
*/

mod data;
mod handle;
mod settings;
mod sound;

pub use data::*;
pub use handle::*;
pub use settings::*;

use crate::{command::ValueChangeCommand, command_writers_and_readers};

use super::{PlaybackRate, Region};

#[derive(Debug, Clone, Copy, PartialEq)]
struct SetLoopRegionCommand(Option<Region>);

#[derive(Debug, Clone, Copy, PartialEq)]
enum SeekCommand {
	By(f64),
	To(f64),
}

command_writers_and_readers!(
	playback_rate_change: ValueChangeCommand<PlaybackRate>,
	set_loop_region: SetLoopRegionCommand,
	seek: SeekCommand
);

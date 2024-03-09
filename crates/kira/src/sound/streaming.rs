/*!
Decodes data gradually from an audio file.

To play a streaming sound, pass a [`StreamingSoundData`] to
[`AudioManager::play`](crate::manager::AudioManager::play).

```no_run
use kira::{
	manager::{
		AudioManager, AudioManagerSettings,
		backend::DefaultBackend,
	},
	sound::streaming::{StreamingSoundData, StreamingSoundSettings},
};

let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
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

use crate::command::{command_writer_and_reader, CommandReader, CommandWriter, ValueChangeCommand};

use super::{PlaybackRate, Region};

#[derive(Debug, Clone, Copy, PartialEq)]
struct SetLoopRegionCommand(Option<Region>);

#[derive(Debug, Clone, Copy, PartialEq)]
enum SeekCommand {
	By(f64),
	To(f64),
}

pub(crate) struct CommandWriters {
	playback_rate_change: CommandWriter<ValueChangeCommand<PlaybackRate>>,
	set_loop_region: CommandWriter<SetLoopRegionCommand>,
	seek: CommandWriter<SeekCommand>,
}

pub(crate) struct SoundCommandReaders {
	playback_rate_change: CommandReader<ValueChangeCommand<PlaybackRate>>,
}

pub(crate) struct DecodeSchedulerCommandReaders {
	set_loop_region: CommandReader<SetLoopRegionCommand>,
	seek: CommandReader<SeekCommand>,
}

fn command_writers_and_readers() -> (
	CommandWriters,
	SoundCommandReaders,
	DecodeSchedulerCommandReaders,
) {
	let (playback_rate_change_writer, playback_rate_change_reader) = command_writer_and_reader();
	let (set_loop_region_writer, set_loop_region_reader) = command_writer_and_reader();
	let (seek_writer, seek_reader) = command_writer_and_reader();
	(
		CommandWriters {
			playback_rate_change: playback_rate_change_writer,
			set_loop_region: set_loop_region_writer,
			seek: seek_writer,
		},
		SoundCommandReaders {
			playback_rate_change: playback_rate_change_reader,
		},
		DecodeSchedulerCommandReaders {
			set_loop_region: set_loop_region_reader,
			seek: seek_reader,
		},
	)
}

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
let sound_data = StreamingSoundData::from_file("sound.ogg")?;
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
	command::{command_writer_and_reader, CommandReader, CommandWriter, ValueChangeCommand},
	tween::Tween,
	Dbfs, Panning, StartTime,
};

use super::{PlaybackRate, Region};

#[derive(Debug)]
pub(crate) struct CommandWriters {
	set_volume: CommandWriter<ValueChangeCommand<Dbfs>>,
	set_playback_rate: CommandWriter<ValueChangeCommand<PlaybackRate>>,
	set_panning: CommandWriter<ValueChangeCommand<Panning>>,
	set_loop_region: CommandWriter<Option<Region>>,
	pause: CommandWriter<Tween>,
	resume: CommandWriter<(StartTime, Tween)>,
	stop: CommandWriter<Tween>,
	seek_by: CommandWriter<f64>,
	seek_to: CommandWriter<f64>,
}

pub(crate) struct CommandReaders {
	set_volume: CommandReader<ValueChangeCommand<Dbfs>>,
	set_playback_rate: CommandReader<ValueChangeCommand<PlaybackRate>>,
	set_panning: CommandReader<ValueChangeCommand<Panning>>,
	pause: CommandReader<Tween>,
	resume: CommandReader<(StartTime, Tween)>,
	stop: CommandReader<Tween>,
}

#[derive(Debug)]
pub(crate) struct DecodeSchedulerCommandReaders {
	set_loop_region: CommandReader<Option<Region>>,
	seek_by: CommandReader<f64>,
	seek_to: CommandReader<f64>,
}

#[must_use]
fn command_writers_and_readers() -> (
	CommandWriters,
	CommandReaders,
	DecodeSchedulerCommandReaders,
) {
	let (set_volume_writer, set_volume_reader) = command_writer_and_reader();
	let (set_playback_rate_writer, set_playback_rate_reader) = command_writer_and_reader();
	let (set_panning_writer, set_panning_reader) = command_writer_and_reader();
	let (set_loop_region_writer, set_loop_region_reader) = command_writer_and_reader();
	let (pause_writer, pause_reader) = command_writer_and_reader();
	let (resume_writer, resume_reader) = command_writer_and_reader();
	let (stop_writer, stop_reader) = command_writer_and_reader();
	let (seek_by_writer, seek_by_reader) = command_writer_and_reader();
	let (seek_to_writer, seek_to_reader) = command_writer_and_reader();
	(
		CommandWriters {
			set_volume: set_volume_writer,
			set_playback_rate: set_playback_rate_writer,
			set_panning: set_panning_writer,
			set_loop_region: set_loop_region_writer,
			pause: pause_writer,
			resume: resume_writer,
			stop: stop_writer,
			seek_by: seek_by_writer,
			seek_to: seek_to_writer,
		},
		CommandReaders {
			set_volume: set_volume_reader,
			set_playback_rate: set_playback_rate_reader,
			set_panning: set_panning_reader,
			pause: pause_reader,
			resume: resume_reader,
			stop: stop_reader,
		},
		DecodeSchedulerCommandReaders {
			set_loop_region: set_loop_region_reader,
			seek_by: seek_by_reader,
			seek_to: seek_to_reader,
		},
	)
}

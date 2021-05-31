# v0.5.3 - May 31, 2021
- Fix an issue where the `AudioManager` cleanup would fail if
there are existing track handles

# v0.5.2 - April 27, 2021
- Added `Sound::from_mp3_reader`, `Sound::from_ogg_reader`,
`Sound::from_flac_reader`, and `Sound::from_wav_reader` (thanks
@Zicklag!)
- (Hopefully) fixed an issue where capacities (the maximum number
of instances, sequences, etc.) could decrease over time

# v0.5.1 - March 28, 2021
- Added a `Default` implementation for `TrackSends`
- Fixed a compile error when building Kira with the `serde_support`
feature enabled

# v0.5.0 - March 7, 2021

## Additions
- Added send tracks, which you can route sub-tracks to in
addition to their parent track. This is useful for sharing
effects between multiple sub-tracks.
- Made the `volume` setting for `Track`s a `Value<f64>`,
which means you can link it to a parameter
- Added `Main`/`Sub`/`SendTrackHandle::set_volume`
- Added an `init` callback to the `Effect` trait
- Added new effects: `Distortion`, `Delay`, and `Reverb`
- Added a `mix` setting for effects, which lets you blend
dry and wet signal
- Added `InstanceHandle::position`, which gets the current
playback position of the instance
- Added `CachedValue::with_valid_range` for clamping values

## Bugfixes
- The default filter cutoff is no longer outside of the range
of human hearing

## Other changes
- Changed settings structs with an `id` field to use `Option<Id>`.
This eliminates a confusing situation where cloning the same
settings struct and passing it to multiple objects results
in each object overwriting the previous one since it has the same
ID.
- Changes to errors related to sending commands to the audio thread:
  - Command-sending related errors are now listed in the `CommandError` enum
  - `BackendDisconnected` is no longer included in error enums
  - Handle structs whose only error variant was `BackendDisconnected` now
  return a `CommandError`
  - `TrackHandleError` was split into `AddEffectError` and `RemoveEffectError`
- `MetronomeHandle.event_iter` has been replaced with `MetronomeHandle.pop_event`,
which works the same way as `SequenceInstanceHandle.pop_event`
- Changed all occurrences of the term "pitch" to "playback rate"

# v0.4.1 - January 23, 2021
Added serde support for `Arrangement`s and `SoundClip`s

# v0.4.0 - January 23, 2021

## `wasm32` support
Kira now runs on any platform supporting `wasm32` and having a
`cpal` backend. This means one can now run an instance of Kira
in a web browser.

## Handle-based API
The API has gone through a major revision. Previously, to do
just about anything, you would have to use the `AudioManager`:

```rust
audio_manager.stop_instance(instance_id)?;
audio_manager.set_metronome_tempo(Tempo(128.0))?;
// etc...
```

This meant that you had to pass a reference to the audio manager
to every part of the code that needs it. It also meant that
the `AudioManager` struct had an overwhelming number of methods.

The API has been changed so that whenever you create a new thing
(like a sound, an instance, or a mixer track), you receive a handle
to that thing that you can use to control it.

```rust
let mut sound = audio_manager.load_sound("sound.ogg", SoundSettings::default())?;
let mut instance = sound.play(InstanceSettings::default())?;
instance.set_pitch(2.0)?;
```

## Multiple metronomes
You can now create multiple metronomes. Each sequence instance
can be assigned to a different metronome, and interval events
can be received from a `MetronomeHandle`:

```rust
let mut metronome = audio_manager.add_metronome(
	MetronomeSettings::new().interval_events_to_emit([0.25, 0.5, 1.0]))?;
audio_manager.start_sequence({
	let mut sequence = Sequence::<()>::new(SequenceSettings::default());
	// sequence code
	sequence
}, SequenceInstanceSettings::new().metronome(&metronome))?;
metronome.start()?;
for interval in metronome.event_iter() {
	println!("{}", interval);
}
```

Most people will only need one metronome at a time - the main
point of this is to move more functionality out of the
`AudioManager` struct.

## Serde support
Sequences and most config structs now have serialization/deserialization
support via the `serde_support` feature.

## Improved error handling
The capacity limits specified in `AudioManagerSettings` are now enforced,
and the audio manager will also check that when you remove something with
an ID, something with that ID actually exists. Because this creates a lot
of new error variants, the large `AudioError` enum has been split up into
smaller, situational error enums.

# v0.3.0 - December 26th, 2020

## Per-sequence custom event types
Previously, custom events emitted by sequences were retrieved
by calling `AudioManager::pop_event`, which meant that the
audio manager had a generic type parameter for custom events,
and every sequence had to emit custom events of the same type.

Now, each sequence has its own custom event type, and you receive
those events from an `EventReceiver` that the audio manager
returns when you call `AudioManager::add_sequence`. This gives
you more flexibility in how you organize your custom events
as well as moving some functionality out of the `AudioManager`
struct, which already has a lot of methods.

## Audio streams
Audio streams provide a way of sending arbitrary audio data
to the mixer. Sometimes, you need to play audio that you
generate in real time, like voice chats. This feature
lets you do that.

## Groups
Sounds, arrangements, and sequences can now be grouped so that
they can all be paused, resumed, or stopped at once. This is
useful for controlling broad categories of sounds, like gameplay
sounds or music.

## Other changes
- Added `Sequence::play_random`
- Added `Value::Random`
- Renamed `Sound::new` to `Sound::from_frames`
- Audio file format decoding is now gated behind feature flags
- Changed functions for pausing, resuming, and stopping instances
to take settings structs (`PauseInstanceSettings`,
`ResumeInstanceSettings`, and `StopInstanceSettings`)
- When resuming an instance, you can now choose to have it seek
backward to the time when it was paused. This is useful if you
need to keep audio synced up with something in-game, but you
still want a smooth fade out when pausing the game.
- Renamed `Sequence::emit_custom_event` to `Sequence::emit`
- Added `AudioManager::seek_instance` and `AudioManager::seek_instance_to`
for changing the playback positions of running instances
- Refined the behavior of looping backwards instances
- Added `Tween::linear`
- Make `Arrangement::settings` private and add settings parameters
to `Arrangement::new_loop` and `Arrangement::new_loop_with_intro`

# v0.2.0 - December 6th, 2020

## Arrangements
This release adds `Arrangement`s, which allow you to stitch together
multiple sounds into a larger sound that you can play instances of.

The main use case for this is setting up seamless loops, including
looping songs with intros. The previous release could create seamless
loops, but only if the sound didn't have an intro.

Functions that dealt with instances of sounds now deal with instances
of sounds *or* arrangements. `InstanceSettings` has become
`PlayableSettings` to correspond to the new `Playable` enum.

Arrangements also help reduce some complexity from the code for
instances and make some more unusual behaviors, like playing
instances backwards or rewinding instances (not implemented yet,
but planned), easier to reason about.

I'm sure there's other good uses for arrangements, too! But I think
the most common use case will be songs with intros.

## Other changes
- Remove `SoundMetadata` and move `semantic_duration` into `PlayableSettings`
- Rename `StereoSample` to `Frame`
- Added support for panning instances:
	- Added `Frame::panned`
	- Added a `panning` field and method to `InstanceSettings`
	- Added `AudioManager::set_instance_panning`
	- Added `Sequence::set_instance_panning`
- Added parameter mappings, which allow `Value`s to map to parameters with
custom scaling and offsets. `Value::Parameter` now contains a `Mapping`
as its second piece of data. `ParameterId`s can be converted into
`Value::Parameter`s with the default 1:1 mapping.
- Changed `Tween` to a C-style struct with named fields:
	- `duration` - the duration of the tween
	- `easing` - the easing function to use (linear, power, etc.)
	- `ease_direction` - the easing direction (in, out, in-out)
- Added chainable methods to more settings structs
- Replaced `AudioManager::events` with `AudioManager::pop_event`
- Add `Sequence:pause/stop/resume_sequence`
- Replace `AudioError::SupportedStreamConfigsError` with `AudioError::DefaultStreamConfigError`

# v0.1.2 - December 4th, 2020
Changes:
- Update cpal to 0.13.1
- Fix a crash when using a mono output config
- Use the system's default output config instead of the config with the highest sample rate in the first valid range

# v0.1.1 - November 18th, 2020
Changes:
- Fix readme path in Cargo.toml

# v0.1.0 - November 18th, 2020
First public release

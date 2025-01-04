# v0.10 - Unreleased

## Buffered audio

Kira now processes audio in chunks instead of one sample at a time. This means that
`Sound::process` no longer returns a `Frame`; instead it receives a slice of `Frame`s
to overwrite. `Effect::process` takes an input slice instead of a single frame,
and it overwrites that slice instead of returning a new `Frame`.

The benefit of this change is significantly improved performance. The criterion
benchmarks aren't comparable to the ones in v0.9.x and earlier, but in my unscientific
test, I can play about twice as many sounds on my PC without crackling.

There are some tradeoffs, but I think they're reasonable:

- Modulators are no longer sample accurate. Instead, they update once per internal
  processing chunk. Sounds and effects can interpolate between the previous and
  current modulator value using `Parameter::interpolated_value` to avoid discontinuities.
- Clocks are no longer sample accurate. For my use case which involves dynamically
  generating music, the default internal buffer size of 128 frames sounds almost exactly
  the same as sample-accurate clocks. You can adjust the internal buffer size to get
  the right tradeoff of performance vs. accuracy for your game. I have some ideas for
  how sample-accurate clocks could be implemented within the buffered architecture, so
  if you find yourself needing sample-accurate clocks, let me know!
- The delay effect can no longer have its delay time changed after the fact. If you
  know how to implement a delay that can smoothly change its delay time with the
  buffered architecture, please make a PR!

## Hierarchical mixer

Sounds now live inside mixer tracks. Previously, to play a sound on a mixer track, you
would use `StaticSoundData`/`StreamingSoundData::output_destination`. Now, you pass the
sound to `TrackHandle::play`. Additionally, tracks can contain other tracks. Tracks
can also route their outputs to send tracks, which are a separate concept now. This change
enables the following feature:

## Pausing and resuming mixer tracks

Mixer tracks can now be paused and resumed. Pausing a mixer track pauses all sounds playing
on the track, as well as all child tracks.

## Spatial audio overhaul

The concepts of spatial scenes and emitters have been removed, and listeners no longer
output to a mixer track. Instead, mixer tracks can optionally have spatial properties,
like position and spatialization strength. Sounds and child tracks on the track will
have spatialization applied relative to a specified listener.

This release also adds `Value::FromListenerDistance`, which can be used to map sound and
effect parameters to the distance between a spatial track and its corresponding listener.

## Simplified volume and playback rate types

Previously, `Volume` was an enum with `Amplitude` and `Decibels` variants, and `PlaybackRate`
was an enum with `Factor` and `Semitones` variants. There's a couple problems with this:

- It's unclear what scale a tween uses when tweening from one variant to another. For instance,
  if you tween a `Volume::Amplitude` to a `Volume::Decibels` with linear easing, is it
  linear in the amplitude domain or decibels?
- Amplitude isn't a good default representation for volume because it's not perceptually linear.

Now, everything that previously used `Volume` uses the simpler `Decibels` type, and
`PlaybackRate` always contains a factor. (`Semitones` still exists as a separate type that
implements `Into<PlaybackRate>`).

There's also a new `Panning` type that's used instead of bare `f64`s. Panning has been changed
so `-1.0` is left instead of `0.0`, since this makes more sense mathematically.

## Other changes

- Reorganized some types and modules to reduce unnecessary nesting
- Added `WaitingToResume` and `Resuming` variants to `PlaybackState`
- Changes to `Mapping`:
  - Added an `easing` field
  - Inputs are now always clamped to the input range
  - Removed the `Default` implementation
  - Added methods for performing math operations on the output range
- Implemented some math operations to `Value`
- Changed the fields of `Capacities` back to `u64`s
- `ClockInfoProvider` and `ModulatorValueProvider` and now combined into one
  `Info` struct, which also provides info about spatial audio state
- Added `CpalBackend::pop_cpu_usage` (desktop only for now)

# v0.9.6 - November 10, 2024

- Improve performance when adding or subtracting large `f64`s from a `ClockTime`
(thanks @crabm4n!)
- Fix UB detected by miri in the stacked borrows model (thanks @Imberflur!)

# v0.9.5 - September 2, 2024

- Update `glam` to 0.29.0

# v0.9.4 - August 8, 2024

- Fix bug where static sounds played backwards would never be marked as finished,
and thus never unloaded
- Update `triple_buffer` to 8.0.0

# v0.9.3 - June 15, 2024

- Update `glam` to 0.28.0

# v0.9.2 - June 2, 2024

- Fix `StaticSoundHandle/StreamingSoundHandle::pause/resume/stop` not taking effect
immediately if the sound has a start time. This was an unintended change from the
behavior in v0.8.x and earlier versions.
- Fix sounds erroneously reporting their state as `Playing` before playback has
resumed after calling `StaticSoundHandle/StreamingSoundHandle::resume_at` with
a non-immediate `StartTime`
- Fix sounds entering a limbo state where they output no sound and can never be
unloaded when their output destination (track or emitter) is removed
- Fix a bug where static sounds could enter a limbo state where they're stopped, but
never unloaded if the clock they're waiting on is removed

# v0.9.1 - May 31, 2024

- Fix sounds stopping after having already started if the clock they were originally
  waiting on stops
- Implement `Default` for `Region`, `EndPosition`, `PlaybackPosition`, and `Value`
- Implement `Debug` for handles, command types, and `ResourceController`

# v0.9.0 - May 11, 2024

## `ClockTime::fraction`

`ClockTime` now has a `fraction` field, which represents a fraction of a tick. This
means sounds and tweens can be scheduled for times in-between ticks.

In addition, `ClockHandle::fractional_position` has been removed because
`ClockHandle::time` provides that info anyway, and the shape of `ClockInfo` has
changed to hold a `ClockTime` (this is only relevant if you're creating implementations
of one of Kira's traits).

## Added configuration for the `CpalBackend`

(Implemented by @zeozeozeo)

The device and buffer size used by the `CpalBackend` are now configurable via
`CpalBackendSettings`.

## Most param changes are now infallible

Anything that could previously fail because of a command buffer filling up or
getting poisoned can no longer fail that way, so you can call functions
like `Emitter::set_position` as frequently as you want.

## Updated API for sound start positions and playback regions

v0.8 introduced a `playback_region` setting for static and streaming sounds
which replaced the previous `start_position` setting. It was meant to serve two
purposes:

- Allow you to play only a portion of a sound
- Allow setting the start position of the sound

However, these two purposes had an unintuitive interaction. Say you want to play
a sound starting 3 seconds in. In v0.8, you would do something like this:

```rs
manager.play(
  StaticSoundData::from_file(
    "test.ogg",
    StaticSoundSettings::new().playback_region(3..),
  )?,
)?;
```

Then let's say you wanted to seek the sound to 2 seconds. The sound would stop
because you've set the playback region not to include that part of the sound,
when all you really wanted to do was set the start position.

In v0.9, these two purposes are separated. The `playback_region` setting has been
reverted to the `start_position` setting from versions before v0.8, and to serve
the purpose of playing portions of sounds, `slice` methods have been added to
`StaticSoundData` and `StreamingSoundData`.

Note that the `loop_region` option added in v0.8 remains and works the same way
in v0.9.

## `StartTime::delayed`

Previously, you could delay the playback of a static sound by setting its
start position to a negative number. This only worked with static sounds, however,
not streaming sounds or tweens. This is now disallowed, which allows for some
minor internal code cleanup. In its place is the more explicit and intuitive
`StartTime::delayed`, which works with anything that has a `StartTime`.

## `Static/StreamingSoundHandle::resume_at`

Previously, if you paused a sound and set the fade-in tween to have a start time
in the future, the sound wouldn't become audible until the start time, but it would
still be advancing in the background as soon as you called `resume`. The `resume_at`
method delays playback until the specified `StartTime`.

## Added settings methods to `Static/StreamingSoundData`

Chainable methods have been added to `StaticSoundData` and `StreamingSoundData` to
change settings on the sound data. So instead of doing this:

```rs
let sound = StaticSoundData::from_file("sound.ogg", StaticSoundSettings::new()
  .volume(0.5)
  .playback_rate(2.0)
)?;
```

You can use this less verbose code:

```rs
let sound = StaticSoundData::from_file("sound.ogg")?
  .volume(0.5)
  .playback_rate(2.0);
```

`StaticSoundSettings` and `StreamingSoundSettings` still exist and are still fields
of the corresponding `SoundData`s, so you can still store settings independently
of a sound data.

## Resource capacities are now `u16`s

This allows for some performance improvements, but it does mean you are limited to
65,536 of each kind of resource (sounds, clocks, etc.). I apologize to anyone who
was trying to play more than 65k sounds.

## Remove `AudioManager::pause/resume`

This feature was thrown into a previous version of Kira as a quick and dirty way
to pause/resume all sounds. However, it's the wrong tool for the job, since it pauses
and resumes the entire audio system, which is almost never what you want.

## Performance improvements

The benchmarks run about 26-29% faster (on my machine) compared to v0.8.7.

## Other changes

- Added `android_shared_stdcxx` feature
- Update `glam` to 0.27
- Added `TrackBuilder::add_built_effect` and `TrackBuilder::with_built_effect`
- Improve performance when using `start_position` with streaming sounds
- Remove `RangeInclusive` conversions for `Region`s, as all current uses of
  `Region`s treat the end bound as exclusive
- Stop streaming sounds immediately if there's a decoding error
- Remove all uses of `#[non_exhaustive]`
- Add `#[must_use]` where appropriate
- Moved some types and modules to reduce excessive nesting
- `AudioManager::main_track` now returns `&mut TrackHandle` instead of `TrackHandle`

# v0.8.7 - January 31, 2024

- Fix ClockInfoProvider having poor timing resolution

# v0.8.6 - January 13, 2024

- Fix a typo in the readme
- Add `StreamingSoundData::duration`
- Make `AudioManager`s `Sync` if the backend is `Sync`
- Make the `CpalBackend` `Sync` on wasm targets
- Make the `MockBackend` `Sync`
- Update `glam` to v0.25.0

# v0.8.5 - September 22, 2023

- Fix `kira::spatial::scene::AddEmitterError` not being publicly available
- Fix some typos in the documentation
- Add the `assert_no_alloc` feature
- Fix garbage audio getting sent to surround sound channels

# v0.8.4 - June 19, 2023

- Add `serde` feature
- Implement `PartialOrd` for `ClockTime`
- Implement `Default` for `Volume` and `PlaybackRate`

# v0.8.3 - May 28, 2023

This release removes reverse playback support for streaming sounds. There's
pretty serious issues with garbled audio when streaming an mp3 or ogg file
backwards, and based the initial investigation, these issues won't be trivial to
fix. This feature may return in the future, but for now, you should not rely on
it.

# v0.8.2 - May 27, 2023

- Fix errors when streaming some ogg files

Known issues:

- Seeking can cause errors with short ogg files
- Reverse playback of ogg files results in garbled sound

# v0.8.1 - May 26, 2023

- Added `StaticSoundData::from_media_source`
- Added `StreamingSoundData::from_media_source`

# v0.8.0 - May 21, 2023

## Spatial audio

The main highlight of this release: Kira now supports 3D positional audio! This
is a simple implementation of 3D audio that only support volume attenuation
based on distance and panning based on direction. Doppler effect and
distance-based filtering and reverb are not supported yet.

This is meant to be an MVP of positional audio. Please give it a try and let me
know what improvements you'd like to see so I can gauge what expansions to this
API should look like.

## Modulators

Modulators are globally available streams of values that parameters like volume
and playback rate can be linked to. These are useful for controlling multiple
parameters with one value and using more complex modulations, like LFOs.

For anyone who's made implementations of traits in previous versions of Kira,
keep in mind that `Tweener` was renamed to `Parameter`, and now `Tweener` is the
name of a modulator implementation that comes with Kira.

## New effects

Two new effects were added: compressor and EQ filter. The compressor adjusts the
volume of audio to make louder parts quieter. An EQ filter a single band of a
parametric EQ useful for adjusting the volume of frequencies of sound.

## Playback region/loop region settings

The `start_position` setting for static and streaming sounds has been replaced
with a `playback_region` setting which lets you specify an end position for the
sound as well as a start position. The `loop_behavior` setting has been replaced
with `loop_region`, which lets you specify an end point for the loop. You can
now change the `loop_region` after the sound is created using the
`set_loop_region` function on the sound's handle.

## Other changes

- `StaticSoundData::frames` is now an `Arc<[Frame]>` instead of `Arc<Vec<Frame>>`
- Exposed the `Decoder` trait for streaming sounds
- Moved `PlaybackState` to the sound module
- Streaming sounds now support reverse playback
- Added `TrackBuilder::with_effect`
- Moved `ClockSpeed` to the `clock` module
- Moved `PlaybackRate` to the `sound` module

# v0.7.3 - March 18, 2023

- Fix compile error on WASM targets

# v0.7.2 - March 18, 2023

- Fix crackling on Mac OS when multiple applications are playing audio
- Update `cpal` to 0.15.1

# v0.7.1 - October 23, 2022

- Update `cpal` to 0.14.1
- Update `ringbuf` to 0.3.1
- Implement `PartialEq` for `StaticSoundData`, `DistortionKind`,
  `DistortionBuilder`, `FilterMode`, `FilterBuilder`, `PanningControlBuilder`,
  `ReverbBuilder`, `VolumeControlBuilder`, and `TrackRoutes`
- Implement `Debug` for `StaticSoundData` and `TrackRoutes`
- Implement `Clone` for `TrackRoutes`

# v0.7.0 - August 20th, 2022

This is a bugfix release, but unfortunately one of the bugfixes did require a
breaking change. Fortunately, this breaking change only affects people who have
created their own `Sound` or `Effect` implementations, which is not the most
common use case.

Fixes:

- Fix a panic when starting a `StaticSound` with a start position later than the
  end of the sound
- Fix negative start positions getting rounded up to `0.0`. Now sounds played
  with negative start positions will output silence until the position reaches
  `0.0`, at which point normal playback will resume. This is the behavior from
  versions 0.5.x and prior, and it was not meant to change in 0.6.x.
- Streaming sounds will no longer stop after they encounter an error, allowing
  them to recover from non-fatal errors and continue playback
- Fix a bug where if a sound was played with the start time set to a clock time,
  and the clock time had already passed by the time the sound was played, it
  would not start until the next tick of that clock

Breaking changes (only for `Sound` and `Effect` implementations):

- Removed the `on_clock_tick` callback from `Sound`, `Effect`, and `Tweener`
- Instead, `Sound::process`, `Effect::process`, and `Tweener::update` receive a
  `&ClockInfoProvider` argument. `ClockInfoProvider` can report the current
  state of any active clock.

# v0.6.1 - August 12th, 2022

- Added `ClockHandle::fractional_position`
- Added `StaticSoundData::with_settings` and
  `StaticSoundData::with_modified_settings`
- Changed the following functions to take `&self` arguments instead of
  `&mut self` (thanks @nobbele!):
  - `ClockHandle::set_speed`
  - `ClockHandle::start`
  - `ClockHandle::pause`
  - `ClockHandle::stop`
  - `AudioManager::pause`
  - `AudioManager::resume`
  - `TrackHandle::set_volume`
  - `TrackHandle::set_route`

# v0.6.0 - March 7th, 2022

Kira v0.6 is a complete rewrite of Kira with the following goals:

- Make the API more elegant
- Make the library leaner by removing features that weren't pulling their weight
- Provide a solid technical foundation for future feature development

Of course, the overall goals of Kira are the same: provide a library that fills
missing niches in most audio libraries and enables people to be creative with
their game audio.

## Streaming sounds

Kira now supports streaming sounds! Unlike static sounds, which keep all audio
data in memory, streaming sounds decode audio from the filesystem in realtime.
This has a much leaner memory footprint at the cost of higher CPU usage.

## The `Sound` trait

Static and streaming sounds are both implementors of the new `Sound` trait.
`Sound`s produce arbitrary streams of audio, so they can be used for both finite
sounds or infinite sounds, like voice chat audio. In this sense, they're similar
to `AudioStream`s from previous versions of Kira, but they can be automatically
unloaded when they're finished, and they can receive clock events (see below).
They're better integrated with the rest of Kira, making them first class
citizens instead of escape hatches.

Kira no longer has any concept of "instances". If you want to play a static
sound multiple times, you can clone it each time you want to pass it to
`AudioManager::play`. (Static sounds share their samples via an `Arc`, so
cloning is cheap.) Streaming sounds cannot be cloned since each one opens up a
new file handle.

## Clocks

Metronomes and sequences from previous versions of Kira were useful for complex
audio scripting, but most games don't need such a complex system. In Kira v0.6,
they're both replaced by clocks, which are simple timing sources.

Static and streaming sounds can be set to start playing at a certain clock time.
Additionally, tweens can be set to start at a clock time. This means anything
involving a tween, such as fading out a sound or changing its playback rate, can
be synced to a clock.

If you need a more complex system, you should be able to build it in gameplay
code using clocks as a building block.

## More flexible mixer routing

The mixer no longer makes a distinction between sub-tracks and send tracks. Any
sub-track can be routed to any number of other sub-tracks.

## Modular backends

Previous versions of Kira were hardcoded to use cpal to talk to the operation
system's audio drivers. Kira v0.6.0 has a `Backend` trait, so you can implement
your own backends.

## More permissive licensing

Kira is now available under the MIT or Apache-2.0 license. (Previous versions
were only available under MIT.)

## Feature removals

### Parameters

Previous versions of Kira had global "parameters" that you could link settings
to, like metronome tempos and instance playback rates. The only way to smoothly
tween a setting was to link it to a parameter and tween that parameter. It is
useful to be able to link multiple settings to one parameter, but the more
common use case was to create a parameter just to tween one setting, which isn't
very ergonomic.

In Kira v0.6, parameters have been removed, and all settings can be tweened
directly. In future versions, I'd like to bring back global parameters and allow
users to either tween settings directly _or_ link them to parameters, I just
haven't figured out a good way to architect that.

### Arrangements

The main purpose of arrangements was to make it easier to create looping music
with intro sections. It served that purpose well, but it was overkill for that
purpose, and it wouldn't work with streaming sounds.

### Groups

Groups were meant to help with pausing, resuming, stopping, and unloading large
categories of resources. It tried to cover different types of resources with
different notions of pausing, resuming, and stopping. It mapped decently to
instances and sequences, but I think it would have been likely that future
versions of Kira would have a resource type that groups didn't make sense for,
and the abstraction would become shakier over time. Furthermore, resource
management can be done from gameplay code, so it's not even necessary for Kira
to provide this feature.

## Changes since v0.6.0 beta 6

- Added volume control and panning control effects
- Removed the built-in panning control from mixer tracks

# v0.6.0 beta 6 - March 5th, 2022

- Moved the functionality from `kira-cpal` and `kira-loaders` into the main
  `kira` crate. `kira-cpal` and `kira-loaders` are now unneeded and deprecated.
  - `CpalBackend` from `kira-cpal` is now available as
    `kira::manager::backend::cpal::CpalBackend`
  - `kira_loaders::load` and `kira_loaders::load_from_cursor` are now
    `StaticSoundData::from_file` and `StaticSoundData::from_cursor`
  - `kira_loaders::stream` and `kira_loaders::stream_from_cursor` are now
    `StreamingSoundData::from_file` and `StreamingSoundData::from_cursor`
- Added `Renderer::on_change_sample_rate`, which the `Backend` should call if
  the audio sample rate changes
- Added `Effect::on_change_sample_rate`, which is called when the audio sample
  rate changes
- Changes to the `Backend` trait:
  - Added the associated type `Backend::settings` and the method
    `Backend::setup`, which is used for creating `Backend`s (basically a `new`
    function, but required by the `Backend` trait)
  - Renamed `Backend::init` to `Backend::start`
  - Removed `Backend::sample_rate`
  - Removed `UnusedResourceCollector` from the public API. `Backend`s are no
    longer responsible for dropping unused resources.
- Updated `MockBackend` to the new API
- Removed the `backend` argument to `AudioManager::new`
- Restructured `AudioManagerSettings`
- The cpal backend will now do its best to gracefully handle device
  disconnection and sample rate changes
- The cpal backend now works in wasm environments

# v0.6.0 beta 5 - January 17, 2022

- Fix static sounds not pausing/resuming/stopping immediately when playback is
  waiting for a clock tick
- Remove `From<f64>` implementation for `ClockSpeed`
- Change `AudioManager::add_clock` to take a `ClockSpeed` argument instead of an
  `impl Into<ClockSpeed>` argument

# v0.6.0 beta 4 - January 6, 2022

- Fix clocks not resetting their fractional position when stopped

# v0.6.0 beta 3 - January 4, 2022

- Fix clock tick 0 occurring one tick after the clock is started instead of
  immediately when the clock is started
- Fix static sound pause/resume/stop fades never starting when the start time is
  set to a clock time

# v0.6.0 beta 2 - January 3, 2022

- Remove `Clock` and `Clocks` from the public API
- Sounds and effects now have an `on_clock_tick` callback instead of having a
  `&Clocks` argument passed into `process`
- Remove parameters
- Settings that were previously `Value`s can now be tweened directly without
  needing to link them to a parameter
- All functions that send commands to the renderer now use `CommandError` as
  their error type
- Effects now have a similar structure to sounds
  - `EffectBuilder` - trait for types that can be converted to an `Effect` and a
    handle
  - `Effect` - runs on the renderer
- `TrackSettings` is now `TrackBuilder`. Effects can be added by passing an
  `EffectBuilder` to `TrackBuilder::add_effect`.
- Changes to the built-in effects:
  - Removed `Distortion` from the public API, `DistortionSettings` is now
    `DistortionBuilder`, added `DistortionHandle`
  - Removed `Delay` from the public API, `DelaySettings` is now `DelayBuilder`,
    added `DelayHandle`
  - Removed `Reverb` from the public API, `ReverbSettings` is now
    `ReverbBuilder`, added `ReverbHandle`
  - Removed `Filter` from the public API, `FilterSettings` is now
    `FilterBuilder`, added `FilterHandle`
- Renamed `Tweenable` to `Tweener`
- Added a `Tweenable` trait for things that a `Tweener` can control. `Tweener`
  is now generic over the type of the `Tweenable` value
- Volume settings now use the `Volume` type instead of `f64`
- Playback rate settings now use the `PlaybackRate` type instead of `f64`
- Clock speed settings now use the `ClockSpeed` type instead of `f64`
- Fix audio artifacts when a static sound loops
- Slight performance improvement when sounds are center-panned
- Allow configuring the main mixer track using a `TrackBuilder` in
  `AudioManagerSettings`

# v0.6.0 beta 1 - December 24, 2021

- Fix looping static sounds with a loop start position greater than 0.0 starting
  playback at that position. (The intended behavior is that the sound will still
  start at the beginning, but jump back to the loop start position after
  reaching the end.)

# v0.6.0 beta 0 - December 4, 2021

Complete rewrite, most things have changed

# v0.5.3 - May 31, 2021

- Fix an issue where the `AudioManager` cleanup would fail if there are existing
  track handles

# v0.5.2 - April 27, 2021

- Added `Sound::from_mp3_reader`, `Sound::from_ogg_reader`,
  `Sound::from_flac_reader`, and `Sound::from_wav_reader` (thanks @Zicklag!)
- (Hopefully) fixed an issue where capacities (the maximum number of instances,
  sequences, etc.) could decrease over time

# v0.5.1 - March 28, 2021

- Added a `Default` implementation for `TrackSends`
- Fixed a compile error when building Kira with the `serde_support` feature
  enabled

# v0.5.0 - March 7, 2021

## Additions

- Added send tracks, which you can route sub-tracks to in addition to their
  parent track. This is useful for sharing effects between multiple sub-tracks.
- Made the `volume` setting for `Track`s a `Value<f64>`, which means you can
  link it to a parameter
- Added `Main`/`Sub`/`SendTrackHandle::set_volume`
- Added an `init` callback to the `Effect` trait
- Added new effects: `Distortion`, `Delay`, and `Reverb`
- Added a `mix` setting for effects, which lets you blend dry and wet signal
- Added `InstanceHandle::position`, which gets the current playback position of
  the instance
- Added `CachedValue::with_valid_range` for clamping values

## Bugfixes

- The default filter cutoff is no longer outside of the range of human hearing

## Other changes

- Changed settings structs with an `id` field to use `Option<Id>`. This
  eliminates a confusing situation where cloning the same settings struct and
  passing it to multiple objects results in each object overwriting the previous
  one since it has the same ID.
- Changes to errors related to sending commands to the audio thread:
  - Command-sending related errors are now listed in the `CommandError` enum
  - `BackendDisconnected` is no longer included in error enums
  - Handle structs whose only error variant was `BackendDisconnected` now return
    a `CommandError`
  - `TrackHandleError` was split into `AddEffectError` and `RemoveEffectError`
- `MetronomeHandle.event_iter` has been replaced with
  `MetronomeHandle.pop_event`, which works the same way as
  `SequenceInstanceHandle.pop_event`
- Changed all occurrences of the term "pitch" to "playback rate"

# v0.4.1 - January 23, 2021

Added serde support for `Arrangement`s and `SoundClip`s

# v0.4.0 - January 23, 2021

## `wasm32` support

Kira now runs on any platform supporting `wasm32` and having a `cpal` backend.
This means one can now run an instance of Kira in a web browser.

## Handle-based API

The API has gone through a major revision. Previously, to do just about
anything, you would have to use the `AudioManager`:

```rust
audio_manager.stop_instance(instance_id)?;
audio_manager.set_metronome_tempo(Tempo(128.0))?;
// etc...
```

This meant that you had to pass a reference to the audio manager to every part
of the code that needs it. It also meant that the `AudioManager` struct had an
overwhelming number of methods.

The API has been changed so that whenever you create a new thing (like a sound,
an instance, or a mixer track), you receive a handle to that thing that you can
use to control it.

```rust
let mut sound = audio_manager.load_sound("sound.ogg", SoundSettings::default())?;
let mut instance = sound.play(InstanceSettings::default())?;
instance.set_pitch(2.0)?;
```

## Multiple metronomes

You can now create multiple metronomes. Each sequence instance can be assigned
to a different metronome, and interval events can be received from a
`MetronomeHandle`:

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

Most people will only need one metronome at a time - the main point of this is
to move more functionality out of the `AudioManager` struct.

## Serde support

Sequences and most config structs now have serialization/deserialization support
via the `serde_support` feature.

## Improved error handling

The capacity limits specified in `AudioManagerSettings` are now enforced, and
the audio manager will also check that when you remove something with an ID,
something with that ID actually exists. Because this creates a lot of new error
variants, the large `AudioError` enum has been split up into smaller,
situational error enums.

# v0.3.0 - December 26th, 2020

## Per-sequence custom event types

Previously, custom events emitted by sequences were retrieved by calling
`AudioManager::pop_event`, which meant that the audio manager had a generic type
parameter for custom events, and every sequence had to emit custom events of the
same type.

Now, each sequence has its own custom event type, and you receive those events
from an `EventReceiver` that the audio manager returns when you call
`AudioManager::add_sequence`. This gives you more flexibility in how you
organize your custom events as well as moving some functionality out of the
`AudioManager` struct, which already has a lot of methods.

## Audio streams

Audio streams provide a way of sending arbitrary audio data to the mixer.
Sometimes, you need to play audio that you generate in real time, like voice
chats. This feature lets you do that.

## Groups

Sounds, arrangements, and sequences can now be grouped so that they can all be
paused, resumed, or stopped at once. This is useful for controlling broad
categories of sounds, like gameplay sounds or music.

## Other changes

- Added `Sequence::play_random`
- Added `Value::Random`
- Renamed `Sound::new` to `Sound::from_frames`
- Audio file format decoding is now gated behind feature flags
- Changed functions for pausing, resuming, and stopping instances to take
  settings structs (`PauseInstanceSettings`, `ResumeInstanceSettings`, and
  `StopInstanceSettings`)
- When resuming an instance, you can now choose to have it seek backward to the
  time when it was paused. This is useful if you need to keep audio synced up
  with something in-game, but you still want a smooth fade out when pausing the
  game.
- Renamed `Sequence::emit_custom_event` to `Sequence::emit`
- Added `AudioManager::seek_instance` and `AudioManager::seek_instance_to` for
  changing the playback positions of running instances
- Refined the behavior of looping backwards instances
- Added `Tween::linear`
- Make `Arrangement::settings` private and add settings parameters to
  `Arrangement::new_loop` and `Arrangement::new_loop_with_intro`

# v0.2.0 - December 6th, 2020

## Arrangements

This release adds `Arrangement`s, which allow you to stitch together multiple
sounds into a larger sound that you can play instances of.

The main use case for this is setting up seamless loops, including looping songs
with intros. The previous release could create seamless loops, but only if the
sound didn't have an intro.

Functions that dealt with instances of sounds now deal with instances of sounds
_or_ arrangements. `InstanceSettings` has become `PlayableSettings` to
correspond to the new `Playable` enum.

Arrangements also help reduce some complexity from the code for instances and
make some more unusual behaviors, like playing instances backwards or rewinding
instances (not implemented yet, but planned), easier to reason about.

I'm sure there's other good uses for arrangements, too! But I think the most
common use case will be songs with intros.

## Other changes

- Remove `SoundMetadata` and move `semantic_duration` into `PlayableSettings`
- Rename `StereoSample` to `Frame`
- Added support for panning instances:
  - Added `Frame::panned`
  - Added a `panning` field and method to `InstanceSettings`
  - Added `AudioManager::set_instance_panning`
  - Added `Sequence::set_instance_panning`
- Added parameter mappings, which allow `Value`s to map to parameters with
  custom scaling and offsets. `Value::Parameter` now contains a `Mapping` as its
  second piece of data. `ParameterId`s can be converted into `Value::Parameter`s
  with the default 1:1 mapping.
- Changed `Tween` to a C-style struct with named fields:
  - `duration` - the duration of the tween
  - `easing` - the easing function to use (linear, power, etc.)
  - `ease_direction` - the easing direction (in, out, in-out)
- Added chainable methods to more settings structs
- Replaced `AudioManager::events` with `AudioManager::pop_event`
- Add `Sequence:pause/stop/resume_sequence`
- Replace `AudioError::SupportedStreamConfigsError` with
  `AudioError::DefaultStreamConfigError`

# v0.1.2 - December 4th, 2020

Changes:

- Update cpal to 0.13.1
- Fix a crash when using a mono output config
- Use the system's default output config instead of the config with the highest
  sample rate in the first valid range

# v0.1.1 - November 18th, 2020

Changes:

- Fix readme path in Cargo.toml

# v0.1.0 - November 18th, 2020

First public release

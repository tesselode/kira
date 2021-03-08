# Kira

#### [Documentation](https://docs.rs/kira/) | [Web demo](https://kira-demo.surge.sh/) [(source)](https://github.com/Moxinilian/kira-web-demo)

Kira is an audio library designed to help create expressive audio
for games. Besides the common features you'd expect from an audio
library, it provides interfaces for scripting audio events,
seamlessly looping complex pieces of music, smoothly changing
parameters, and more.

## Examples

### Simple sound playback

```rust
let mut audio_manager = AudioManager::new(AudioManagerSettings::default())?;
let mut sound_handle = audio_manager.load_sound("sound.ogg", SoundSettings::default())?;
sound_handle.play(InstanceSettings::default())?;
```

### Looping a song while preserving trailing sounds

```rust
let sound_handle = audio_manager.load_sound(
	"loop.ogg",
	SoundSettings::new().semantic_duration(Tempo(128.0).beats_to_seconds(8.0)),
)?;
let mut arrangement_handle = audio_manager.add_arrangement(Arrangement::new_loop(
	&sound_handle,
	LoopArrangementSettings::default(),
))?;
arrangement_handle.play(InstanceSettings::default())?;
```

### Playing sounds and emitting events in time with a metronome

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum CustomEvent {
	Kick,
}

let kick_sound_handle = audio_manager.load_sound("kick.wav", SoundSettings::default())?;
let mut metronome_handle =
	audio_manager.add_metronome(MetronomeSettings::new().tempo(Tempo(150.0)))?;
audio_manager.start_sequence(
	{
		let mut sequence = Sequence::new(SequenceSettings::default());
		sequence.start_loop();
		sequence.play(&kick_sound_handle, InstanceSettings::default());
		sequence.emit(CustomEvent::Kick);
		sequence.wait(kira::Duration::Beats(1.0));
		sequence
	},
	SequenceInstanceSettings::new().metronome(&metronome_handle),
)?;
metronome_handle.start()?;
```

## Platform support

Kira should support all of the platforms supported by cpal.
Windows, Linux, and WASM have been tested.

## Roadmap

Kira is in early development, and is not production ready.
Here are some features that I'd like the library to have:
- More mixer effects (EQ, compressor, better reverb, etc.)
- C API
- Streaming sounds
- 3d audio (maybe!)

## Contributing

I'd love for other people to get involved with development! Since the
library is still in the early stages, I'm open to all kinds of input -
bug reports, feature requests, design critiques, etc. Feel free to
open an issue or pull request!

## License

[MIT](license.md)

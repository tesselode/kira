# Kira

Kira is an audio library designed to help create expressive audio
for games. It aims to fill the holes in many game engines' built-in
audio APIs with features like custom loop points and audio event
scripting.

The repo contains the main library, written in Rust, as well as
Lua bindings for use with frameworks like LÖVE.

## Examples

### Simple sound playback

```rust
let mut audio_manager = AudioManager::<()>::new(AudioManagerSettings::default())?;
let sound_id = audio_manager.add_sound(Sound::from_file("loop.ogg", PlayableSettings::default())?)?;
audio_manager.play(sound_id, InstanceSettings::default())?;
```

### Looping a song with an intro

```rust
let sound_id = audio_manager.add_sound(Sound::from_file(
	"loop.ogg",
	PlayableSettings {
		semantic_duration: Some(Tempo(128.0).beats_to_seconds(16.0)),
		..Default::default()
	},
)?)?;
// when the sound loops, start the loop 4 beats in
let loop_start = Tempo(128.0).beats_to_seconds(4.0);
audio_manager.play(sound_id, InstanceSettings::new().loop_start(loop_start))?;
```

### Timing sounds with a metronome

```rust
let mut sequence = Sequence::new();
sequence.start_loop();
sequence.wait_for_interval(4.0);
sequence.play(kick_drum_sound_id, InstanceSettings::default());
sequence.emit_custom_event(CustomEvent::KickDrum);
audio_manager.start_sequence(sequence)?;
audio_manager.start_metronome()?;
```

## Roadmap

Kira is in very early development, and is not production ready.
Here are some features that I'd like the library to have:
- More observable events (like `InstanceFinished`)
- Custom commands that sequences can wait on
- Tween ease modes
- Parameter mapping
- More mixer effects (delay, reverb, EQ, etc.)
- Nested mixer tracks and send tracks
- C API
- A project system for setting up assets using JSON (maybe,
not entirely sure if the library needs this)

## Contributing

I'd love for other people to get involved with development! Since the
library is still in the early stages, I'm open to all kinds of input -
bug reports, feature requests, design critiques, etc. Feel free to
open an issue or pull request!

## License

[MIT](license.md)

# Kira

Kira is an audio library designed to help create expressive audio
for games. It aims to fill the holes in many game engines' built-in
audio APIs with features like custom loop points and audio event
scripting.

You can find a demo of some of Kira's features [here](https://github.com/tesselode/kira-demo/).

## Examples

### Simple sound playback

```rust
let mut audio_manager = AudioManager::new(AudioManagerSettings::default())?;
let sound_id = audio_manager.load_sound("loop.ogg", PlayableSettings::default())?;
audio_manager.play(sound_id, InstanceSettings::default())?;
```

### Looping a song while preserving trailing sounds

```rust
let sound_id = audio_manager.load_sound(
	std::env::current_dir()?.join("assets/loop.wav"),
	PlayableSettings {
		semantic_duration: Some(Tempo(140.0).beats_to_seconds(16.0)),
		..Default::default()
	},
)?;
let arrangement_id = audio_manager.add_arrangement(Arrangement::new_loop(sound_id))?;
audio_manager.play(arrangement_id, InstanceSettings::default())?;
```

### Playing sounds and emitting events in time with a metronome

```rust
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
enum CustomEvent {
	KickDrum,
}

let mut sequence = Sequence::<CustomEvent>::new(SequenceSettings::default());
sequence.start_loop();
sequence.wait_for_interval(4.0);
sequence.play(kick_drum_sound_id, InstanceSettings::default());
sequence.emit(CustomEvent::KickDrum);
let (id, mut event_receiver) = audio_manager.start_sequence(sequence, SequenceInstanceSettings::default())?;
// start the metronome so the sequence will have a pulse to listen for
audio_manager.start_metronome()?;
// pop events
while let Some(event) = event_receiver.pop() {
	println!("{:?}", event);
}
```

## Roadmap

Kira is in very early development, and is not production ready.
Here are some features that I'd like the library to have:
- More observable events (like `InstanceFinished`)
- Custom commands that sequences can wait on
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

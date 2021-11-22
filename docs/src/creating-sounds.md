# Creating `Sound` Implementations

Sounds in Kira have two phases:
1. The `SoundData` phase: the user has created a sound, but it is
not yet producing sound on the audio thread. If the sound data has
settings, they should still be customizable at this point.
2. The `Sound` phase: the user has played the sound using
`AudioManager::play`, which transfers ownership to the audio thread.

The `SoundData` trait has the `into_sound` function, which "splits"
the sound data into the live `Sound` and a `Handle` which the user
can use to control the sound from gameplay code.

`Sound`s simply produce a `Frame` of audio each time `process` is
called. A `Sound` can be a finite chunk of audio, an infinite stream
of audio (e.g. voice chat), or anything else.

Kira does not provide any tools for passing messages from gameplay
code to a `Sound` or vice versa. (Internally, Kira uses the
[`ringbuf`](https://crates.io/crates/ringbuf) crate for this purpose.)

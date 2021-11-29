# `kira-cpal`

`kira-cpal` is a Kira backend for desktop targets.

## Examples

### Setting up an `AudioManager` with a `CpalBackend`

```rust
use kira::manager::{AudioManager, AudioManagerSettings};
use kira_cpal::CpalBackend;

let mut manager = AudioManager::new(
	CpalBackend::new()?,
	AudioManagerSettings::default(),
)?;
```

## License

This project is licensed under either of

- Apache License, Version 2.0 (LICENSE-APACHE)
- MIT license (LICENSE-MIT)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in `kira-cpal` by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.

# Installation

To use Kira in your project, add it to the Cargo.toml file for your crate in the
`dependencies` section.

```toml
[dependencies]
kira = "0.7.1"
```

## Features

By default, Kira comes with a `cpal` backend for communicating with the
operating system's audio drivers and support for mp3, ogg, flac, and wav files.
You can manually pick which of these features to enable by setting
`default-features` to `false` and listing the specific features you want.

For example, if you're only going to use ogg files, you can disable the other
file types to save some compile time and binary size:

```toml
[dependencies]
kira = { version = "0.7.1", default-features = false, features = ["cpal", "ogg"] }
```

## Performance

By default, Rust programs run with the `dev` profile are not optimized. This can
lead to poor performance of audio playback and long loading times for audio
files. You can alleviate this by building Kira and its audio-related
dependencies with a higher optimization level. Add the following to your
Cargo.toml:

```toml
[profile.dev.package.kira]
opt-level = 3

[profile.dev.package.cpal]
opt-level = 3

[profile.dev.package.symphonia]
opt-level = 3

[profile.dev.package.symphonia-bundle-mp3]
opt-level = 3

[profile.dev.package.symphonia-format-ogg]
opt-level = 3

[profile.dev.package.symphonia-codec-vorbis]
opt-level = 3

[profile.dev.package.symphonia-bundle-flac]
opt-level = 3

[profile.dev.package.symphonia-format-wav]
opt-level = 3

[profile.dev.package.symphonia-codec-pcm]
opt-level = 3
```

You can also build all of your projects with a higher optimization level by
using this snippet instead:

```toml
[profile.dev.package."*"]
opt-level = 3
```

Building dependencies with a higher optimization level does increase compile
times, but only when compiling your project from scratch. If you only make
changes to your crate, you're not recompiling the dependencies, so you don't
suffer from the longer compilation step in that case. Building dependencies
optimized and the main crate unoptimized can be a good balance of performance
and compile times for games.

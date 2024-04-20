/*!
Positions audio in 3D space.

Oftentimes, it's useful to give sounds a location in a 3D (or 2D) space
and play back those sounds from the perspective of a character's ears
located somewhere else in that space. For example, as a player character
gets closer to a waterfall, you may want the sound of the waterfall to
get louder.

Kira's spatial audio system currently supports:

- **Attenuation**: changing the volume of sounds depending on the distance
from the listener
- **Spatialization**: changing the panning (left ear/right ear positioning)
of sounds depending on their angle from the listener

There are three components to Kira's spatial audio system:

- **Emitters**, which produce sound from a point in space
- **Listeners**, which receive sound from the emitters from a point in space
- **Spatial scenes**, which hold listeners and emitters

# Geometric parameters

Some functions take parameters of type `mint::Vector3` and `mint::Quaternion`.
[`mint`](https://crates.io/crates/mint) is a library that defines common math
types for interoperability between math libraries. You can pass in types from any
library that supports conversion to `mint` types into Kira's functions. The
following examples will use `glam` with the `mint` feature enabled.

# Usage

To use spatial audio, first create a spatial scene using
[`AudioManager::add_spatial_scene`](crate::manager::AudioManager::add_spatial_scene):

```no_run
use kira::{
	manager::{AudioManager, AudioManagerSettings, backend::DefaultBackend},
	spatial::scene::SpatialSceneSettings,
};

let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
let mut scene = manager.add_spatial_scene(SpatialSceneSettings::default())?;
# Result::<(), Box<dyn std::error::Error>>::Ok(())
```

Then, create a listener using
[`SpatialSceneHandle::add_listener`](crate::spatial::scene::SpatialSceneHandle::add_listener):

```no_run
use kira::{
	manager::{AudioManager, AudioManagerSettings, backend::DefaultBackend},
	spatial::{scene::SpatialSceneSettings, listener::ListenerSettings},
};
use glam::{Vec3, Quat};

let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
let mut scene = manager.add_spatial_scene(SpatialSceneSettings::default())?;
let listener = scene.add_listener(Vec3::ZERO, Quat::IDENTITY, ListenerSettings::default())?;
# Result::<(), Box<dyn std::error::Error>>::Ok(())
```

Next, create an emitter using
[`SpatialSceneHandle::add_emitter`](crate::spatial::scene::SpatialSceneHandle::add_emitter):

```no_run
use kira::{
	manager::{AudioManager, AudioManagerSettings, backend::DefaultBackend},
	spatial::{
		scene::SpatialSceneSettings,
		listener::ListenerSettings,
		emitter::EmitterSettings,
	},
};
use glam::{Vec3, Quat};

let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
let mut scene = manager.add_spatial_scene(SpatialSceneSettings::default())?;
let listener = scene.add_listener(Vec3::ZERO, Quat::IDENTITY, ListenerSettings::default())?;
let emitter = scene.add_emitter(Vec3::new(100.0, 100.0, 0.0), EmitterSettings::default())?;
# Result::<(), Box<dyn std::error::Error>>::Ok(())
```

If you run the program now, you won't hear anything. That's because there's nothing
playing through the emitter. Let's play a sound:

```no_run
use kira::{
	manager::{AudioManager, AudioManagerSettings, backend::DefaultBackend},
	spatial::{
		scene::SpatialSceneSettings,
		listener::ListenerSettings,
		emitter::EmitterSettings,
	},
	sound::static_sound::{StaticSoundData, StaticSoundSettings},
};
use glam::{Vec3, Quat};

let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
let mut scene = manager.add_spatial_scene(SpatialSceneSettings::default())?;
let listener = scene.add_listener(Vec3::ZERO, Quat::IDENTITY, ListenerSettings::default())?;
let emitter = scene.add_emitter(Vec3::new(100.0, 100.0, 0.0), EmitterSettings::default())?;
let sound = StaticSoundData::from_file("sound.ogg")?
	.output_destination(&emitter);
manager.play(sound)?;
# Result::<(), Box<dyn std::error::Error>>::Ok(())
```

Now you can move the emitter or the listener using
[`EmitterHandle::set_position`](crate::spatial::emitter::EmitterHandle::set_position)
or [`ListenerHandle::set_position`](crate::spatial::listener::ListenerHandle::set_position),
respectively, and the volume and panning of sounds will update automatically.

# Customizing the physical characteristics of sounds

Each emitter has separate controls for how the volume is affected by distance from the
listener and how panning is affected by angle. Attenuation and spatialization can be
disabled entirely. See [`EmitterSettings`](crate::spatial::emitter::EmitterSettings)
for more details.
*/

pub mod emitter;
pub mod listener;
pub mod scene;

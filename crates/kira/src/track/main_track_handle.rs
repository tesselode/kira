use glam::{Quat, Vec3};

use crate::{
	command::{CommandWriter, ValueChangeCommand},
	manager::backend::resources::ResourceController,
	sound::{Sound, SoundData},
	spatial::{
		listener::{self, Listener, ListenerHandle},
		scene::SpatialSceneId,
	},
	tween::{Tween, Value},
	PlaySoundError, ResourceLimitReached, Volume,
};

/// Controls the main mixer track.
#[derive(Debug)]
pub struct MainTrackHandle {
	pub(crate) set_volume_command_writer: CommandWriter<ValueChangeCommand<Volume>>,
	pub(crate) sound_controller: ResourceController<Box<dyn Sound>>,
	pub(crate) listener_controller: ResourceController<Listener>,
}

impl MainTrackHandle {
	/// Plays a sound.
	pub fn play<D: SoundData>(
		&mut self,
		sound_data: D,
	) -> Result<D::Handle, PlaySoundError<D::Error>> {
		let (sound, handle) = sound_data
			.into_sound()
			.map_err(PlaySoundError::IntoSoundError)?;
		self.sound_controller
			.insert(sound)
			.map_err(|_| PlaySoundError::SoundLimitReached)?;
		Ok(handle)
	}

	/// Adds a listener for a spatial scene to this track.
	///
	/// An unrotated listener should face in the negative Z direction with
	/// positive X to the right and positive Y up.
	pub fn add_listener(
		&mut self,
		scene: impl Into<SpatialSceneId>,
		position: impl Into<Value<mint::Vector3<f32>>>,
		orientation: impl Into<Value<mint::Quaternion<f32>>>,
	) -> Result<ListenerHandle, ResourceLimitReached> {
		let position: Value<mint::Vector3<f32>> = position.into();
		let orientation: Value<mint::Quaternion<f32>> = orientation.into();
		self.add_listener_inner(scene.into(), position.to_(), orientation.to_())
	}

	/// Sets the (post-effects) volume of the mixer track.
	pub fn set_volume(&mut self, volume: impl Into<Value<Volume>>, tween: Tween) {
		self.set_volume_command_writer.write(ValueChangeCommand {
			target: volume.into(),
			tween,
		})
	}

	/// Returns the maximum number of sounds that can play simultaneously on this track.
	#[must_use]
	pub fn sound_capacity(&self) -> u16 {
		self.sound_controller.capacity()
	}

	/// Returns the number of sounds currently playing on this track.
	#[must_use]
	pub fn num_sounds(&self) -> u16 {
		self.sound_controller.len()
	}

	/// Returns the maximum number of listeners that can exist on this track at a time.
	#[must_use]
	pub fn listener_capacity(&self) -> u16 {
		self.listener_controller.capacity()
	}

	/// Returns the number of listeners on this track.
	#[must_use]
	pub fn num_listeners(&self) -> u16 {
		self.listener_controller.len()
	}

	fn add_listener_inner(
		&mut self,
		spatial_scene_id: SpatialSceneId,
		position: Value<Vec3>,
		orientation: Value<Quat>,
	) -> Result<ListenerHandle, ResourceLimitReached> {
		let (command_writers, command_readers) = listener::command_writers_and_readers();
		let listener = Listener::new(spatial_scene_id, command_readers, position, orientation);
		let handle = ListenerHandle {
			shared: listener.shared(),
			command_writers,
		};
		self.listener_controller.insert(listener)?;
		Ok(handle)
	}
}

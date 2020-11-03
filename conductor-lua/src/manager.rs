use conductor::manager::{AudioManager, AudioManagerSettings};
use mlua::prelude::*;

use crate::{
	error::ConductorLuaError, event::CustomEvent, event::LEvent, instance::LInstanceId,
	instance::LInstanceSettings, metronome::LMetronomeSettings, parameter::LParameterId,
	sequence::LSequence, sequence::LSequenceId, sound::LSoundId, sound::LSoundSettings,
	tempo::LTempo, track::LSubTrackId, track::LTrackSettings, tween::LTween, value::LValue,
};

pub struct LAudioManagerSettings(pub AudioManagerSettings);

impl<'lua> FromLua<'lua> for LAudioManagerSettings {
	fn from_lua(lua_value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
		match lua_value {
			LuaNil => Ok(LAudioManagerSettings(AudioManagerSettings::default())),
			LuaValue::Table(table) => {
				let mut settings = AudioManagerSettings::default();
				if table.contains_key("numCommands")? {
					settings.num_commands = table.get("numCommands")?;
				}
				if table.contains_key("numEvents")? {
					settings.num_events = table.get("numEvents")?;
				}
				if table.contains_key("numSounds")? {
					settings.num_sounds = table.get("numSounds")?;
				}
				if table.contains_key("numParameters")? {
					settings.num_parameters = table.get("numParameters")?;
				}
				if table.contains_key("numInstances")? {
					settings.num_instances = table.get("numInstances")?;
				}
				if table.contains_key("numSequences")? {
					settings.num_sequences = table.get("numSequences")?;
				}
				if table.contains_key("numTracks")? {
					settings.num_tracks = table.get("numTracks")?;
				}
				if table.contains_key("numEffectsPerTrack")? {
					settings.num_effects_per_track = table.get("numEffectsPerTrack")?;
				}
				if table.contains_key("metronomeSettings")? {
					settings.metronome_settings =
						table.get::<_, LMetronomeSettings>("metronomeSettings")?.0;
				}
				Ok(LAudioManagerSettings(settings))
			}
			value => Err(LuaError::external(ConductorLuaError::wrong_argument_type(
				"audio manager settings",
				"table",
				value,
			))),
		}
	}
}

pub struct LAudioManager(pub AudioManager<CustomEvent>);

impl LAudioManager {
	pub fn new(settings: LAudioManagerSettings) -> LuaResult<Self> {
		match AudioManager::new(settings.0) {
			Ok(manager) => Ok(Self(manager)),
			Err(error) => Err(LuaError::external(error)),
		}
	}
}

impl LuaUserData for LAudioManager {
	fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_method_mut("addParameter", |_: &Lua, this: &mut Self, value: f64| {
			this.0
				.add_parameter(value)
				.map(|id| LParameterId(id))
				.map_err(|error| LuaError::external(error))
		});

		methods.add_method_mut(
			"removeParameter",
			|_: &Lua, this: &mut Self, id: LParameterId| {
				this.0
					.remove_parameter(id.0)
					.map_err(|error| LuaError::external(error))
			},
		);

		methods.add_method_mut(
			"setParameter",
			|_: &Lua, this: &mut Self, (id, value, tween): (LParameterId, f64, Option<LTween>)| {
				this.0
					.set_parameter(id.0, value, tween.map(|tween| tween.0))
					.map_err(|error| LuaError::external(error))
			},
		);

		methods.add_method_mut(
			"addSubTrack",
			|_: &Lua, this: &mut Self, settings: LTrackSettings| {
				this.0
					.add_sub_track(settings.0)
					.map(|id| LSubTrackId(id))
					.map_err(|error| LuaError::external(error))
			},
		);

		methods.add_method_mut(
			"removeSubTrack",
			|_: &Lua, this: &mut Self, id: LSubTrackId| {
				this.0
					.remove_sub_track(id.0)
					.map_err(|error| LuaError::external(error))
			},
		);

		methods.add_method_mut(
			"loadSound",
			|_: &Lua, this: &mut Self, (path, settings): (LuaString, LSoundSettings)| match this
				.0
				.load_sound(path.to_str()?, settings.0)
			{
				Ok(id) => Ok(LSoundId(id)),
				Err(error) => Err(LuaError::external(error)),
			},
		);

		methods.add_method_mut(
			"unloadSound",
			|_: &Lua, this: &mut Self, id: LSoundId| match this.0.unload_sound(id.0) {
				Ok(_) => Ok(()),
				Err(error) => Err(LuaError::external(error)),
			},
		);

		methods.add_method_mut(
			"playSound",
			|_: &Lua, this: &mut Self, (sound_id, settings): (LSoundId, LInstanceSettings)| {
				match this.0.play_sound(sound_id.0, settings.0) {
					Ok(id) => Ok(LInstanceId(id)),
					Err(error) => Err(LuaError::external(error)),
				}
			},
		);

		methods.add_method_mut(
			"setInstanceVolume",
			|_: &Lua, this: &mut Self, (id, volume): (LInstanceId, LValue)| match this
				.0
				.set_instance_volume(id.0, volume.0)
			{
				Ok(_) => Ok(()),
				Err(error) => Err(LuaError::external(error)),
			},
		);

		methods.add_method_mut(
			"setInstancePitch",
			|_: &Lua, this: &mut Self, (id, pitch): (LInstanceId, LValue)| match this
				.0
				.set_instance_pitch(id.0, pitch.0)
			{
				Ok(_) => Ok(()),
				Err(error) => Err(LuaError::external(error)),
			},
		);

		methods.add_method_mut(
			"pauseInstance",
			|_: &Lua, this: &mut Self, (id, fade_tween): (LInstanceId, Option<LTween>)| match this
				.0
				.pause_instance(id.0, fade_tween.map(|tween| tween.0))
			{
				Ok(_) => Ok(()),
				Err(error) => Err(LuaError::external(error)),
			},
		);

		methods.add_method_mut(
			"resumeInstance",
			|_: &Lua, this: &mut Self, (id, fade_tween): (LInstanceId, Option<LTween>)| match this
				.0
				.resume_instance(id.0, fade_tween.map(|tween| tween.0))
			{
				Ok(_) => Ok(()),
				Err(error) => Err(LuaError::external(error)),
			},
		);

		methods.add_method_mut(
			"stopInstance",
			|_: &Lua, this: &mut Self, (id, fade_tween): (LInstanceId, Option<LTween>)| match this
				.0
				.stop_instance(id.0, fade_tween.map(|tween| tween.0))
			{
				Ok(_) => Ok(()),
				Err(error) => Err(LuaError::external(error)),
			},
		);

		methods.add_method_mut(
			"pauseInstancesOfSound",
			|_: &Lua, this: &mut Self, (id, fade_tween): (LSoundId, Option<LTween>)| match this
				.0
				.pause_instances_of_sound(id.0, fade_tween.map(|tween| tween.0))
			{
				Ok(_) => Ok(()),
				Err(error) => Err(LuaError::external(error)),
			},
		);

		methods.add_method_mut(
			"resumeInstancesOfSound",
			|_: &Lua, this: &mut Self, (id, fade_tween): (LSoundId, Option<LTween>)| match this
				.0
				.resume_instances_of_sound(id.0, fade_tween.map(|tween| tween.0))
			{
				Ok(_) => Ok(()),
				Err(error) => Err(LuaError::external(error)),
			},
		);

		methods.add_method_mut(
			"stopInstancesOfSound",
			|_: &Lua, this: &mut Self, (id, fade_tween): (LSoundId, Option<LTween>)| match this
				.0
				.stop_instances_of_sound(id.0, fade_tween.map(|tween| tween.0))
			{
				Ok(_) => Ok(()),
				Err(error) => Err(LuaError::external(error)),
			},
		);

		methods.add_method_mut(
			"setMetronomeTempo",
			|_: &Lua, this: &mut Self, tempo: LTempo| match this.0.set_metronome_tempo(tempo.0) {
				Ok(_) => Ok(()),
				Err(error) => Err(LuaError::external(error)),
			},
		);

		methods.add_method_mut(
			"startMetronome",
			|_: &Lua, this: &mut Self, _: ()| match this.0.start_metronome() {
				Ok(_) => Ok(()),
				Err(error) => Err(LuaError::external(error)),
			},
		);

		methods.add_method_mut(
			"pauseMetronome",
			|_: &Lua, this: &mut Self, _: ()| match this.0.pause_metronome() {
				Ok(_) => Ok(()),
				Err(error) => Err(LuaError::external(error)),
			},
		);

		methods.add_method_mut(
			"stopMetronome",
			|_: &Lua, this: &mut Self, _: ()| match this.0.stop_metronome() {
				Ok(_) => Ok(()),
				Err(error) => Err(LuaError::external(error)),
			},
		);

		methods.add_method_mut(
			"startSequence",
			|_: &Lua, this: &mut Self, sequence: LSequence| match this.0.start_sequence(sequence.0)
			{
				Ok(id) => Ok(LSequenceId(id)),
				Err(error) => Err(LuaError::external(error)),
			},
		);

		methods.add_method_mut(
			"muteSequence",
			|_: &Lua, this: &mut Self, id: LSequenceId| {
				this.0
					.mute_sequence(id.0)
					.map_err(|error| LuaError::external(error))
			},
		);

		methods.add_method_mut(
			"unmuteSequence",
			|_: &Lua, this: &mut Self, id: LSequenceId| {
				this.0
					.unmute_sequence(id.0)
					.map_err(|error| LuaError::external(error))
			},
		);

		methods.add_method_mut(
			"pauseSequence",
			|_: &Lua, this: &mut Self, id: LSequenceId| {
				this.0
					.pause_sequence(id.0)
					.map_err(|error| LuaError::external(error))
			},
		);

		methods.add_method_mut(
			"resumeSequence",
			|_: &Lua, this: &mut Self, id: LSequenceId| {
				this.0
					.resume_sequence(id.0)
					.map_err(|error| LuaError::external(error))
			},
		);

		methods.add_method_mut(
			"stopSequence",
			|_: &Lua, this: &mut Self, id: LSequenceId| {
				this.0
					.stop_sequence(id.0)
					.map_err(|error| LuaError::external(error))
			},
		);

		methods.add_method_mut(
			"pauseSequenceAndInstances",
			|_: &Lua, this: &mut Self, (id, fade_tween): (LSequenceId, Option<LTween>)| {
				this.0
					.pause_sequence_and_instances(id.0, fade_tween.map(|tween| tween.0))
					.map_err(|error| LuaError::external(error))
			},
		);

		methods.add_method_mut(
			"resumeSequenceAndInstances",
			|_: &Lua, this: &mut Self, (id, fade_tween): (LSequenceId, Option<LTween>)| {
				this.0
					.resume_sequence_and_instances(id.0, fade_tween.map(|tween| tween.0))
					.map_err(|error| LuaError::external(error))
			},
		);

		methods.add_method_mut(
			"stopSequenceAndInstances",
			|_: &Lua, this: &mut Self, (id, fade_tween): (LSequenceId, Option<LTween>)| {
				this.0
					.stop_sequence_and_instances(id.0, fade_tween.map(|tween| tween.0))
					.map_err(|error| LuaError::external(error))
			},
		);

		methods.add_method_mut("getEvents", |lua: &Lua, this: &mut Self, _: ()| {
			Ok(LuaValue::Table(lua.create_sequence_from(
				this.0.events().iter().map(|event| LEvent(*event)),
			)?))
		});

		methods.add_method_mut("freeUnusedResources", |_: &Lua, this: &mut Self, _: ()| {
			Ok(this.0.free_unused_resources())
		})
	}
}

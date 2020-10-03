use crate::{
	event::LCustomEventHandle,
	instance::{LInstanceId, LInstanceSettings},
	metronome::LMetronomeSettings,
	sequence::{LSequence, LSequenceId},
	sound::{LSoundId, LSoundSettings},
	tween::LTween,
};
use conductor::{
	manager::{AudioManager, AudioManagerSettings, Event},
	tempo::Tempo,
};
use mlua::prelude::*;
use std::error::Error;

pub struct LAudioManagerSettings(AudioManagerSettings);

impl LuaUserData for LAudioManagerSettings {}

impl<'lua> FromLua<'lua> for LAudioManagerSettings {
	fn from_lua(lua_value: LuaValue<'lua>, _lua: &'lua Lua) -> LuaResult<Self> {
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
				if table.contains_key("numInstances")? {
					settings.num_instances = table.get("numInstances")?;
				}
				if table.contains_key("numSequences")? {
					settings.num_sequences = table.get("numCommands")?;
				}
				if table.contains_key("metronomeSettings")? {
					let metronome_settings: LMetronomeSettings = table.get("metronomeSettings")?;
					settings.metronome_settings = metronome_settings.0;
				}
				Ok(LAudioManagerSettings(settings))
			}
			_ => panic!(),
		}
	}
}

pub struct LAudioManager(AudioManager<LCustomEventHandle>);

impl LAudioManager {
	pub fn new(settings: LAudioManagerSettings) -> Result<Self, Box<dyn Error>> {
		Ok(Self(AudioManager::new(settings.0)?))
	}
}

impl LuaUserData for LAudioManager {
	fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_method_mut(
			"loadSound",
			|_, this, (path, settings): (LuaString, LSoundSettings)| {
				let sound_id = this.0.load_sound(path.to_str()?, settings.0).unwrap();
				Ok(LSoundId(sound_id))
			},
		);

		methods.add_method_mut(
			"playSound",
			|_, this, (id, settings): (LSoundId, LInstanceSettings)| {
				let instance_id = this.0.play_sound(id.0, settings.0).unwrap();
				Ok(LInstanceId(instance_id))
			},
		);

		methods.add_method_mut(
			"setInstanceVolume",
			|_, this, (id, volume, tween): (LInstanceId, f64, Option<LTween>)| {
				this.0
					.set_instance_volume(
						id.0,
						volume,
						match tween {
							Some(tween) => Some(tween.0),
							None => None,
						},
					)
					.unwrap();
				Ok(())
			},
		);

		methods.add_method_mut(
			"setInstancePitch",
			|_, this, (id, pitch, tween): (LInstanceId, f64, Option<LTween>)| {
				this.0
					.set_instance_pitch(
						id.0,
						pitch,
						match tween {
							Some(tween) => Some(tween.0),
							None => None,
						},
					)
					.unwrap();
				Ok(())
			},
		);

		methods.add_method_mut(
			"pauseInstance",
			|_, this, (id, fade_tween): (LInstanceId, Option<LTween>)| {
				this.0
					.pause_instance(
						id.0,
						match fade_tween {
							Some(tween) => Some(tween.0),
							None => None,
						},
					)
					.unwrap();
				Ok(())
			},
		);

		methods.add_method_mut(
			"resumeInstance",
			|_, this, (id, fade_tween): (LInstanceId, Option<LTween>)| {
				this.0
					.resume_instance(
						id.0,
						match fade_tween {
							Some(tween) => Some(tween.0),
							None => None,
						},
					)
					.unwrap();
				Ok(())
			},
		);

		methods.add_method_mut(
			"stopInstance",
			|_, this, (id, fade_tween): (LInstanceId, Option<LTween>)| {
				this.0
					.stop_instance(
						id.0,
						match fade_tween {
							Some(tween) => Some(tween.0),
							None => None,
						},
					)
					.unwrap();
				Ok(())
			},
		);

		methods.add_method_mut(
			"pauseInstancesOfSound",
			|_, this, (id, fade_tween): (LSoundId, Option<LTween>)| {
				this.0
					.pause_instances_of_sound(
						id.0,
						match fade_tween {
							Some(tween) => Some(tween.0),
							None => None,
						},
					)
					.unwrap();
				Ok(())
			},
		);

		methods.add_method_mut(
			"resumeInstancesOfSound",
			|_, this, (id, fade_tween): (LSoundId, Option<LTween>)| {
				this.0
					.resume_instances_of_sound(
						id.0,
						match fade_tween {
							Some(tween) => Some(tween.0),
							None => None,
						},
					)
					.unwrap();
				Ok(())
			},
		);

		methods.add_method_mut(
			"stopInstancesOfSound",
			|_, this, (id, fade_tween): (LSoundId, Option<LTween>)| {
				this.0
					.stop_instances_of_sound(
						id.0,
						match fade_tween {
							Some(tween) => Some(tween.0),
							None => None,
						},
					)
					.unwrap();
				Ok(())
			},
		);

		methods.add_method_mut("setMetronomeTempo", |_, this, tempo: f64| {
			this.0.set_metronome_tempo(Tempo(tempo)).unwrap();
			Ok(())
		});

		methods.add_method_mut("startMetronome", |_, this, _: ()| {
			this.0.start_metronome().unwrap();
			Ok(())
		});

		methods.add_method_mut("pauseMetronome", |_, this, _: ()| {
			this.0.pause_metronome().unwrap();
			Ok(())
		});

		methods.add_method_mut("stopMetronome", |_, this, _: ()| {
			this.0.stop_metronome().unwrap();
			Ok(())
		});

		methods.add_method_mut("startSequence", |_, this, sequence: LSequence| {
			let id = this.0.start_sequence(sequence.0).unwrap();
			Ok(LSequenceId(id))
		});

		methods.add_method_mut("muteSequence", |_, this, id: LSequenceId| {
			this.0.mute_sequence(id.0).unwrap();
			Ok(())
		});

		methods.add_method_mut("unmuteSequence", |_, this, id: LSequenceId| {
			this.0.unmute_sequence(id.0).unwrap();
			Ok(())
		});

		methods.add_method_mut("getEvents", |_, this, callbacks: Option<LuaTable>| {
			for event in this.0.events() {
				if let Some(callbacks) = &callbacks {
					match event {
						Event::MetronomeIntervalPassed(interval) => {
							if let Some(callback) = callbacks
								.get::<&str, Option<LuaFunction>>("metronomeIntervalPassed")?
							{
								callback.call(interval)?;
							}
						}
						Event::Custom(handle) => {
							if let Some(callback) =
								callbacks.get::<&str, Option<LuaFunction>>("custom")?
							{
								callback.call(handle)?;
							}
						}
					}
				}
			}
			Ok(())
		});

		methods.add_method_mut("freeUnusedResources", |_, this, _: ()| {
			this.0.free_unused_resources();
			Ok(())
		})
	}
}

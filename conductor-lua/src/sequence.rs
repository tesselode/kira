use crate::{
	duration::{DurationUnit, LDuration},
	event::LCustomEventHandle,
	instance::LInstanceSettings,
	sound::LSoundId,
	tween::LTween,
};
use conductor::sequence::{Sequence, SequenceId, SequenceInstanceHandle};
use mlua::prelude::*;

#[derive(Clone)]
pub struct LSequenceId(pub SequenceId);

impl LuaUserData for LSequenceId {}

#[derive(Clone)]
pub struct LSequenceInstanceHandle(SequenceInstanceHandle);

impl LuaUserData for LSequenceInstanceHandle {}

#[derive(Clone)]
pub struct LSequence(pub Sequence<LCustomEventHandle>);

impl LSequence {
	pub fn new() -> Self {
		Self(Sequence::new())
	}
}

impl LuaUserData for LSequence {
	fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_method_mut("wait", |_, this, (value, unit): (f64, DurationUnit)| {
			let duration = LDuration::new(value, unit);
			this.0.wait(duration.0);
			Ok(())
		});

		methods.add_method_mut("waitForInterval", |_, this, interval: f64| {
			this.0.wait_for_interval(interval);
			Ok(())
		});

		methods.add_method_mut("startLoop", |_, this, _: ()| {
			this.0.start_loop();
			Ok(())
		});

		methods.add_method_mut(
			"playSound",
			|_, this, (id, settings): (LSoundId, LInstanceSettings)| {
				let handle = this.0.play_sound(id.0, settings.0);
				Ok(LSequenceInstanceHandle(handle))
			},
		);

		methods.add_method_mut(
			"setInstanceVolume",
			|_, this, (handle, volume, tween): (LSequenceInstanceHandle, f64, Option<LTween>)| {
				this.0.set_instance_volume(
					handle.0,
					volume,
					match tween {
						Some(tween) => Some(tween.0),
						None => None,
					},
				);
				Ok(())
			},
		);

		methods.add_method_mut(
			"setInstancePitch",
			|_, this, (handle, pitch, tween): (LSequenceInstanceHandle, f64, Option<LTween>)| {
				this.0.set_instance_pitch(
					handle.0,
					pitch,
					match tween {
						Some(tween) => Some(tween.0),
						None => None,
					},
				);
				Ok(())
			},
		);

		methods.add_method_mut(
			"pauseInstance",
			|_, this, (handle, fade_tween): (LSequenceInstanceHandle, Option<LTween>)| {
				this.0.pause_instance(
					handle.0,
					match fade_tween {
						Some(tween) => Some(tween.0),
						None => None,
					},
				);
				Ok(())
			},
		);

		methods.add_method_mut(
			"resumeInstance",
			|_, this, (handle, fade_tween): (LSequenceInstanceHandle, Option<LTween>)| {
				this.0.resume_instance(
					handle.0,
					match fade_tween {
						Some(tween) => Some(tween.0),
						None => None,
					},
				);
				Ok(())
			},
		);

		methods.add_method_mut(
			"stopInstance",
			|_, this, (handle, fade_tween): (LSequenceInstanceHandle, Option<LTween>)| {
				this.0.stop_instance(
					handle.0,
					match fade_tween {
						Some(tween) => Some(tween.0),
						None => None,
					},
				);
				Ok(())
			},
		);

		methods.add_method_mut(
			"pauseInstancesOfSound",
			|_, this, (id, fade_tween): (LSoundId, Option<LTween>)| {
				this.0.pause_instances_of_sound(
					id.0,
					match fade_tween {
						Some(tween) => Some(tween.0),
						None => None,
					},
				);
				Ok(())
			},
		);

		methods.add_method_mut(
			"resumeInstancesOfSound",
			|_, this, (id, fade_tween): (LSoundId, Option<LTween>)| {
				this.0.resume_instances_of_sound(
					id.0,
					match fade_tween {
						Some(tween) => Some(tween.0),
						None => None,
					},
				);
				Ok(())
			},
		);

		methods.add_method_mut(
			"stopInstancesOfSound",
			|_, this, (id, fade_tween): (LSoundId, Option<LTween>)| {
				this.0.stop_instances_of_sound(
					id.0,
					match fade_tween {
						Some(tween) => Some(tween.0),
						None => None,
					},
				);
				Ok(())
			},
		);

		methods.add_method_mut("startMetronome", |_, this, _: ()| {
			this.0.start_metronome();
			Ok(())
		});

		methods.add_method_mut("pauseMetronome", |_, this, _: ()| {
			this.0.pause_metronome();
			Ok(())
		});

		methods.add_method_mut("stopMetronome", |_, this, _: ()| {
			this.0.stop_metronome();
			Ok(())
		});

		methods.add_method_mut("emitCustomEvent", |_, this, handle: LCustomEventHandle| {
			this.0.emit_custom_event(handle);
			Ok(())
		})
	}
}

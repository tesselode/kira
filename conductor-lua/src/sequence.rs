use crate::{
	duration::{DurationUnit, LDuration},
	instance::LInstanceSettings,
	sound::LSoundId,
};
use conductor::sequence::{Sequence, SequenceId, SequenceInstanceHandle};
use mlua::prelude::*;

pub struct LSequenceId(pub SequenceId);

impl LuaUserData for LSequenceId {}

pub struct LSequenceInstanceHandle(SequenceInstanceHandle);

impl LuaUserData for LSequenceInstanceHandle {}

#[derive(Clone)]
pub struct LSequence(pub Sequence);

impl LSequence {
	pub fn new() -> Self {
		Self(Sequence::new())
	}
}

impl LuaUserData for LSequence {
	fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_method_mut("wait", |_, this, (value, unit): (f32, DurationUnit)| {
			let duration = LDuration::new(value, unit);
			this.0.wait(duration.0);
			Ok(())
		});

		methods.add_method_mut("waitForInterval", |_, this, interval: f32| {
			this.0.wait_for_interval(interval);
			Ok(())
		});

		methods.add_method_mut("goTo", |_, this, index: usize| {
			this.0.go_to(index);
			Ok(())
		});

		methods.add_method_mut(
			"playSound",
			|_, this, (id, settings): (LSoundId, LInstanceSettings)| {
				let handle = this.0.play_sound(id.0, settings.0);
				Ok(LSequenceInstanceHandle(handle))
			},
		);
	}
}

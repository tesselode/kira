mod handle;

pub use handle::*;

use std::sync::{
	atomic::{AtomicBool, Ordering},
	Arc,
};

use glam::{Quat, Vec3};

use crate::{
	arena::Key,
	command::{read_commands_into_parameters, ValueChangeCommand},
	command_writers_and_readers,
	info::Info,
	tween::{Parameter, Value},
};

/// A unique identifier for a listener.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ListenerId(pub(crate) Key);

pub(crate) struct Listener {
	pub shared: Arc<ListenerShared>,
	pub position: Parameter<Vec3>,
	pub orientation: Parameter<Quat>,
	pub command_readers: CommandReaders,
}

impl Listener {
	pub fn new(
		id: ListenerId,
		position: Value<Vec3>,
		orientation: Value<Quat>,
	) -> (Self, ListenerHandle) {
		let shared = Arc::new(ListenerShared::new());
		let (command_writers, command_readers) = command_writers_and_readers();
		(
			Self {
				shared: shared.clone(),
				position: Parameter::new(position, Vec3::ZERO),
				orientation: Parameter::new(orientation, Quat::IDENTITY),
				command_readers,
			},
			ListenerHandle {
				id,
				shared,
				command_writers,
			},
		)
	}

	pub fn on_start_processing(&mut self) {
		read_commands_into_parameters!(self, position, orientation);
	}

	pub(crate) fn update(&mut self, dt: f64, info: &Info) {
		self.position.update(dt, info);
		self.orientation.update(dt, info);
	}
}

impl Default for Listener {
	fn default() -> Self {
		let (_, command_readers) = command_writers_and_readers();
		Self {
			shared: Arc::new(ListenerShared::new()),
			position: Parameter::new(Value::Fixed(Vec3::ZERO), Vec3::ZERO),
			orientation: Parameter::new(Value::Fixed(Quat::IDENTITY), Quat::IDENTITY),
			command_readers,
		}
	}
}

#[derive(Debug)]
pub(crate) struct ListenerShared {
	removed: AtomicBool,
}

impl ListenerShared {
	pub fn new() -> Self {
		Self {
			removed: AtomicBool::new(false),
		}
	}

	#[must_use]
	pub fn is_marked_for_removal(&self) -> bool {
		self.removed.load(Ordering::SeqCst)
	}

	pub fn mark_for_removal(&self) {
		self.removed.store(true, Ordering::SeqCst);
	}
}

command_writers_and_readers! {
	set_position: ValueChangeCommand<Vec3>,
	set_orientation: ValueChangeCommand<Quat>,
}

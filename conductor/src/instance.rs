use crate::sound::SoundId;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_INSTANCE_INDEX: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct InstanceId {
	index: usize,
}

impl InstanceId {
	pub fn new() -> Self {
		let index = NEXT_INSTANCE_INDEX.fetch_add(1, Ordering::Relaxed);
		Self { index }
	}
}

#[derive(Debug, Copy, Clone)]
pub struct InstanceSettings {
	pub volume: f32,
	pub pitch: f32,
}

impl Default for InstanceSettings {
	fn default() -> Self {
		Self {
			volume: 1.0,
			pitch: 1.0,
		}
	}
}

pub(crate) struct Instance {
	pub sound_id: SoundId,
	pub volume: f32,
	pub pitch: f32,
	pub position: f32,
}

impl Instance {
	pub fn new(sound_id: SoundId, settings: InstanceSettings) -> Self {
		Self {
			sound_id,
			volume: settings.volume,
			pitch: settings.pitch,
			position: 0.0,
		}
	}
}

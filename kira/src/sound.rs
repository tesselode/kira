//! A chunk of audio data.

use atomig::{Atomic, Ordering};

use crate::Frame;

use self::data::SoundData;

pub mod data;
pub mod handle;
pub mod instance;
pub mod settings;

pub(crate) struct Sound {
	data: Box<dyn SoundData>,
	loop_start: Option<f64>,
	semantic_duration: Option<f64>,
	cooldown: f64,
	cooldown_timer: Atomic<f64>,
}

impl Sound {
	pub fn new(
		data: impl SoundData + 'static,
		loop_start: Option<f64>,
		semantic_duration: Option<f64>,
		cooldown: f64,
	) -> Self {
		Self {
			data: Box::new(data),
			loop_start,
			semantic_duration,
			cooldown,
			cooldown_timer: Atomic::new(0.0),
		}
	}

	pub fn duration(&self) -> f64 {
		self.data.duration()
	}

	pub fn loop_start(&self) -> Option<f64> {
		self.loop_start
	}

	pub fn semantic_duration(&self) -> Option<f64> {
		self.semantic_duration
	}

	pub fn start_cooldown(&self) {
		self.cooldown_timer.store(self.cooldown, Ordering::SeqCst);
	}

	pub fn cooling_down(&self) -> bool {
		self.cooldown_timer.load(Ordering::SeqCst) > 0.0
	}

	pub fn update(&self, dt: f64) {
		let mut cooldown_timer = self.cooldown_timer.load(Ordering::SeqCst);
		cooldown_timer -= dt;
		cooldown_timer = cooldown_timer.max(0.0);
		self.cooldown_timer.store(cooldown_timer, Ordering::SeqCst);
	}

	pub fn frame_at_position(&self, position: f64) -> Frame {
		self.data.frame_at_position(position)
	}
}

#[cfg(feature = "log_drops")]
impl Drop for Sound {
	fn drop(&mut self) {
		println!("dropped Sound on thread {:?}", std::thread::current().id());
	}
}

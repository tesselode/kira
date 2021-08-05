use std::{sync::Arc, time::Duration};

use crate::frame::Frame;

use super::Sound;

// TODO: explain how this works

struct Section {
	sound: Arc<dyn Sound>,
	loop_end: f64,
}

pub struct SeamlessLoop {
	main: Section,
	intro: Option<Section>,
}

impl SeamlessLoop {
	pub fn new(sound: impl Into<Arc<dyn Sound>>, loop_end: f64) -> Self {
		Self {
			main: Section {
				sound: sound.into(),
				loop_end,
			},
			intro: None,
		}
	}

	pub fn with_intro(
		intro_sound: impl Into<Arc<dyn Sound>>,
		intro_loop_end: f64,
		main_sound: impl Into<Arc<dyn Sound>>,
		main_loop_end: f64,
	) -> Self {
		Self {
			main: Section {
				sound: main_sound.into(),
				loop_end: main_loop_end,
			},
			intro: Some(Section {
				sound: intro_sound.into(),
				loop_end: intro_loop_end,
			}),
		}
	}
}

impl Sound for SeamlessLoop {
	fn duration(&self) -> Duration {
		Duration::from_secs_f64(if let Some(intro) = &self.intro {
			intro.loop_end + self.main.loop_end * 2.0
		} else {
			self.main.loop_end * 2.0
		})
	}

	fn frame_at_position(&self, mut position: f64) -> Frame {
		let mut out = Frame::from_mono(0.0);
		if let Some(intro) = &self.intro {
			out += intro.sound.frame_at_position(position);
			position -= intro.loop_end;
		}
		if position >= 0.0 {
			out += self.main.sound.frame_at_position(position);
			position -= self.main.loop_end;
		}
		if position >= 0.0 {
			out += self.main.sound.frame_at_position(position);
		}
		out
	}

	fn default_loop_start(&self) -> Option<f64> {
		Some(if let Some(intro) = &self.intro {
			intro.loop_end + self.main.loop_end
		} else {
			self.main.loop_end
		})
	}
}

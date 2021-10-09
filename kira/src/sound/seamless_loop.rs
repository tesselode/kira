use std::{time::Duration};

use crate::{frame::Frame, loop_behavior::LoopBehavior};

use super::Sound;

// TODO: explain how this works

struct Section {
	sound: Box<dyn Sound>,
	loop_end: f64,
}

/// Wraps sounds to create seamless loops that preserve leftover
/// audio after the loop point.
pub struct SeamlessLoop {
	main: Section,
	intro: Option<Section>,
}

impl SeamlessLoop {
	/// Creates a new [`SeamlessLoop`] that loops one sound.
	pub fn new(sound: impl Sound + 'static, loop_end: f64) -> Self {
		Self {
			main: Section {
				sound: Box::new(sound),
				loop_end,
			},
			intro: None,
		}
	}

	/// Creates a new [`SeamlessLoop`] with an intro section and a looping
	/// section.
	pub fn with_intro(
		intro_sound: impl Sound + 'static,
		intro_loop_end: f64,
		main_sound: impl Sound + 'static,
		main_loop_end: f64,
	) -> Self {
		Self {
			main: Section {
				sound: Box::new(main_sound),
				loop_end: main_loop_end,
			},
			intro: Some(Section {
				sound: Box::new(intro_sound),
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
		let mut out = Frame::ZERO;
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

	fn default_loop_behavior(&self) -> Option<LoopBehavior> {
		Some(if let Some(intro) = &self.intro {
			LoopBehavior {
				start_position: intro.loop_end + self.main.loop_end,
			}
		} else {
			LoopBehavior {
				start_position: self.main.loop_end,
			}
		})
	}
}

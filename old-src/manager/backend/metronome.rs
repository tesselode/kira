#[derive(Eq, PartialEq)]
enum MetronomeState {
	Stopped,
	Ticking,
}

pub struct Metronome {
	pub tempo: f32,
	state: MetronomeState,
	position: f32,
	previous_position: f32,
}

impl Metronome {
	pub fn new(tempo: f32) -> Self {
		Self {
			tempo,
			state: MetronomeState::Stopped,
			position: 0.0,
			previous_position: 0.0,
		}
	}

	pub fn start(&mut self) {
		self.state = MetronomeState::Ticking;
	}

	pub fn update(&mut self, dt: f32) {
		if self.state == MetronomeState::Ticking {
			self.previous_position = self.position;
			self.position += (self.tempo / 60.0) * dt;
		}
	}

	pub fn interval_passed(&self, interval: f32) -> bool {
		self.state == MetronomeState::Ticking
			&& (self.previous_position % interval) > (self.position % interval)
	}
}

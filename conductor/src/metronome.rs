pub struct Metronome {
	tempo: f32,
	ticking: bool,
	time: f32,
	previous_time: f32,
}

impl Metronome {
	pub fn new(tempo: f32) -> Self {
		Self {
			tempo,
			ticking: false,
			time: 0.0,
			previous_time: 0.0,
		}
	}

	pub fn start(&mut self) {
		self.ticking = true;
	}

	pub fn pause(&mut self) {
		self.ticking = false;
	}

	pub fn stop(&mut self) {
		self.ticking = false;
		self.time = 0.0;
		self.previous_time = 0.0;
	}

	pub fn update(&mut self, dt: f32) {
		if !self.ticking {
			return;
		}
		self.previous_time = self.time;
		self.time += (self.tempo / 60.0) * dt;
		if self.interval_passed(1.0) {
			println!("beat");
		}
	}

	pub fn interval_passed(&self, interval: f32) -> bool {
		(self.previous_time % interval) > (self.time % interval)
	}
}

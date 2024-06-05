use std::sync::{
	atomic::{AtomicBool, Ordering},
	Arc, Mutex,
};

/// An event that can be fired to start other events.
#[derive(Debug, Clone)]
pub struct Trigger(Arc<AtomicBool>);

impl Trigger {
	/// Creates a new trigger.
	pub fn new() -> Self {
		let mut triggers = TRIGGERS.lock().unwrap();
		triggers.retain(|trigger| Arc::strong_count(&trigger.0) > 1);
		let trigger = Self(Arc::new(AtomicBool::new(false)));
		triggers.push(trigger.clone());
		trigger
	}

	/// Activates the trigger, starting any events waiting on the trigger.
	pub fn fire(&self) {
		self.0.store(true, Ordering::SeqCst);
	}

	/// Returns whether the trigger has already been activated.
	pub fn fired(&self) -> bool {
		self.0.load(Ordering::SeqCst)
	}
}

impl Default for Trigger {
	fn default() -> Self {
		Self::new()
	}
}

static TRIGGERS: Mutex<Vec<Trigger>> = Mutex::new(vec![]);

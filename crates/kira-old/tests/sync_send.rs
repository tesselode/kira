use kira_old::manager::{backend::cpal::CpalBackend, AudioManager};

fn main() {
	sync_send::<AudioManager<CpalBackend>>()
}

fn sync_send<T: Sync + Send>() {}

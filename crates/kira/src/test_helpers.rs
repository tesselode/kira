use crate::{Frame, info::MockInfoBuilder, sound::Sound};

pub fn expect_frame_soon(expected_frame: Frame, sound: &mut dyn Sound) {
	const NUM_SAMPLES_TO_WAIT: usize = 10;
	let mut collected_samples = vec![];
	for _ in 0..NUM_SAMPLES_TO_WAIT {
		let frame = sound.process_one(1.0, &MockInfoBuilder::new().build());
		if frame == expected_frame {
			return;
		}
		collected_samples.push(frame);
	}
	panic!(
		"Sound did not output frame with value {:?} within {} samples. Recent samples: {:#?}",
		expected_frame, NUM_SAMPLES_TO_WAIT, collected_samples
	);
}

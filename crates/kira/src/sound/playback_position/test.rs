use super::PlaybackPosition;

#[test]
fn into_samples() {
	const SAMPLE_RATE: u32 = 44100;
	const SECONDS_TO_SAMPLES: &[(f64, usize)] = &[(8.786054, 387465), (0.061519, 2713)];
	for (seconds, samples) in SECONDS_TO_SAMPLES {
		assert_eq!(
			PlaybackPosition::Seconds(*seconds).into_samples(SAMPLE_RATE),
			*samples
		);
	}
}

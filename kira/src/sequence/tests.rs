use super::{super::Duration, Sequence, SequenceError};

#[test]
fn fails_validation_on_infinite_loop() {
	let non_looping_sequence = Sequence::<()>::new(Default::default());
	assert!(
		non_looping_sequence.validate().is_ok(),
		"Sequences without loop points should pass validation"
	);

	let valid_looping_sequence = {
		let mut sequence = Sequence::<()>::new(Default::default());
		sequence.start_loop();
		sequence.wait(Duration::Seconds(0.1));
		sequence
	};
	assert!(
		valid_looping_sequence.validate().is_ok(),
		"Looping sequences with waits in the loop section should pass validation"
	);

	let infinitely_looping_sequence = {
		let mut sequence = Sequence::<()>::new(Default::default());
		sequence.start_loop();
		sequence.emit(());
		sequence
	};
	if let Err(SequenceError::InfiniteLoop) = infinitely_looping_sequence.validate() {
	} else {
		panic!("Sequences with infinite loops should fail validation");
	}
}

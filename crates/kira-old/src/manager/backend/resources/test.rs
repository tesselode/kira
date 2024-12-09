use crate::{manager::backend::resources::SelfReferentialResourceStorage, ResourceLimitReached};

use super::ResourceStorage;

#[test]
fn resource_storage() {
	let (mut storage, mut controller) = ResourceStorage::new(2);

	// add
	let one = controller.insert(1).unwrap();
	let two = controller.insert(2).unwrap();
	assert_eq!(controller.insert(3), Err(ResourceLimitReached));
	storage.remove_and_add(|_| false);

	// get
	assert_eq!(storage.get_mut(one), Some(&mut 1));
	assert_eq!(storage.get_mut(two), Some(&mut 2));

	// iter
	assert_eq!(
		storage.iter_mut().collect::<Vec<_>>(),
		vec![(two, &mut 2), (one, &mut 1)]
	);

	// remove
	storage.remove_and_add(|&x| x < 2);
	assert_eq!(storage.get_mut(one), None);
	assert_eq!(storage.get_mut(two), Some(&mut 2));
	assert_eq!(storage.iter_mut().collect::<Vec<_>>(), vec![(two, &mut 2)]);

	// re-add
	let three = controller.insert(3).unwrap();
	assert_eq!(controller.insert(4), Err(ResourceLimitReached));
	storage.remove_and_add(|_| false);
	assert_eq!(storage.get_mut(two), Some(&mut 2));
	assert_eq!(storage.get_mut(three), Some(&mut 3));
	assert_eq!(
		storage.iter_mut().collect::<Vec<_>>(),
		vec![(three, &mut 3), (two, &mut 2)]
	);
}

#[test]
fn self_referential_resource_storage() {
	let (mut storage, mut controller) = SelfReferentialResourceStorage::new(2);

	// add
	let one = controller.insert(1).unwrap();
	let two = controller.insert(2).unwrap();
	assert_eq!(controller.insert(3), Err(ResourceLimitReached));
	storage.remove_and_add(|_| false);

	// get
	assert_eq!(storage.get_mut(one), Some(&mut 1));
	assert_eq!(storage.get_mut(two), Some(&mut 2));

	// iter
	assert_eq!(
		storage.iter_mut().collect::<Vec<_>>(),
		vec![(two, &mut 2), (one, &mut 1)]
	);

	// remove
	storage.remove_and_add(|&x| x < 2);
	assert_eq!(storage.get_mut(one), None);
	assert_eq!(storage.get_mut(two), Some(&mut 2));
	assert_eq!(storage.iter_mut().collect::<Vec<_>>(), vec![(two, &mut 2)]);

	// re-add
	let three = controller.insert(3).unwrap();
	assert_eq!(controller.insert(4), Err(ResourceLimitReached));
	storage.remove_and_add(|_| false);
	assert_eq!(storage.get_mut(two), Some(&mut 2));
	assert_eq!(storage.get_mut(three), Some(&mut 3));
	assert_eq!(
		storage.iter_mut().collect::<Vec<_>>(),
		vec![(three, &mut 3), (two, &mut 2)]
	);
}

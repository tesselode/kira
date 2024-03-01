use crate::arena::{
	error::{ArenaFull, InsertWithKeyError},
	Arena,
};

#[test]
fn controller() {
	let arena = Arena::<()>::new(1);
	let controller1 = arena.controller();
	controller1.try_reserve().unwrap();
	// controllers should share state
	let controller2 = arena.controller();
	assert_eq!(controller2.try_reserve(), Err(ArenaFull));
}

#[test]
fn try_reserve() {
	let arena = Arena::<()>::new(3);
	let controller = arena.controller();
	// we should be able to reserve 3 indices
	// because the capacity is 3
	assert!(controller.try_reserve().is_ok());
	assert!(controller.try_reserve().is_ok());
	assert!(controller.try_reserve().is_ok());
	// we should not be able to reserve a 4th key
	assert_eq!(controller.try_reserve(), Err(ArenaFull));
}

#[test]
fn capacity() {
	let mut arena = Arena::new(3);
	assert_eq!(arena.capacity(), 3);
	// the capacity of the arena should be constant
	arena.insert(1).unwrap();
	assert_eq!(arena.capacity(), 3);
	let key2 = arena.insert(2).unwrap();
	assert_eq!(arena.capacity(), 3);
	arena.remove(key2);
	assert_eq!(arena.capacity(), 3);
}

#[test]
fn len() {
	let mut arena = Arena::new(3);
	arena.insert(1).unwrap();
	assert_eq!(arena.len(), 1);
	arena.insert(2).unwrap();
	assert_eq!(arena.len(), 2);
	let key3 = arena.insert(3).unwrap();
	assert_eq!(arena.len(), 3);
	// if inserting an element fails, the length should not
	// increase
	arena.insert(4).ok();
	assert_eq!(arena.len(), 3);
	arena.remove(key3);
	assert_eq!(arena.len(), 2);
	// if removing an element fails, the length should not
	// decrease
	arena.remove(key3);
	assert_eq!(arena.len(), 2);
}

#[test]
fn controller_capacity() {
	let mut arena = Arena::new(3);
	let controller = arena.controller();
	assert_eq!(controller.capacity(), 3);
	controller.try_reserve().unwrap();
	assert_eq!(controller.capacity(), 3);
	let key = arena.insert(()).unwrap();
	assert_eq!(controller.capacity(), 3);
	arena.remove(key);
	assert_eq!(controller.capacity(), 3);
}

#[test]
fn controller_len() {
	let mut arena = Arena::new(3);
	let controller = arena.controller();
	assert_eq!(controller.len(), 0);
	controller.try_reserve().unwrap();
	assert_eq!(controller.len(), 1);
	let key = arena.insert(()).unwrap();
	assert_eq!(controller.len(), 2);
	arena.remove(key);
	assert_eq!(controller.len(), 1);
}

#[test]
fn insert_with_key() {
	let mut arena = Arena::new(3);
	let controller = arena.controller();
	let key = controller.try_reserve().unwrap();
	// we should be able to insert with the key we reserved
	assert!(arena.insert_with_key(key, 1).is_ok());
	// the item should be in the arena
	assert_eq!(arena.get(key), Some(&1));
	// we should not be able to insert again with the same key
	assert_eq!(
		arena.insert_with_key(key, 2),
		Err(InsertWithKeyError::KeyNotReserved)
	);
}

#[test]
fn insert_with_invalid_key_index() {
	let key = {
		let mut arena = Arena::new(5);
		for _ in 0..4 {
			arena.insert(()).unwrap();
		}
		arena.insert(()).unwrap()
	};
	let mut arena = Arena::new(3);
	assert_eq!(
		arena.insert_with_key(key, ()),
		Err(InsertWithKeyError::InvalidKey)
	);
}

#[test]
fn insert_with_invalid_key_generation() {
	let key = {
		let mut arena = Arena::new(1);
		let key = arena.insert(()).unwrap();
		arena.remove(key);
		arena.insert(()).unwrap()
	};
	let mut arena = Arena::new(1);
	assert_eq!(
		arena.insert_with_key(key, ()),
		Err(InsertWithKeyError::InvalidKey)
	);
}

#[test]
fn insert() {
	let mut arena = Arena::new(3);
	// we should be able to insert 3 items
	let key1 = arena.insert(1).unwrap();
	let key2 = arena.insert(2).unwrap();
	let key3 = arena.insert(3).unwrap();
	// we should be able to retrieve those items with the
	// returned indices
	assert_eq!(arena.get(key1), Some(&1));
	assert_eq!(arena.get(key2), Some(&2));
	assert_eq!(arena.get(key3), Some(&3));
	// we should not be able to insert a 4th item
	assert_eq!(arena.insert(4), Err(ArenaFull));
}

#[test]
fn remove() {
	let mut arena = Arena::new(3);
	let key1 = arena.insert(1).unwrap();
	let key2 = arena.insert(2).unwrap();
	let key3 = arena.insert(3).unwrap();
	// we should be able to remove an item and get it back
	assert_eq!(arena.remove(key2), Some(2));
	// if there's no item associated with the key,
	// `remove` should return `None`
	assert_eq!(arena.remove(key2), None);
	// the other items should still be in the arena
	assert_eq!(arena.get(key1), Some(&1));
	assert_eq!(arena.get(key3), Some(&3));
	// there should be space to insert another item now
	assert!(arena.insert(4).is_ok());
	// we shouldn't be able to remove the new item in the
	// same slot with an old key
	assert_eq!(arena.remove(key2), None);
}

#[test]
fn get() {
	let mut arena = Arena::new(3);
	let key1 = arena.insert(1).unwrap();
	let key2 = arena.insert(2).unwrap();
	let key3 = arena.insert(3).unwrap();
	// get should return shared references
	assert_eq!(arena.get(key1), Some(&1));
	assert_eq!(arena.get(key2), Some(&2));
	assert_eq!(arena.get(key3), Some(&3));
	// get_mut should return mutable references
	assert_eq!(arena.get_mut(key1), Some(&mut 1));
	assert_eq!(arena.get_mut(key2), Some(&mut 2));
	assert_eq!(arena.get_mut(key3), Some(&mut 3));
	// after removing an item, get should return None
	arena.remove(key2);
	assert_eq!(arena.get(key2), None);
	// even after inserting a new item into the same slot,
	// the old key shouldn't work
	arena.insert(4).unwrap();
	assert_eq!(arena.get(key2), None);
}

#[test]
fn retain() {
	let mut arena = Arena::new(6);
	let key1 = arena.insert(1).unwrap();
	let key2 = arena.insert(2).unwrap();
	let key3 = arena.insert(3).unwrap();
	let key4 = arena.insert(4).unwrap();
	let key5 = arena.insert(5).unwrap();
	let key6 = arena.insert(6).unwrap();
	arena.retain(|num| num % 2 == 0);
	assert_eq!(arena.get(key1), None);
	assert_eq!(arena.get(key2), Some(&2));
	assert_eq!(arena.get(key3), None);
	assert_eq!(arena.get(key4), Some(&4));
	assert_eq!(arena.get(key5), None);
	assert_eq!(arena.get(key6), Some(&6));
}

#[test]
fn iter() {
	let mut arena = Arena::new(3);
	let key1 = arena.insert(1).unwrap();
	let key2 = arena.insert(2).unwrap();
	let key3 = arena.insert(3).unwrap();
	// iterators should visit all values
	let mut iter = arena.iter();
	assert_eq!(iter.next(), Some((key3, &3)));
	assert_eq!(iter.next(), Some((key2, &2)));
	assert_eq!(iter.next(), Some((key1, &1)));
	assert_eq!(iter.next(), None);
	// iterators should not visit removed values
	arena.remove(key2);
	let mut iter = arena.iter();
	assert_eq!(iter.next(), Some((key3, &3)));
	assert_eq!(iter.next(), Some((key1, &1)));
	assert_eq!(iter.next(), None);
	// iteration should always be newest first
	let key4 = arena.insert(4).unwrap();
	let mut iter = arena.iter();
	assert_eq!(iter.next(), Some((key4, &4)));
	assert_eq!(iter.next(), Some((key3, &3)));
	assert_eq!(iter.next(), Some((key1, &1)));
	assert_eq!(iter.next(), None);
}

#[test]
fn iter_mut() {
	let mut arena = Arena::new(3);
	let key1 = arena.insert(1).unwrap();
	let key2 = arena.insert(2).unwrap();
	let key3 = arena.insert(3).unwrap();
	// iterators should visit all values
	let mut iter = arena.iter_mut();
	assert_eq!(iter.next(), Some((key3, &mut 3)));
	assert_eq!(iter.next(), Some((key2, &mut 2)));
	assert_eq!(iter.next(), Some((key1, &mut 1)));
	assert_eq!(iter.next(), None);
	// iterators should not visit removed values
	arena.remove(key2);
	let mut iter = arena.iter_mut();
	assert_eq!(iter.next(), Some((key3, &mut 3)));
	assert_eq!(iter.next(), Some((key1, &mut 1)));
	assert_eq!(iter.next(), None);
	// iteration should always be newest first
	let key4 = arena.insert(4).unwrap();
	let mut iter = arena.iter_mut();
	assert_eq!(iter.next(), Some((key4, &mut 4)));
	assert_eq!(iter.next(), Some((key3, &mut 3)));
	assert_eq!(iter.next(), Some((key1, &mut 1)));
	assert_eq!(iter.next(), None);
}

#[test]
fn drain_filter() {
	let mut arena = Arena::new(6);
	let key1 = arena.insert(1).unwrap();
	let key2 = arena.insert(2).unwrap();
	let key3 = arena.insert(3).unwrap();
	let key4 = arena.insert(4).unwrap();
	let key5 = arena.insert(5).unwrap();
	let key6 = arena.insert(6).unwrap();
	let mut iter = arena.drain_filter(|num| num % 2 == 0);
	assert_eq!(iter.next(), Some((key6, 6)));
	assert_eq!(iter.next(), Some((key4, 4)));
	assert_eq!(iter.next(), Some((key2, 2)));
	assert_eq!(iter.next(), None);
	assert_eq!(arena.len(), 3);
	assert_eq!(arena.get(key1), Some(&1));
	assert_eq!(arena.get(key2), None);
	assert_eq!(arena.get(key3), Some(&3));
	assert_eq!(arena.get(key4), None);
	assert_eq!(arena.get(key5), Some(&5));
	assert_eq!(arena.get(key6), None);
}

use std::{
	hash::Hash,
	ops::{Add, Div, Mul, Sub},
};

use indexmap::IndexSet;

pub fn lerp<T>(a: T, b: T, amount: T) -> T
where
	T: Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Copy,
{
	a + (b - a) * amount
}

pub fn inverse_lerp<T>(start: T, end: T, point: T) -> T
where
	T: Sub<Output = T> + Div<Output = T> + Copy,
{
	(point - start) / (end - start)
}

pub fn index_set_from_vec<T: Hash + Eq>(v: Vec<T>) -> IndexSet<T> {
	let mut set = IndexSet::new();
	for item in v {
		set.insert(item);
	}
	set
}

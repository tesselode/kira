use std::hash::Hash;

use indexmap::IndexSet;

pub fn lerp(a: f64, b: f64, amount: f64) -> f64 {
	a + (b - a) * amount
}

pub fn inverse_lerp(start: f64, end: f64, point: f64) -> f64 {
	(point - start) / (end - start)
}

pub fn index_set_from_vec<T: Hash + Eq>(v: Vec<T>) -> IndexSet<T> {
	let mut set = IndexSet::new();
	for item in v {
		set.insert(item);
	}
	set
}

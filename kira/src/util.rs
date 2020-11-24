use std::ops::{Add, Div, Mul, Sub};

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

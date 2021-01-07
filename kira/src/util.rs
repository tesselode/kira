use nanorand::RNG;
use uuid::Uuid;

pub fn lerp(a: f64, b: f64, amount: f64) -> f64 {
	a + (b - a) * amount
}

pub fn inverse_lerp(start: f64, end: f64, point: f64) -> f64 {
	(point - start) / (end - start)
}

pub fn generate_uuid() -> Uuid {
	let mut rng = nanorand::tls_rng();
	let mut random_bytes: [u8; 16] = [0; 16];
	for i in 0..16 {
		random_bytes[i] = rng.generate();
	}
	uuid::Builder::from_slice(&random_bytes)
		.unwrap()
		.set_variant(uuid::Variant::RFC4122)
		.set_version(uuid::Version::Random)
		.build()
}

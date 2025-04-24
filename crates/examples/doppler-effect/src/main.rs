use kira::{
	backend::DefaultBackend, sound::static_sound::StaticSoundData, track::{SpatialTrackBuilder, SpatialTrackDistances}, AudioManager, AudioManagerSettings, Decibels, Easing, Mapping, Tween, Value
};
use macroquad::prelude::*;

const CAMERA_MAX_SPEED: f32 = 160.0;
const CAMERA_ACCEL: f32 = 1000.0;
const CAMERA_DRAG: f32 = 8.0;

const LOOK_SPEED: f32 = 0.005;
const WORLD_UP: Vec3 = vec3(0.0, 1.0, 0.0);
const SPATIAL_TRACK_POSITION: Vec3 = vec3(0.0, 1.0, 6.0);
const OSCILLATION_AMPLITUDE: f32 = 40.0;
const OSCILLATION_SPEED: f32 = 4.0;

fn conf() -> Conf {
	Conf {
		window_title: String::from("Macroquad"),
		window_width: 1260,
		window_height: 768,
		fullscreen: false,
		..Default::default()
	}
}

#[macroquad::main(conf)]
async fn main() {
	let mut camera_controller = CameraController::new();

	let mut last_mouse_position: Vec2 = mouse_position().into();

	let mut audio_manager =
		AudioManager::<DefaultBackend>::new(AudioManagerSettings::default()).unwrap();
	
	let mut listener = audio_manager
		.add_listener(camera_controller.position, camera_controller.orientation())
		.unwrap();
	
	let mut spatial_track = audio_manager
		.add_spatial_sub_track(
			&listener,
			SPATIAL_TRACK_POSITION,
			SpatialTrackBuilder::new()
				.distances(SpatialTrackDistances {
					min_distance: 1.0,
					max_distance: 400.0,
				})
				// NOTE: Even though the doppler effect is enabled, the sound will not be affected by it
				// until the listener and the spatial track have set the game loop delta time. See below!
				.doppler_effect(true)
		)
		.unwrap();
	
	spatial_track
		.play(
			// motor_loop.wav: https://freesound.org/people/soundjoao/sounds/325809/
			StaticSoundData::from_file("crates/examples/assets/motor_loop.wav")
				.unwrap()
				.loop_region(0.0..),
		)
		.unwrap();

	let mut time = 0.0f32;
	
	loop {
		let delta_time = get_frame_time();
		time += delta_time;

		if is_key_pressed(KeyCode::Escape) {
			break;
		}

		let mouse_position: Vec2 = mouse_position().into();
		let mouse_delta = mouse_position - last_mouse_position;
		last_mouse_position = mouse_position;
		camera_controller.update(delta_time, mouse_delta);
		listener.set_position(camera_controller.position, Tween::default());
		listener.set_orientation(camera_controller.orientation(), Tween::default());

		clear_background(LIGHTGRAY);

		// Going 3d!
		set_camera(&camera_controller.camera());

		draw_grid(20, 1., BLACK, GRAY);

		let source_position = Vec3::new(
			SPATIAL_TRACK_POSITION.x,
			SPATIAL_TRACK_POSITION.y,
			SPATIAL_TRACK_POSITION.z + (time * OSCILLATION_SPEED).sin() * OSCILLATION_AMPLITUDE
		);

		spatial_track.set_position(source_position, Tween::default());

		// need to set these every frame unless you're dealing with a fixed timestep
		listener.set_game_loop_delta_time(delta_time as f64);
		spatial_track.set_game_loop_delta_time(delta_time as f64);

		draw_cube_wires(source_position, vec3(2., 2., 2.), GREEN);

		// Back to screen space, render some text
		set_default_camera();
		
		draw_text(
			&format!("FPS: {}", get_fps()),
			20.0,
			40.0,
			30.0,
			BLACK,
		);

		next_frame().await
	}
}

struct CameraController {
	position: Vec3,
	yaw: f32,
	pitch: f32,
	velocity: Vec3,
}

impl CameraController {
	fn new() -> Self {
		Self {
			position: vec3(0.0, 1.0, 50.0),
			yaw: 0.0,
			pitch: 0.0,
			velocity: Vec3::ZERO,
		}
	}

	fn update(&mut self, delta_time: f32, mouse_delta: Vec2) {
		let forward = self.front();
		let right = self.right();

		let mut desired_dir = Vec3::ZERO;
		if is_key_down(KeyCode::W) {
			desired_dir += forward;
		}
		if is_key_down(KeyCode::S) {
			desired_dir -= forward;
		}
		if is_key_down(KeyCode::A) {
			desired_dir -= right;
		}
		if is_key_down(KeyCode::D) {
			desired_dir += right;
		}

		if is_key_down(KeyCode::Left) {
			self.yaw += 2.0 * delta_time;
		}
		if is_key_down(KeyCode::Right) {
			self.yaw -= 2.0 * delta_time;
		}

		if is_key_down(KeyCode::Up) {
			self.pitch += 2.0 * delta_time;
		}
		if is_key_down(KeyCode::Down) {
			self.pitch -= 2.0 * delta_time;
		}
		
		let desired_dir = desired_dir.normalize_or_zero();
		self.velocity += desired_dir * CAMERA_ACCEL * delta_time;
		self.velocity *= 1.0 - CAMERA_DRAG * delta_time;
		if self.velocity.length() > CAMERA_MAX_SPEED {
			self.velocity = self.velocity.normalize() * CAMERA_MAX_SPEED;
		}
		
		self.position += self.velocity * delta_time;
		self.yaw -= mouse_delta.x * LOOK_SPEED;
		self.pitch = (self.pitch - mouse_delta.y * LOOK_SPEED).clamp(-1.5, 1.5);
	}

	fn orientation(&self) -> Quat {
		Quat::from_rotation_y(self.yaw) * Quat::from_rotation_x(self.pitch)
	}

	fn camera(&self) -> Camera3D {
		Camera3D {
			position: self.position,
			target: self.position + self.front(),
			up: WORLD_UP,
			..Default::default()
		}
	}

	fn front(&self) -> Vec3 {
		-self.orientation().mul_vec3(Vec3::Z).normalize()
	}

	fn right(&self) -> Vec3 {
		self.orientation().mul_vec3(Vec3::X).normalize()
	}
}

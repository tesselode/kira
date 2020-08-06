use conductor::{
	manager::{AudioManager, AudioManagerSettings, InstanceSettings, LooperSettings},
	project::{Project, SoundId, SoundSettings},
	sequence::Sequence,
	time::Time,
};
use ggez::{
	event::{KeyCode, KeyMods},
	graphics, Context, GameResult,
};
use std::error::Error;

struct MainState {
	audio_manager: AudioManager,
	test_loop_id: SoundId,
	cymbal_id: SoundId,
}

impl MainState {
	pub fn new() -> Result<Self, Box<dyn Error>> {
		let mut project = Project::new();
		let test_loop_id = project.load_sound(
			&std::env::current_dir()
				.unwrap()
				.join("assets/test_loop.ogg"),
			SoundSettings {
				tempo: Some(128.0),
				default_loop_end: Some(Time::Beats(4.0)),
				..Default::default()
			},
		)?;
		let cymbal_id = project.load_sound(
			&std::env::current_dir().unwrap().join("assets/cymbal.ogg"),
			SoundSettings::default(),
		)?;
		let mut audio_manager = AudioManager::new(
			project,
			AudioManagerSettings {
				tempo: 128.0,
				..Default::default()
			},
		)?;
		audio_manager.start_metronome();
		audio_manager.loop_sound(test_loop_id, LooperSettings::default())?;
		Ok(Self {
			audio_manager,
			test_loop_id,
			cymbal_id,
		})
	}
}

impl ggez::event::EventHandler for MainState {
	fn update(&mut self, _ctx: &mut Context) -> GameResult {
		Ok(())
	}

	fn key_down_event(
		&mut self,
		_ctx: &mut Context,
		keycode: KeyCode,
		_keymods: KeyMods,
		_repeat: bool,
	) {
		match keycode {
			KeyCode::Space => {
				self.audio_manager
					.start_sequence(
						Sequence::new()
							.on_interval(0.5)
							.play_sound(self.cymbal_id, InstanceSettings::default())
							.wait(Time::Beats(0.25))
							.play_sound(self.cymbal_id, InstanceSettings::default()),
					)
					.unwrap();
			}
			_ => {}
		}
	}

	fn draw(&mut self, ctx: &mut Context) -> GameResult {
		graphics::clear(ctx, graphics::BLACK);
		graphics::present(ctx)?;
		Ok(())
	}
}

fn main() -> Result<(), Box<dyn Error>> {
	let (mut ctx, mut event_loop) = ggez::ContextBuilder::new("conductor-test", "tesselode")
		.modules(ggez::conf::ModuleConf {
			audio: false,
			..Default::default()
		})
		.build()?;
	let mut main_state = MainState::new()?;
	ggez::event::run(&mut ctx, &mut event_loop, &mut main_state)?;
	Ok(())
}

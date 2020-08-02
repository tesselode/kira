use conductor::{
	manager::{AudioManager, AudioManagerSettings},
	project::{Project, SoundId, SoundSettings},
};
use ggez::{
	event::{KeyCode, KeyMods},
	graphics, Context, GameResult,
};
use std::error::Error;

struct MainState {
	audio_manager: AudioManager,
	sound_id: SoundId,
}

impl MainState {
	pub fn new() -> Result<Self, Box<dyn Error>> {
		let mut project = Project::new();
		let sound_id = project.load_sound(
			&std::env::current_dir().unwrap().join("assets/hhclosed.ogg"),
			SoundSettings::default(),
		)?;
		let mut audio_manager = AudioManager::new(
			project,
			AudioManagerSettings {
				tempo: 90.0,
				metronome_event_intervals: vec![0.25, 0.5, 1.0],
				..Default::default()
			},
		)?;
		audio_manager.start_metronome();
		Ok(Self {
			audio_manager,
			sound_id,
		})
	}
}

impl ggez::event::EventHandler for MainState {
	fn update(&mut self, _ctx: &mut Context) -> GameResult {
		for event in self.audio_manager.events() {
			println!("{:?}", event);
		}
		Ok(())
	}

	fn key_down_event(
		&mut self,
		_ctx: &mut Context,
		_keycode: KeyCode,
		_keymods: KeyMods,
		_repeat: bool,
	) {
		self.audio_manager.play_sound(self.sound_id);
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

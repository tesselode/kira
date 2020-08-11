use conductor::{
	instance::InstanceSettings,
	manager::{AudioManager, AudioManagerSettings},
	metronome::MetronomeSettings,
	project::Project,
	sequence::{PlaySoundTaskSettings, Sequence},
	sound::SoundId,
	time::Time,
	tween::Tween,
};
use ggez::{
	event::{KeyCode, KeyMods},
	graphics, Context, GameResult,
};
use std::error::Error;

struct MainState {
	audio_manager: AudioManager,
	sound_id: SoundId,
	paused: bool,
}

impl MainState {
	pub fn new() -> Result<Self, Box<dyn Error>> {
		let mut project = Project::new();
		let sound_id = project.load_sound(
			&std::env::current_dir()
				.unwrap()
				.join("assets/test_song.ogg"),
		)?;
		let mut audio_manager = AudioManager::new(project, AudioManagerSettings::default())?;
		let instance_id = audio_manager.play_sound(sound_id, InstanceSettings::default())?;
		audio_manager.set_instance_pitch(instance_id, 0.25, Some(Tween(4.0)))?;
		Ok(Self {
			audio_manager,
			sound_id,
			paused: false,
		})
	}
}

impl ggez::event::EventHandler for MainState {
	fn update(&mut self, _ctx: &mut Context) -> GameResult {
		self.audio_manager.events();
		Ok(())
	}

	fn key_down_event(
		&mut self,
		_ctx: &mut Context,
		keycode: KeyCode,
		_keymods: KeyMods,
		_repeat: bool,
	) {
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

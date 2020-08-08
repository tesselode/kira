use conductor::{
	id::{MetronomeId, SoundId},
	manager::{AudioManager, AudioManagerSettings, InstanceSettings},
	project::{MetronomeSettings, Project},
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
	sound_id: SoundId,
	metronome_id: MetronomeId,
}

impl MainState {
	pub fn new() -> Result<Self, Box<dyn Error>> {
		let mut project = Project::new();
		let sound_id = project.load_sound(
			&std::env::current_dir()
				.unwrap()
				.join("assets/test_loop.ogg"),
		)?;
		let metronome_id = project.create_metronome(
			128.0,
			MetronomeSettings {
				interval_events_to_emit: vec![0.25, 0.5, 1.0],
			},
		);
		let mut sequence = Sequence::new(metronome_id);
		sequence.play_sound(sound_id, InstanceSettings::default());
		sequence.wait(Time::Beats(4.0));
		sequence.go_to(0);
		let mut audio_manager = AudioManager::new(project, AudioManagerSettings::default())?;
		audio_manager.start_metronome(metronome_id).unwrap();
		audio_manager.start_sequence(sequence).unwrap();
		Ok(Self {
			audio_manager,
			sound_id,
			metronome_id,
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
		match keycode {
			KeyCode::Space => {
				self.audio_manager
					.start_metronome(self.metronome_id)
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

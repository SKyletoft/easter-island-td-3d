#![allow(clippy::type_complexity, dead_code)]

use bevy::{pbr::DirectionalLightShadowMap, prelude::*};
use gameplay::{
	levels::{Difficulty, GameState, SpawnTimer},
	towers::{Banking, Tower},
	ui::Click,
};

mod easy;
mod gameplay;
mod hard;
mod normal;

fn main() {
	App::new()
		.add_plugins(DefaultPlugins)
		.add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
		.insert_resource(DirectionalLightShadowMap { size: 8192 })
		.insert_resource(SpawnTimer(Timer::from_seconds(2.0, TimerMode::Repeating)))
		.insert_resource(GameState {
			number: 0,
			active: true,
			difficulty: Difficulty::Easy,
			wave: easy::WAVES[0].clone(),
		})
		.insert_resource(Banking {
			selection: Some(Tower::Land),
			balance: 1000000,
		})
		.add_event::<Click>()
		.add_systems(
			Startup,
			(
				easy::setup,
				gameplay::ui::setup_ui,
				gameplay::ui::init_textures,
				gameplay::towers::init_bullet_model,
				gameplay::enemies::init_enemies,
			),
		)
		.add_systems(
			Update,
			(
				gameplay::enemies::move_enemies,
				gameplay::enemies::animate_enemies,
				gameplay::cursor::move_cursor_and_camera,
				gameplay::towers::move_bullets,
				gameplay::towers::spawn_tower,
				gameplay::towers::land_attack,
				gameplay::towers::air_attack,
				gameplay::levels::spawn_enemy,
				gameplay::ui::run_shop,
				gameplay::ui::generate_clicks,
			),
		)
		.run();
}

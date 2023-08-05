#![allow(clippy::type_complexity, dead_code)]

use bevy::{pbr::DirectionalLightShadowMap, prelude::*};
use gameplay::{Click, Difficulty, Tower};

mod easy;
mod gameplay;
mod hard;
mod normal;

fn main() {
	App::new()
		.add_plugins(DefaultPlugins)
		.add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
		.insert_resource(DirectionalLightShadowMap { size: 8192 })
		.insert_resource(gameplay::SpawnTimer(Timer::from_seconds(
			2.0,
			TimerMode::Repeating,
		)))
		.insert_resource(gameplay::GameState {
			number: 0,
			active: true,
			difficulty: Difficulty::Easy,
			wave: easy::WAVES[0].clone(),
		})
		.insert_resource(gameplay::Banking {
			selection: Some(Tower::Land),
			balance: 1000000,
		})
		.add_event::<Click>()
		.add_systems(
			Startup,
			(
				easy::setup,
				gameplay::setup_ui,
				gameplay::init_bullet_model,
				gameplay::init_textures,
				gameplay::init_enemies,
			),
		)
		.add_systems(
			Update,
			(
				gameplay::move_enemies,
				gameplay::move_cursor_and_camera,
				gameplay::move_bullets,
				gameplay::animate_enemies,
				gameplay::spawn_enemy,
				gameplay::spawn_tower,
				gameplay::run_shop,
				gameplay::generate_clicks,
				gameplay::land_attack,
				gameplay::air_attack,
			),
		)
		.run();
}

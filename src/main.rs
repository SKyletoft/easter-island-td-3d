#![allow(clippy::type_complexity)]

use bevy::{pbr::DirectionalLightShadowMap, prelude::*};
use gameplay::{Click, Difficulty, Tower};

mod easy;
mod gameplay;
mod hard;
mod normal;

fn main() {
	App::new()
		.add_plugins(DefaultPlugins)
		// .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
		.insert_resource(DirectionalLightShadowMap { size: 8192 })
		.insert_resource(gameplay::SpawnTimer(Timer::from_seconds(
			2.0,
			TimerMode::Repeating,
		)))
		.insert_resource(gameplay::Level {
			number: 0,
			active: true,
			difficulty: Difficulty::Easy,
			wave: easy::WAVES[0].clone(),
		})
		.insert_resource(gameplay::Selection {
			tower: Some(Tower::Land),
		})
		.add_event::<Click>()
		.add_systems(
			Startup,
			(
				easy::setup,
				gameplay::setup_ui,
				// gameplay::load_scene,
				// gameplay::setup,
			),
		)
		.add_systems(
			Update,
			(
				gameplay::generate_clicks,
				gameplay::move_enemies,
				gameplay::animate_enemies,
				gameplay::move_cursor_and_camera,
				gameplay::land_attack,
				// gameplay::air_attack,
				gameplay::spawn_enemy,
				gameplay::spawn_tower,
			),
		)
		.run();
}

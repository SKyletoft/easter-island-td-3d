#![allow(clippy::type_complexity)]

use bevy::{prelude::*, pbr::DirectionalLightShadowMap};
use gameplay::Difficulty;

mod easy;
mod gameplay;
mod hard;
mod normal;

fn main() {
	App::new()
		.add_plugins(DefaultPlugins)
		.add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
		.insert_resource(DirectionalLightShadowMap {size: 10000})
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
		.add_systems(Startup, easy::setup)
		.add_systems(
			Update,
			(
				gameplay::move_enemies,
				gameplay::animate_enemies,
				gameplay::move_cursor_and_camera,
				gameplay::land_attack,
				// gameplay::air_attack,
				gameplay::spawn_enemy,
			),
		)
		.run();
}

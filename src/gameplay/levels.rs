use bevy::prelude::*;

use super::enemies::{EnemyType, PathSelection};
use crate::{easy::Wave, gameplay::enemies};

type Colour = Color;

// ------------------------------ LEVELS ---------------------------------

#[derive(Debug)]
pub enum Difficulty {
	Easy,
	Normal,
	Hard,
}

#[derive(Resource, Debug)]
pub struct SpawnTimer(pub Timer);

#[derive(Resource, Debug)]
pub struct GameState {
	pub number: usize,
	pub active: bool,
	pub difficulty: Difficulty,
	pub wave: Wave,
}

#[derive(Event, Debug)]
pub struct LevelEnd;

pub fn spawn_enemy(
	time: Res<Time>,
	mut level: ResMut<GameState>,
	mut timer: ResMut<SpawnTimer>,
	mut commands: Commands,
) {
	if !level.active || !timer.0.tick(time.delta()).just_finished() {
		return;
	}

	let Some((enemy_type, path)) = level.wave.0.pop() else {
		level.active = false;
		return;
	};
	let path_selection = PathSelection(path as usize);

	match enemy_type {
		EnemyType::Slow => {
			commands.spawn(enemies::slow(path_selection));
		}
		EnemyType::Normal => {
			commands.spawn(enemies::normal(path_selection));
		}
		EnemyType::Fast => {
			commands.spawn(enemies::fast(path_selection));
		}
		EnemyType::Air => {
			commands.spawn(enemies::air(path_selection));
		}
		EnemyType::Split => {
			commands.spawn(enemies::split(path_selection));
		}
	}
}

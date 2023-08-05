use bevy::prelude::*;
use once_cell::sync::{Lazy, OnceCell};

use crate::{easy, gameplay::path::Path};

type Colour = Color;

// ------------------------------ ENEMIES --------------------------------

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum EnemyType {
	Slow,
	Normal,
	Fast,
	Air,
	Split,
}

#[derive(Component, Debug)]
pub struct Enemy;

#[derive(Component, Debug)]
pub struct Slow;

#[derive(Component, Debug)]
pub struct Normal;

#[derive(Component, Debug)]
pub struct Fast;

#[derive(Component, Debug)]
pub struct Air;

#[derive(Component, Debug)]
pub struct SplitParent;

#[derive(Component, Debug)]
pub struct SplitChild;

#[derive(Component, Debug)]
pub struct Health {
	pub max: i32,
	pub current: i32,
}

impl Health {
	pub const fn new(val: i32) -> Health {
		assert!(val > 0);
		Health {
			max: val,
			current: val,
		}
	}
}

#[derive(Component, Debug)]
pub struct Progress(pub f32);

#[derive(Component, Debug)]
pub struct Speed(pub f32);

#[derive(Component, Debug)]
pub struct PathSelection(pub usize);

#[derive(Bundle, Debug)]
pub struct EnemyBundle {
	pub enemy: Enemy,
	pub health: Health,
	pub speed: Speed,
	pub progress: Progress,
	pub path_selection: PathSelection,
}

pub fn move_enemies(
	mut query: Query<(&mut Transform, &Speed, &mut Progress, &PathSelection), With<Enemy>>,
	d_time: Res<Time>,
) {
	const AVG_RANGE: f32 = 0.005;

	for (mut loc, speed, mut prog, path_selection) in query.iter_mut() {
		prog.0 += speed.0 * d_time.delta_seconds();
		let path = &easy::PATHS[path_selection.0];
		// Rounds out the corner
		loc.translation =
			(path.interpolate(prog.0 + AVG_RANGE) + path.interpolate(prog.0 - AVG_RANGE)) / 2.0;

		let towards = path.interpolate(prog.0 + AVG_RANGE);
		loc.look_at(towards, Vec3::Y);
	}
}

pub fn animate_enemies(time: Res<Time>, mut enemies: Query<&mut Transform, With<Enemy>>) {
	static WALK_CYCLE: Lazy<Path> = Lazy::new(|| {
		Path::new(&[
			Vec3::new(100.0, 100.0, 100.0),
			Vec3::new(102.0, 98.0, 102.0),
			Vec3::new(108.0, 92.0, 108.0),
			Vec3::new(110.0, 90.0, 110.0),
			Vec3::new(108.0, 92.0, 108.0),
			Vec3::new(102.0, 98.0, 102.0),
			Vec3::new(100.0, 100.0, 100.0),
		])
	});
	const TIME_SCALING_FACTOR: f32 = 1.0;
	const SIZE_SCALING_FACTOR: f32 = 0.005;

	let secs = time.elapsed_seconds();
	for mut trans in enemies.iter_mut() {
		let dt = (secs * TIME_SCALING_FACTOR) % 1.0;
		trans.scale = SIZE_SCALING_FACTOR * WALK_CYCLE.interpolate(dt);
	}
}

// Static handles to prevent reloading the same asset over and over again.
// Might already be handled by Bevy though
static SLOW_SCENE: OnceCell<Handle<Scene>> = OnceCell::new();
static NORMAL_SCENE: OnceCell<Handle<Scene>> = OnceCell::new();
static FAST_SCENE: OnceCell<Handle<Scene>> = OnceCell::new();
static AIR_SCENE: OnceCell<Handle<Scene>> = OnceCell::new();
static SPLIT_SCENE: OnceCell<Handle<Scene>> = OnceCell::new();

pub fn init_enemies(asset_server: Res<AssetServer>) {
	SLOW_SCENE.get_or_init(|| asset_server.load("exported/Slow.gltf#Scene0"));
	NORMAL_SCENE.get_or_init(|| asset_server.load("exported/Normal.gltf#Scene0"));
	FAST_SCENE.get_or_init(|| asset_server.load("exported/Fast.gltf#Scene0"));
	AIR_SCENE.get_or_init(|| asset_server.load("exported/Air.gltf#Scene0"));
	SPLIT_SCENE.get_or_init(|| asset_server.load("exported/Split.gltf#Scene0"));
}

pub fn slow(path_selection: PathSelection) -> impl Bundle {
	(
		SceneBundle {
			scene: SLOW_SCENE
				.get()
				.expect("Slow scene should've been initialised")
				.clone(),
			transform: Transform::from_xyz(0.0, 1.5, 0.0).with_scale(Vec3::ONE * 0.5),
			..default()
		},
		EnemyBundle {
			enemy: Enemy,
			speed: Speed(0.01),
			health: Health::new(10),
			progress: Progress(0.0),
			path_selection,
		},
		Slow,
	)
}

pub fn normal(path_selection: PathSelection) -> impl Bundle {
	(
		SceneBundle {
			scene: NORMAL_SCENE
				.get()
				.expect("Slow scene should've been initialised")
				.clone(),
			transform: Transform::from_xyz(0.0, 1.5, 0.0).with_scale(Vec3::ONE * 0.5),
			..default()
		},
		EnemyBundle {
			enemy: Enemy,
			speed: Speed(0.02),
			health: Health::new(10),
			progress: Progress(0.0),
			path_selection,
		},
		Slow,
	)
}

pub fn fast(path_selection: PathSelection) -> impl Bundle {
	(
		SceneBundle {
			scene: FAST_SCENE
				.get()
				.expect("Slow scene should've been initialised")
				.clone(),
			transform: Transform::from_xyz(0.0, 1.5, 0.0).with_scale(Vec3::ONE * 0.5),
			..default()
		},
		EnemyBundle {
			enemy: Enemy,
			speed: Speed(0.04),
			health: Health::new(100),
			progress: Progress(0.0),
			path_selection,
		},
		Fast,
	)
}

pub fn air(path_selection: PathSelection) -> impl Bundle {
	(
		SceneBundle {
			scene: AIR_SCENE
				.get()
				.expect("Slow scene should've been initialised")
				.clone(),
			transform: Transform::from_xyz(0.0, 1.5, 0.0).with_scale(Vec3::ONE * 0.5),
			..default()
		},
		EnemyBundle {
			enemy: Enemy,
			speed: Speed(0.04),
			health: Health::new(1000),
			progress: Progress(0.0),
			path_selection,
		},
		Fast,
	)
}

pub fn split(path_selection: PathSelection) -> impl Bundle {
	(
		SceneBundle {
			scene: SPLIT_SCENE
				.get()
				.expect("Slow scene should've been initialised")
				.clone(),
			transform: Transform::from_xyz(0.0, 1.5, 0.0).with_scale(Vec3::ONE * 0.5),
			..default()
		},
		EnemyBundle {
			enemy: Enemy,
			speed: Speed(0.04),
			health: Health::new(1000),
			progress: Progress(0.0),
			path_selection,
		},
		Fast,
	)
}

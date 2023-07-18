use std::iter;

use bevy::prelude::*;
use once_cell::sync::Lazy;

use crate::gameplay::{self, EnemyType, Path};

type Colour = Color;

pub fn setup(
	mut commands: Commands,
	asset_server: Res<AssetServer>,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
) {
	gameplay::spawn_axes(&mut commands, &mut meshes, &mut materials);
	gameplay::spawn_cursors(&mut commands, &mut meshes, &mut materials);

	// Level
	commands.spawn((PbrBundle {
		mesh: asset_server.load("exported/easy.gltf#Mesh0/Primitive0"),
		material: materials.add(Colour::rgb(0.0, 0.8, 0.0).into()),
		transform: Transform::from_xyz(0.0, 0.0, 0.0),
		..default()
	},));

	// Light
	commands.spawn(DirectionalLightBundle {
		directional_light: DirectionalLight {
			shadows_enabled: true,
			illuminance: 25_000.0,
			..default()
		},
		transform: Transform::from_xyz(4.0, 8.0, 14.0).looking_at(Vec3::ZERO, Vec3::Y),
		..default()
	});

	// Camera
	commands.spawn(Camera3dBundle {
		// transform: Transform::from_xyz(-15.0, 12.5, 1.0)
		transform: Transform::from_xyz(-30.0, 25.0, 2.0).looking_at(Vec3::ZERO, Vec3::Y),
		..default()
	});
}

pub static BOTTOM_PATH: Lazy<Path> = Lazy::new(|| {
	Path::new([
		(-6, 0, -45),
		(-6, 0, -26),
		(-14, 0, -26),
		(-14, 0, 10),
		(-10, 0, 10),
		(-10, 0, 18),
		(-2, 0, 18),
		(-2, 0, 30),
		(6, 0, 30),
		(6, 0, 45),
	])
});

pub static TOP_PATH: Lazy<Path> = Lazy::new(|| {
	Path::new([
		(-6, 0, -45),
		(-6, 0, -26),
		(2, 0, -26),
		(2, 0, -14),
		(10, 0, -14),
		(10, 0, -6),
		(14, 0, -6),
		(14, 0, 30),
		(6, 0, 30),
		(6, 0, 45),
	])
});

pub static MIDDLE_PATH: Lazy<Path> = Lazy::new(|| {
	Path::new([
		(-6, 0, -45),
		(-6, 0, -26),
		(2, 0, -26),
		(2, 0, -14),
		(-2, 0, -14),
		(-2, 0, 2),
		(2, 0, 2),
		(2, 0, 18),
		(-2, 0, 18),
		(-2, 0, 30),
		(6, 0, 30),
		(6, 0, 45),
	])
});

pub static PATHS: [&Lazy<Path>; 3] = [&TOP_PATH, &MIDDLE_PATH, &BOTTOM_PATH];

#[derive(Debug, Default, Clone)]
pub struct Wave(pub Vec<(EnemyType, u8)>);

impl Wave {
	pub fn new(enemy_pattern: &[EnemyType], enemy_count: usize) -> Self {
		Wave(
			iter::repeat(enemy_pattern.iter().copied())
				.flatten()
				.zip(iter::repeat(0..3).flatten())
				.take(enemy_count)
				.collect(),
		)
	}
}

pub static WAVES: Lazy<[Wave; 5]> = Lazy::new(|| {
	[
		Wave::new(&[EnemyType::Slow], 30),
		Wave::new(&[EnemyType::Normal], 30),
		Wave::new(&[EnemyType::Air], 30),
		Wave::new(&[EnemyType::Fast, EnemyType::Slow], 30),
		Wave::new(&[EnemyType::Split], 10),
	]
});

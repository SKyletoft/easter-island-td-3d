use std::iter;

use bevy::prelude::*;
use once_cell::sync::Lazy;

use crate::gameplay::{self, EnemyType, Path};

// Moved to a separate file because it absolutely destroys treesitter performance somehow
pub const HEIGHT_MAP: [[i8; 20]; 16] = include!("easy_height_map.rs");

pub fn setup(
	mut commands: Commands,
	asset_server: Res<AssetServer>,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
) {
	gameplay::spawn_axes(&mut commands, &mut meshes, &mut materials);
	gameplay::spawn_cursors(&mut commands, &mut meshes, &mut materials, &asset_server);

	let half_scale = Vec3 {
		x: 0.5,
		y: 1.0,
		z: 0.5,
	};

	// Level
	let ground: Handle<Mesh> = asset_server.load("exported/SimpleGround.gltf#Mesh0/Primitive0");
	let depth: Handle<Image> = asset_server.load("blender/EasyGroundDepth.png");
	let albedo: Handle<Image> = asset_server.load("blender/EasyGroundAlbedo.png");
	let overlay: Handle<Image> = asset_server.load("blender/EasyGroundOverlay.png");
	let normal: Handle<Image> = asset_server.load("blender/EasyGroundNormal.png");

	commands.spawn(PbrBundle {
		mesh: ground.clone(),
		material: materials.add(StandardMaterial {
			base_color_texture: Some(overlay),
			normal_map_texture: Some(normal.clone()),
			alpha_mode: AlphaMode::Mask(0.5),
			depth_map: Some(depth),
			parallax_mapping_method: ParallaxMappingMethod::Relief { max_steps: 3 },
			parallax_depth_scale: 0.1,
			perceptual_roughness: 0.2,
			max_parallax_layer_count: 16.0,
			..default()
		}),
		transform: Transform::from_xyz(0.0, 0.01, 0.0).with_scale(half_scale),
		..default()
	});
	commands.spawn(PbrBundle {
		mesh: ground,
		material: materials.add(StandardMaterial {
			base_color_texture: Some(albedo),
			normal_map_texture: Some(normal),
			..default()
		}),
		transform: Transform::from_xyz(0.0, 0.0, 0.0).with_scale(half_scale),
		..default()
	});

	// gameplay::visualise_height_map(&HEIGHT_MAP, &mut commands, &mut meshes, &mut materials);

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
		// transform: Transform::from_xyz(-30.0, 25.0, 2.0)
		transform: Transform::from_xyz(-12.0, 10.0, 0.8).looking_at(Vec3::ZERO, Vec3::Y),
		..default()
	});
}

pub static PATHS: [Lazy<Path>; 3] = [
	Lazy::new(|| {
		Path::from_keyframes([
			(-3, 0, -22),
			(-3, 0, -13),
			(1, 0, -13),
			(1, 0, -7),
			(5, 0, -7),
			(5, 0, -3),
			(7, 0, -3),
			(7, 0, 15),
			(3, 0, 15),
			(3, 0, 22),
		])
	}),
	Lazy::new(|| {
		Path::from_keyframes([
			(-3, 0, -22),
			(-3, 0, -13),
			(1, 0, -13),
			(1, 0, -7),
			(-1, 0, -7),
			(-1, 0, 1),
			(1, 0, 1),
			(1, 0, 9),
			(-1, 0, 9),
			(-1, 0, 15),
			(3, 0, 15),
			(3, 0, 22),
		])
	}),
	Lazy::new(|| {
		Path::from_keyframes([
			(-3, 0, -22),
			(-3, 0, -13),
			(-7, 0, -13),
			(-7, 0, 5),
			(-5, 0, 5),
			(-5, 0, 9),
			(-1, 0, 9),
			(-1, 0, 15),
			(3, 0, 15),
			(3, 0, 22),
		])
	}),
];

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
		Wave::new(&[EnemyType::Fast], 30),
		Wave::new(&[EnemyType::Normal], 30),
		Wave::new(&[EnemyType::Air], 30),
		Wave::new(&[EnemyType::Fast, EnemyType::Slow], 30),
		Wave::new(&[EnemyType::Split], 10),
	]
});

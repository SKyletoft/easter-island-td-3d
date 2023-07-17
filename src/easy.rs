use bevy::prelude::*;

use crate::gameplay::{Cursor, Enemy, Progress, Slow, Speed, VCursor};

pub fn setup(
	mut commands: Commands,
	asset_server: Res<AssetServer>,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
) {
	let slow_mesh_handle = asset_server.load("exported/slow.gltf#Mesh0/Primitive0");
	let moai_mesh_handle = asset_server.load("exported/moai.gltf#Mesh0/Primitive0");
	let easy_mesh_handle = asset_server.load("exported/easy.gltf#Mesh0/Primitive0");
	// let slow_material_handle = asset_server.load("exported/slow.gltf#Material/Primitive0");

	commands.spawn(PbrBundle {
		mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
		material: materials.add(Color::rgb(0.0, 0.0, 0.0).into()),
		transform: Transform::from_xyz(0.0, 6.0, 0.0).with_scale((0.5, 0.5, 0.5).into()),
		..default()
	});
	commands.spawn(PbrBundle {
		mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
		material: materials.add(Color::rgb(1.0, 0.0, 0.0).into()),
		transform: Transform::from_xyz(2.0, 6.0, 0.0).with_scale((0.5, 0.5, 0.5).into()),
		..default()
	});
	commands.spawn(PbrBundle {
		mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
		material: materials.add(Color::rgb(0.0, 1.0, 0.0).into()),
		transform: Transform::from_xyz(0.0, 8.0, 0.0).with_scale((0.5, 0.5, 0.5).into()),
		..default()
	});
	commands.spawn(PbrBundle {
		mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
		material: materials.add(Color::rgb(0.0, 0.0, 1.0).into()),
		transform: Transform::from_xyz(0.0, 6.0, 2.0).with_scale((0.5, 0.5, 0.5).into()),
		..default()
	});

	commands.spawn((
		PbrBundle {
			mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
			material: materials.add(Color::rgb(1.0, 0.0, 0.0).into()),
			transform: Transform::from_xyz(0.0, 0.0, 0.0).with_scale((0.1, 0.1, 0.1).into()),
			..default()
		},
		Cursor,
	));
	commands.spawn((
		PbrBundle {
			mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
			material: materials.add(Color::rgb(0.0, 0.0, 1.0).into()),
			transform: Transform::from_xyz(0.0, 0.0, 0.0).with_scale((0.1, 0.1, 0.1).into()),
			..default()
		},
		VCursor,
	));
	commands.spawn(PbrBundle {
		mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
		material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
		transform: Transform::from_xyz(0.0, 0.0, 0.0).with_scale((0.1, 0.1, 0.1).into()),
		..default()
	});

	commands.spawn((PbrBundle {
		mesh: moai_mesh_handle.clone(),
		material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
		transform: Transform::from_xyz(-1.0, 0.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
		..default()
	},));
	commands.spawn((PbrBundle {
		mesh: moai_mesh_handle.clone(),
		material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
		transform: Transform::from_xyz(1.0, 0.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
		..default()
	},));
	commands.spawn((PbrBundle {
		mesh: moai_mesh_handle.clone(),
		material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
		transform: Transform::from_xyz(0.0, 0.0, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
		..default()
	},));
	commands.spawn((PbrBundle {
		mesh: moai_mesh_handle,
		material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
		transform: Transform::from_xyz(0.0, 0.0, -1.0).looking_at(Vec3::ZERO, Vec3::Y),
		..default()
	},));

	commands.spawn((PbrBundle {
		mesh: easy_mesh_handle,
		material: materials.add(Color::rgb(0.0, 0.8, 0.0).into()),
		transform: Transform::from_xyz(0.0, 0.0, 0.0),
		..default()
	},));
	commands.spawn((
		PbrBundle {
			mesh: slow_mesh_handle,
			material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
			transform: Transform::from_xyz(0.0, 1.5, 0.0).with_scale(Vec3::ONE * 0.5),
			..default()
		},
		Enemy::Slow(Slow(0)),
		Speed(0.04),
		Progress(0.0),
	));
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

pub const BOTTOM_PATH: [Vec3; 10] = [
	Vec3::new(-6.0, 0.0, -40.0),
	Vec3::new(-6.0, 0.0, -26.0),
	Vec3::new(-14.0, 0.0, -26.0),
	Vec3::new(-14.0, 0.0, 10.0),
	Vec3::new(-10.0, 0.0, 10.0),
	Vec3::new(-10.0, 0.0, 18.0),
	Vec3::new(-2.0, 0.0, 18.0),
	Vec3::new(-2.0, 0.0, 30.0),
	Vec3::new(6.0, 0.0, 30.0),
	Vec3::new(6.0, 0.0, 40.0),
];
pub const TOP_PATH: [Vec3; 10] = [
	Vec3::new(-6.0, 0.0, -40.0),
	Vec3::new(-6.0, 0.0, -26.0),
	Vec3::new(2.0, 0.0, -26.0),
	Vec3::new(2.0, 0.0, -14.0),
	Vec3::new(10.0, 0.0, -14.0),
	Vec3::new(10.0, 0.0, -6.0),
	Vec3::new(14.0, 0.0, -6.0),
	Vec3::new(14.0, 0.0, 30.0),
	Vec3::new(6.0, 0.0, 30.0),
	Vec3::new(6.0, 0.0, 40.0),
];
pub const MIDDLE_PATH: [Vec3; 12] = [
	Vec3::new(-6.0, 0.0, -40.0),
	Vec3::new(-6.0, 0.0, -26.0),
	Vec3::new(2.0, 0.0, -26.0),
	Vec3::new(2.0, 0.0, -14.0),
	Vec3::new(-2.0, 0.0, -14.0),
	Vec3::new(-2.0, 0.0, 2.0),
	Vec3::new(2.0, 0.0, 2.0),
	Vec3::new(2.0, 0.0, 18.0),
	Vec3::new(-2.0, 0.0, 18.0),
	Vec3::new(-2.0, 0.0, 30.0),
	Vec3::new(6.0, 0.0, 30.0),
	Vec3::new(6.0, 0.0, 40.0),
];

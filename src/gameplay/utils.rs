use bevy::prelude::*;

use crate::{
	easy,
	gameplay::cursor::{Cursor, SquareHighlight},
};

type Colour = Color;

// ------------------------------- UTILS ---------------------------------

pub fn round_to_grid(v: Vec3) -> Vec3 {
	const ROUNDING_OFFSET: f32 = 1.0;
	const ROUND_TO: f32 = 2.0;

	let f = |x: f32| ((x + ROUNDING_OFFSET) / ROUND_TO).round() * ROUND_TO - ROUNDING_OFFSET;
	Vec3::new(f(v.x), v.y, f(v.z))
}

pub fn to_map_space(Vec3 { x, y: _, z }: Vec3) -> (usize, usize) {
	let height = easy::HEIGHT_MAP.len() - 1;
	let width = easy::HEIGHT_MAP[0].len() - 1;

	let d_x = (((x + height as f32) / 2.0).round() as usize)
		.min(height)
		.max(0);
	let d_z = (((z + width as f32) / 2.0).round() as usize)
		.min(width)
		.max(0);
	(d_x, d_z)
}

pub fn with_height(v: Vec3) -> Vec3 {
	let (d_x, d_z) = to_map_space(v);
	let height_data = easy::HEIGHT_MAP[d_x][d_z];
	let d_y = 1.0 - height_data as f32 / 10.0;

	Vec3::new(v.x, d_y, v.z)
}

pub fn to_grid_with_height(v: Vec3) -> Vec3 {
	with_height(round_to_grid(v))
}

pub fn get_world_pos(
	p: Vec2,
	cam_query: &mut Query<
		(&Camera, &GlobalTransform),
		(With<Camera3d>, Without<Cursor>, Without<VisualMarker>),
	>,
) -> Option<Vec3> {
	let (cam, g_trans) = cam_query.get_single_mut().ok()?;
	let ray = cam.viewport_to_world(g_trans, p)?;
	let dist = ray.intersect_plane(Vec3::new(0.0, 1.0, 0.0), Vec3::Y)?;
	Some(ray.get_point(dist))
}

// ------------------------------- UTILS ---------------------------------

pub fn spawn_cursors(
	commands: &mut Commands,
	meshes: &mut ResMut<Assets<Mesh>>,
	materials: &mut ResMut<Assets<StandardMaterial>>,
	asset_server: &Res<AssetServer>,
) {
	commands.spawn((
		PbrBundle {
			mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
			material: materials.add(Colour::rgb(1.0, 0.0, 0.0).into()),
			transform: Transform::from_xyz(0.0, 0.0, 0.0).with_scale((0.1, 4.0, 0.1).into()),
			..default()
		},
		Cursor,
	));
	commands.spawn((
		SceneBundle {
			scene: asset_server.load("exported/Square.gltf#Scene0"),
			transform: Transform::from_xyz(0.0, 0.0, 0.0),
			..default()
		},
		SquareHighlight,
	));
	commands.spawn(PbrBundle {
		mesh: asset_server.load("exported/Range.gltf#Mesh0/Primitive0"),
		material: materials.add(StandardMaterial {
			alpha_mode: AlphaMode::Blend,
			base_color: Colour::rgba(0.8, 0.8, 0.8, 0.6),
			unlit: true,
			double_sided: true,
			cull_mode: None,
			..default()
		}),
		transform: Transform::from_xyz(0.0, 1.0, 0.0),
		..default()
	});
}

pub fn spawn_axes(
	commands: &mut Commands,
	meshes: &mut ResMut<Assets<Mesh>>,
	materials: &mut ResMut<Assets<StandardMaterial>>,
) {
	let cube = meshes.add(Mesh::from(shape::Cube { size: 1.0 }));
	commands.spawn(PbrBundle {
		mesh: cube.clone(),
		material: materials.add(Colour::rgb(0.0, 0.0, 0.0).into()),
		transform: Transform::from_xyz(0.0, 6.0, 0.0).with_scale((0.5, 0.5, 0.5).into()),
		..default()
	});
	commands.spawn(PbrBundle {
		mesh: cube.clone(),
		material: materials.add(Colour::rgb(1.0, 0.0, 0.0).into()),
		transform: Transform::from_xyz(2.0, 6.0, 0.0).with_scale((0.5, 0.5, 0.5).into()),
		..default()
	});
	commands.spawn(PbrBundle {
		mesh: cube.clone(),
		material: materials.add(Colour::rgb(0.0, 1.0, 0.0).into()),
		transform: Transform::from_xyz(0.0, 8.0, 0.0).with_scale((0.5, 0.5, 0.5).into()),
		..default()
	});
	commands.spawn(PbrBundle {
		mesh: cube,
		material: materials.add(Colour::rgb(0.0, 0.0, 1.0).into()),
		transform: Transform::from_xyz(0.0, 6.0, 2.0).with_scale((0.5, 0.5, 0.5).into()),
		..default()
	});
}

#[derive(Component)]
pub struct VisualMarker;

pub fn visualise_height_map<const N: usize, const M: usize>(
	height_map: &[[i8; N]; M],
	commands: &mut Commands,
	meshes: &mut ResMut<Assets<Mesh>>,
	materials: &mut ResMut<Assets<StandardMaterial>>,
) {
	let material = materials.add(Colour::rgb(1.0, 1.0, 1.0).into());
	let mesh = meshes.add(Mesh::from(shape::Cube { size: 1.0 }));

	for x in (0..M).step_by(1) {
		for y in (0..N).step_by(1) {
			let size = height_map[x][y] as f32 / 10.0 + 0.05;

			let x = x as f32 * 2.0 - (M - 1) as f32;
			let y = y as f32 * 2.0 - (N - 1) as f32;

			commands.spawn((
				PbrBundle {
					mesh: mesh.clone(),
					material: material.clone(),
					transform: Transform::from_xyz(x, 1.0, y)
						.with_scale(Vec3::new(size, size, size)),
					..default()
				},
				VisualMarker,
			));
		}
	}
}

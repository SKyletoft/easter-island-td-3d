#![allow(clippy::type_complexity)]

use bevy::{input::mouse::MouseMotion, prelude::*};

#[derive(Component, Debug)]
struct Slow(usize);

#[derive(Component, Debug)]
struct Normal(usize);

#[derive(Component, Debug)]
struct Fast(usize);

#[derive(Component, Debug)]
struct Air(usize);

#[derive(Component, Debug)]
struct SplitParent(usize);

#[derive(Component, Debug)]
struct SplitChild(usize);

#[derive(Component, Debug)]
enum Enemy {
	Slow(Slow),
	Normal(Normal),
	Fast(Fast),
	Air(Air),
	SplitParent(SplitParent),
	SplitChild(SplitChild),
}

#[derive(Component, Debug)]
struct Health {
	max: u32,
	current: u32,
}

#[derive(Component, Debug)]
struct Cursor;

#[derive(Component, Debug)]
struct VCursor;

#[derive(Component, Debug)]
struct Progress(f32);

#[derive(Component, Debug)]
struct Speed(f32);

#[derive(Bundle, Debug)]
struct EnemyBundle {
	enemy: Enemy,
	health: Health,
	progress: Progress,
}

fn setup(
	mut commands: Commands,
	asset_server: Res<AssetServer>,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
) {
	let slow_mesh_handle = asset_server.load("exported/slow.gltf#Mesh0/Primitive0");
	let moai_mesh_handle = asset_server.load("exported/moai.gltf#Mesh0/Primitive0");
	// let slow_material_handle = asset_server.load("exported/slow.gltf#Material/Primitive0");

	commands.spawn((
		PbrBundle {
			mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
			material: materials.add(Color::rgb(1.0, 0.0, 0.0).into()),
			transform: Transform::from_xyz(0.0, 0.0, 4.0).with_scale((0.1, 0.1, 0.1).into()),
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
		transform: Transform::from_xyz(-1.0, 0.0, 0.0)
			.with_scale((0.1, 0.1, 0.1).into())
			.looking_at(Vec3::ZERO, Vec3::Y),
		..default()
	},));
	commands.spawn((PbrBundle {
		mesh: moai_mesh_handle.clone(),
		material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
		transform: Transform::from_xyz(1.0, 0.0, 0.0)
			.with_scale((0.1, 0.1, 0.1).into())
			.looking_at(Vec3::ZERO, Vec3::Y),
		..default()
	},));
	commands.spawn((PbrBundle {
		mesh: moai_mesh_handle.clone(),
		material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
		transform: Transform::from_xyz(0.0, 0.0, 1.0)
			.with_scale((0.1, 0.1, 0.1).into())
			.looking_at(Vec3::ZERO, Vec3::Y),
		..default()
	},));
	commands.spawn((PbrBundle {
		mesh: moai_mesh_handle,
		material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
		transform: Transform::from_xyz(0.0, 0.0, -1.0)
			.with_scale((0.1, 0.1, 0.1).into())
			.looking_at(Vec3::ZERO, Vec3::Y),
		..default()
	},));

	commands.spawn((
		PbrBundle {
			mesh: slow_mesh_handle,
			material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
			transform: Transform::from_xyz(0.0, 1.5, 0.0).with_scale((0.1, 0.1, 0.1).into()),
			..default()
		},
		Enemy::Slow(Slow(0)),
		Speed(0.04),
		Progress(0.0),
	));
	// light
	commands.spawn(PointLightBundle {
		point_light: PointLight {
			intensity: 1500.0,
			shadows_enabled: true,
			..default()
		},
		transform: Transform::from_xyz(4.0, 8.0, 4.0),
		..default()
	});
	// camera
	commands.spawn(Camera3dBundle {
		transform: Transform::from_xyz(-15.0, 12.5, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
		..default()
	});
}

const PATH: [Vec3; 3] = [
	Vec3::new(0.0, 0.0, -6.0),
	Vec3::new(0.0, 0.0, 0.0),
	Vec3::new(5.0, 0.0, 0.0),
];

fn move_enemies(
	mut query: Query<(&mut Transform, &Speed, &mut Progress), With<Enemy>>,
	d_time: Res<Time>,
) {
	for (mut loc, speed, mut prog) in query.iter_mut() {
		prog.0 += speed.0 * d_time.delta_seconds();
		loc.translation = interpolate(prog.0, &PATH);

		let towards = interpolate(prog.0 + 0.02, &PATH);
		loc.look_at(towards, Vec3::Y);
	}
}

fn move_cursor(
	button: Res<Input<MouseButton>>,
	win_query: Query<&Window>,
	mut cam_query: Query<
		(&Camera, &GlobalTransform, &mut Transform),
		(With<Camera3d>, Without<Cursor>, Without<VCursor>),
	>,
	mut cur_query: Query<
		&mut Transform,
		(With<Cursor>, Without<Camera3d>, Without<VCursor>)
	>,
	mut v_cur_query: Query<
		&mut Transform,
		(With<VCursor>, Without<Camera3d>, Without<Cursor>)
	>,
) {
	let Ok(win) = win_query.get_single() else {
		return;
	};
	let Ok((cam, g_trans, mut trans)) = cam_query.get_single_mut() else {
		return;
	};
	let Ok(mut cur) = cur_query.get_single_mut() else {
		return;
	};
	let Ok(mut v_cur) = v_cur_query.get_single_mut() else {
		return;
	};
	(|| {
		// Move cursor
		let mouse = win.cursor_position()?;
		let ray = cam.viewport_to_world(g_trans, mouse)?;
		let dist = ray.intersect_plane(Vec3::ZERO, Vec3::Y)?;
		let new_cur = ray.get_point(dist);
		cur.translation = new_cur;

		// Move camera
		if button.just_pressed(MouseButton::Left) {
			v_cur.translation = cur.translation;
		}
		if button.pressed(MouseButton::Left) {
			let diff = v_cur.translation - cur.translation;
			trans.translation += diff;
		}

		Some(())
	})();
}

fn interpolate(dt: f32, range: &[Vec3]) -> Vec3 {
	let dt = dt.max(0.0).min(1.0);
	match range.len() {
		0 => (0.0, 0.0, 0.0).into(),
		1 => range[0],
		l => {
			let i = (l - 1) as f32 * dt;
			let i_lo = i.floor() as usize;
			let i_hi = i.ceil() as usize;
			let i_frac = i - i_lo as f32;

			Vec3::lerp(range[i_lo], range[i_hi], i_frac)
		}
	}
}

fn main() {
	App::new()
		.add_plugins(DefaultPlugins)
		.add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
		.add_systems(Startup, setup)
		.add_systems(Update, (move_enemies, move_cursor))
		.run();
}

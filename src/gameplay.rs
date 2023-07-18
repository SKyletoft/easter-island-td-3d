use bevy::prelude::*;

type Colour = bevy::prelude::Color;

use crate::easy::{self, Wave};

// ------------------------------- PATH ----------------------------------

#[derive(Debug)]
pub struct Path(Vec<Vec3>);

impl Path {
	pub fn new<const N: usize>(points: [(i32, i32, i32); N]) -> Self {
		let interpolated = points
			.iter()
			.zip(points.iter().skip(1))
			.flat_map(|(v1, v2)| {
				let (x1, y1, z1) = *v1;
				let (x2, y2, z2) = *v2;

				let as_x = |x| Vec3::new(x as f32, y1 as f32, z1 as f32);
				let as_y = |y| Vec3::new(x1 as f32, y as f32, z1 as f32);
				let as_z = |z| Vec3::new(x1 as f32, y1 as f32, z as f32);

				if y1 == y2 && z1 == z2 {
					if x1 < x2 {
						(x1..=x2).map(as_x).collect::<Vec<Vec3>>()
					} else {
						(x2..=x1).rev().map(as_x).collect::<Vec<Vec3>>()
					}
				} else if x1 == x2 && z1 == z2 {
					if y1 < y2 {
						(y1..=y2).map(as_y).collect::<Vec<Vec3>>()
					} else {
						(y2..=y1).rev().map(as_y).collect::<Vec<Vec3>>()
					}
				} else if x1 == x2 && y1 == y2 {
					if z1 < z2 {
						(z1..=z2).map(as_z).collect::<Vec<Vec3>>()
					} else {
						(z2..=z1).rev().map(as_z).collect::<Vec<Vec3>>()
					}
				} else {
					panic!("Comptime data misconstructed")
				}
				.into_iter()
				.skip(1)
			})
			.collect();

		Path(interpolated)
	}

	pub fn interpolate(&self, dt: f32) -> Vec3 {
		let dt_ = dt.max(0.0).min(1.0);
		match self.0.len() {
			0 => (0.0, 0.0, 0.0).into(),
			1 => self.0[0],
			l => {
				let i = (l - 1) as f32 * dt_;
				let i_lo = i.floor() as usize;
				let i_hi = i.ceil() as usize;
				let i_frac = i - i_lo as f32;

				Vec3::lerp(self.0[i_lo], self.0[i_hi], i_frac)
			}
		}
	}
}

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
		let path = easy::PATHS[path_selection.0];
		loc.translation = (path.interpolate(prog.0 + AVG_RANGE)
			+ path.interpolate(prog.0 - AVG_RANGE))
			/ 2.0;

		let towards = path.interpolate(prog.0 + AVG_RANGE);
		loc.look_at(towards, Vec3::Y);
	}
}

pub fn fast(
	asset_server: Res<AssetServer>,
	mut materials: ResMut<Assets<StandardMaterial>>,
	path_selection: PathSelection,
) -> impl Bundle {
	let mesh = asset_server.load("exported/fast.gltf#Mesh0/Primitive0");

	(
		PbrBundle {
			mesh,
			material: materials.add(Colour::rgb(0.8, 0.7, 0.6).into()),
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
		Fast,
	)
}

pub fn slow(
	asset_server: Res<AssetServer>,
	mut materials: ResMut<Assets<StandardMaterial>>,
	path_selection: PathSelection,
) -> impl Bundle {
	let mesh = asset_server.load("exported/slow.gltf#Mesh0/Primitive0");

	(
		PbrBundle {
			mesh,
			material: materials.add(Colour::rgb(0.8, 0.7, 0.6).into()),
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

// ------------------------------ TOWERS ---------------------------------

#[derive(Component, Debug)]
pub enum Tower {
	Land,
	All,
	Fire,
	Water,
	Air,
	Laser,
}

#[derive(Component, Debug)]
pub struct RangedShooter(f32);

#[derive(Component, Debug)]
pub struct AoE(f32);

#[derive(Component, Debug)]
pub struct AttackSpeed(f32);

#[derive(Component, Debug)]
pub struct Damage(i32);

#[derive(Bundle, Debug)]
pub struct TowerBundle {
	pub tower: Tower,
	pub attack_speed: AttackSpeed,
	pub damage: Damage,
}

pub fn land_attack(
	towers: Query<(&Tower, &Transform, &RangedShooter, &Damage)>,
	mut enemies: Query<(&mut Health, &Transform), Without<Air>>,
) {
	for (_, tower_pos, tower_range, tower_dmg) in towers.iter() {
		for (mut hp, enemy_pos) in enemies
			.iter_mut()
			.filter(|(_, pos)| (tower_pos.translation - pos.translation).length() < tower_range.0)
		{
			hp.current -= tower_dmg.0;
		}
	}
}

pub const fn stone_tower() -> impl Bundle {
	(
		TowerBundle {
			tower: Tower::Land,
			attack_speed: AttackSpeed(15.0),
			damage: Damage(30),
		},
		RangedShooter(50.0),
	)
}

// ------------------------------ CURSOR ---------------------------------

#[derive(Component, Debug)]
pub struct Cursor;

#[derive(Component, Debug)]
pub struct VCursor;

const CAMERA_LIMITS: (Vec3, Vec3) = (
	Vec3::new(-45.0, 25.0, -30.0),
	Vec3::new(-10.0, 25.0, 32.0),
);

pub fn move_cursor_and_camera(
	button: Res<Input<MouseButton>>,
	win_query: Query<&Window>,
	mut cam_query: Query<
		(&Camera, &GlobalTransform, &mut Transform),
		(With<Camera3d>, Without<Cursor>, Without<VCursor>),
	>,
	mut cur_query: Query<&mut Transform, (With<Cursor>, Without<Camera3d>, Without<VCursor>)>,
	mut v_cur_query: Query<&mut Transform, (With<VCursor>, Without<Camera3d>, Without<Cursor>)>,
) {
	let mut inner = move || {
		let win = win_query.get_single().ok()?;
		let (cam, g_trans, mut trans) = cam_query.get_single_mut().ok()?;
		let mut cur = cur_query.get_single_mut().ok()?;
		let mut v_cur = v_cur_query.get_single_mut().ok()?;

		// Move cursor
		let mouse = win.cursor_position()?;
		let ray = cam.viewport_to_world(g_trans, mouse)?;
		let dist = ray.intersect_plane(Vec3::ZERO, Vec3::Y)?;
		let new_cur = ray.get_point(dist);
		cur.translation = new_cur;

		// Move camera
		if button.just_pressed(MouseButton::Left) {
			v_cur.translation = cur.translation;
			dbg!(&v_cur.translation);
		}
		if button.pressed(MouseButton::Left) {
			let diff = v_cur.translation - cur.translation;
			trans.translation = (trans.translation + diff)
				.max(CAMERA_LIMITS.0)
				.min(CAMERA_LIMITS.1);
		}

		Some(())
	};
	let _ = inner();
}

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
pub struct Level {
	pub number: usize,
	pub active: bool,
	pub difficulty: Difficulty,
	pub wave: Wave,
}

#[derive(Event, Debug)]
pub struct LevelEnd;

pub fn spawn_enemy(
	time: Res<Time>,
	mut level: ResMut<Level>,
	mut timer: ResMut<SpawnTimer>,
	mut commands: Commands,
	asset_server: Res<AssetServer>,
	materials: ResMut<Assets<StandardMaterial>>,
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
			commands.spawn(slow(asset_server, materials, path_selection));
		}
		EnemyType::Normal => {
			// commands.spawn(normal(asset_server, materials, path_selection));
		}
		EnemyType::Fast => {
			commands.spawn(fast(asset_server, materials, path_selection));
		}
		EnemyType::Air => {
			// commands.spawn(air(asset_server, materials, path_selection));
		}
		EnemyType::Split => {
			// commands.spawn(split(asset_server, materials, path_selection));
		}
	}
}

// -------------------------- SPAWN DEFAULTS -----------------------------

pub fn spawn_cursors(
	commands: &mut Commands,
	meshes: &mut ResMut<Assets<Mesh>>,
	materials: &mut ResMut<Assets<StandardMaterial>>,
) {
	commands.spawn((
		PbrBundle {
			mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
			material: materials.add(Colour::rgb(1.0, 0.0, 0.0).into()),
			transform: Transform::from_xyz(0.0, 0.0, 0.0).with_scale((0.1, 0.1, 0.1).into()),
			..default()
		},
		Cursor,
	));
	commands.spawn((
		PbrBundle {
			mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
			material: materials.add(Colour::rgb(0.0, 0.0, 1.0).into()),
			transform: Transform::from_xyz(0.0, 0.0, 0.0).with_scale((0.1, 0.1, 0.1).into()),
			..default()
		},
		VCursor,
	));
}

pub fn spawn_axes(
	commands: &mut Commands,
	meshes: &mut ResMut<Assets<Mesh>>,
	materials: &mut ResMut<Assets<StandardMaterial>>,
) {
	commands.spawn(PbrBundle {
		mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
		material: materials.add(Colour::rgb(0.0, 0.0, 0.0).into()),
		transform: Transform::from_xyz(0.0, 6.0, 0.0).with_scale((0.5, 0.5, 0.5).into()),
		..default()
	});
	commands.spawn(PbrBundle {
		mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
		material: materials.add(Colour::rgb(1.0, 0.0, 0.0).into()),
		transform: Transform::from_xyz(2.0, 6.0, 0.0).with_scale((0.5, 0.5, 0.5).into()),
		..default()
	});
	commands.spawn(PbrBundle {
		mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
		material: materials.add(Colour::rgb(0.0, 1.0, 0.0).into()),
		transform: Transform::from_xyz(0.0, 8.0, 0.0).with_scale((0.5, 0.5, 0.5).into()),
		..default()
	});
	commands.spawn(PbrBundle {
		mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
		material: materials.add(Colour::rgb(0.0, 0.0, 1.0).into()),
		transform: Transform::from_xyz(0.0, 6.0, 2.0).with_scale((0.5, 0.5, 0.5).into()),
		..default()
	});
}

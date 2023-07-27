use std::sync::Mutex;

use bevy::{pbr::ScreenSpaceAmbientOcclusionBundle, prelude::*};
use once_cell::sync::{Lazy, OnceCell};

type Colour = bevy::prelude::Color;

use crate::easy::{self, Wave};

const ROUNDING_OFFSET: f32 = 1.0;
fn round_to_grid(v: Vec3) -> Vec3 {
	let f = |x: f32| ((x + ROUNDING_OFFSET) / 2.0).round() * 2.0 - ROUNDING_OFFSET;
	Vec3::new(f(v.x), v.y, f(v.z))
}

#[derive(Event)]
pub struct Click;

pub fn generate_clicks(
	button: Res<Input<MouseButton>>,
	windows: Query<&Window>,
	mut ev: EventWriter<Click>,
) {
	static CURSOR_POS: Mutex<Vec2> = Mutex::new(Vec2::ZERO);

	let p = windows
		.get_single()
		.ok()
		.and_then(Window::cursor_position)
		.unwrap_or(Vec2::ZERO);
	let mut cur_ref = CURSOR_POS.lock().unwrap();

	if button.just_pressed(MouseButton::Left) {
		*cur_ref = p;
	}
	if button.just_released(MouseButton::Left) && p != Vec2::ZERO && *cur_ref == p {
		ev.send(Click);
	}
}

// ------------------------------- PATH ----------------------------------

#[derive(Debug)]
pub struct Path(Vec<Vec3>);

impl Path {
	pub fn from_keyframes<const N: usize>(points: [(i32, i32, i32); N]) -> Self {
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

	pub fn new(points: &[Vec3]) -> Self {
		Path(points.to_vec())
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
		let path = &easy::PATHS[path_selection.0];
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

pub fn slow(asset_server: &Res<AssetServer>, path_selection: PathSelection) -> impl Bundle {
	(
		SceneBundle {
			scene: SLOW_SCENE
				.get_or_init(|| asset_server.load("exported/Slow.gltf#Scene0"))
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

pub fn normal(asset_server: &Res<AssetServer>, path_selection: PathSelection) -> impl Bundle {
	(
		SceneBundle {
			scene: NORMAL_SCENE
				.get_or_init(|| asset_server.load("exported/Normal.gltf#Scene0"))
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

pub fn fast(asset_server: &Res<AssetServer>, path_selection: PathSelection) -> impl Bundle {
	(
		SceneBundle {
			scene: FAST_SCENE
				.get_or_init(|| asset_server.load("exported/Fast.gltf#Scene0"))
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

pub fn air(asset_server: &Res<AssetServer>, path_selection: PathSelection) -> impl Bundle {
	(
		SceneBundle {
			scene: AIR_SCENE
				.get_or_init(|| asset_server.load("exported/Air.gltf#Scene0"))
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

pub fn split(asset_server: &Res<AssetServer>, path_selection: PathSelection) -> impl Bundle {
	(
		SceneBundle {
			scene: SPLIT_SCENE
				.get_or_init(|| asset_server.load("exported/Split.gltf#Scene0"))
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
	mut commands: Commands,
	towers: Query<(&Tower, &Transform, &RangedShooter, &Damage)>,
	mut enemies: Query<(Entity, &mut Health, &Transform), Without<Air>>,
) {
	for (_, tower_pos, tower_range, tower_dmg) in towers.iter() {
		for (entity, mut hp, _) in enemies.iter_mut().filter(|(_, _, pos)| {
			(tower_pos.translation - pos.translation).length() < tower_range.0
		}) {
			hp.current -= tower_dmg.0;
			if hp.current <= 0 {
				commands.entity(entity).despawn();
			}
		}
	}
}

pub fn stone_tower(asset_server: &Res<AssetServer>, location: Vec3) -> impl Bundle {
	(
		SceneBundle {
			scene: asset_server.load("exported/Moai.gltf#Scene0"),
			transform: Transform::from_xyz(location.x, location.y, location.z)
				.looking_to(Vec3::X, Vec3::Y),
			..default()
		},
		ScreenSpaceAmbientOcclusionBundle { ..default() },
		TowerBundle {
			tower: Tower::Land,
			attack_speed: AttackSpeed(15.0),
			damage: Damage(30),
		},
		RangedShooter(5.0),
	)
}

#[derive(Resource)]
pub struct Selection {
	pub tower: Option<Tower>,
}

pub fn spawn_tower(
	mut clicks: EventReader<Click>,
	asset_server: Res<AssetServer>,
	cur_query: Query<&Transform, (With<SquareHighlight>, Without<Cursor>, Without<Camera3d>)>,
	mut tower_selection: ResMut<Selection>,
	mut commands: Commands,
) {
	for _ in clicks.iter() {
		let Ok(cur_trans) = cur_query.get_single() else {
			return;
		};
		let pos = round_to_grid(cur_trans.translation);

		println!("Spawning tower");

		let selection: &Selection = &tower_selection;
		match selection.tower {
			Some(Tower::Land) => {
				commands.spawn(stone_tower(&asset_server, pos));
			}
			Some(Tower::All) => todo!(),
			Some(Tower::Fire) => todo!(),
			Some(Tower::Water) => todo!(),
			Some(Tower::Air) => todo!(),
			Some(Tower::Laser) => todo!(),
			None => {
				return;
			}
		}
		tower_selection.tower = None;
	}
}

// ------------------------------ CURSOR ---------------------------------

#[derive(Component, Debug)]
pub struct Cursor;

#[derive(Component, Debug)]
pub struct SquareHighlight;

pub fn move_cursor_and_camera(
	button: Res<Input<MouseButton>>,
	mut win_query: Query<&mut Window>,
	mut cam_query: Query<
		(&Camera, &GlobalTransform, &mut Transform),
		(With<Camera3d>, Without<Cursor>, Without<VisualMarker>),
	>,
	mut cur_query: Query<
		&mut Transform,
		(
			With<Cursor>,
			Without<Camera3d>,
			Without<SquareHighlight>,
			Without<VisualMarker>,
		),
	>,
	mut hl_query: Query<
		&mut Transform,
		(
			With<SquareHighlight>,
			Without<Camera3d>,
			Without<Cursor>,
			Without<VisualMarker>,
		),
	>,
) {
	const CAMERA_LIMITS_MIN: Vec3 = Vec3::new(-45.0, 25.0, -30.0);
	const CAMERA_LIMITS_MAX: Vec3 = Vec3::new(-10.0, 25.0, 32.0);

	static CURSOR_REFERENCE: Mutex<Vec3> = Mutex::new(Vec3::ZERO);

	let mut inner = move || {
		let mut win = win_query.get_single_mut().ok()?;
		let (cam, g_trans, mut trans) = cam_query.get_single_mut().ok()?;
		let mut cur = cur_query.get_single_mut().ok()?;
		let mut hl = hl_query.get_single_mut().ok()?;

		// Move cursor
		let mouse = win.cursor_position()?;
		let ray = cam.viewport_to_world(g_trans, mouse)?;
		let dist = ray.intersect_plane(Vec3::new(0.0, 1.0, 0.0), Vec3::Y)?;
		let Vec3 { x, y: _, z } = ray.get_point(dist);

		let d_x = (((x + 15.0) / 2.0).round() as usize).min(15).max(0);
		let d_z = (((z + 19.0) / 2.0).round() as usize).min(19).max(0);
		let height_data = easy::HEIGHT_MAP[d_x][d_z];
		let d_y = 1.0 - height_data as f32 / 10.0;
		let v = Vec3::new(x, d_y, z);

		cur.translation = v;
		hl.translation = round_to_grid(v);

		// Move camera
		let mut v_cur = CURSOR_REFERENCE.lock().unwrap();
		if button.just_pressed(MouseButton::Left) {
			*v_cur = cur.translation;
		}
		if button.pressed(MouseButton::Left) {
			let diff = *v_cur - cur.translation;
			trans.translation = (trans.translation + diff)
				.max(CAMERA_LIMITS_MIN)
				.min(CAMERA_LIMITS_MAX);
		}

		win.cursor.visible = mouse.y < 64.0 && mouse.x < 64.0 * 8.0;

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
			commands.spawn(slow(&asset_server, path_selection));
		}
		EnemyType::Normal => {
			commands.spawn(normal(&asset_server, path_selection));
		}
		EnemyType::Fast => {
			commands.spawn(fast(&asset_server, path_selection));
		}
		EnemyType::Air => {
			commands.spawn(air(&asset_server, path_selection));
		}
		EnemyType::Split => {
			commands.spawn(split(&asset_server, path_selection));
		}
	}
}

// -------------------------- SPAWN DEFAULTS -----------------------------

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

#[derive(Component)]
pub struct VisualMarker;

pub fn visualise_height_map(
	height_map: &[[u8; 41]; 33],
	commands: &mut Commands,
	meshes: &mut ResMut<Assets<Mesh>>,
	materials: &mut ResMut<Assets<StandardMaterial>>,
) {
	let material = materials.add(Colour::rgb(1.0, 1.0, 1.0).into());
	for x in (0..33).step_by(1) {
		for y in (0..41).step_by(1) {
			let size = height_map[x][y] as f32 / 10.0 + 0.05;
			let mesh = meshes.add(Mesh::from(shape::Cube { size }));

			let x = x as f32 * 2.0 - 32.0;
			let y = y as f32 * 2.0 - 40.0;

			commands.spawn((
				PbrBundle {
					mesh,
					material: material.clone(),
					transform: Transform::from_xyz(x, 1.0, y),
					..default()
				},
				VisualMarker,
			));
		}
	}
}

// -------------------------------- UI -----------------------------------

pub fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
	let one: Handle<Image> = asset_server.load("exported/gui/one.png");
	let two: Handle<Image> = asset_server.load("exported/gui/two.png");
	let three: Handle<Image> = asset_server.load("exported/gui/three.png");
	let four: Handle<Image> = asset_server.load("exported/gui/four.png");
	let five: Handle<Image> = asset_server.load("exported/gui/five.png");
	let six: Handle<Image> = asset_server.load("exported/gui/six.png");
	let seven: Handle<Image> = asset_server.load("exported/gui/seven.png");
	let volcano: Handle<Image> = asset_server.load("exported/gui/volcano.png");

	commands
		.spawn(NodeBundle {
			style: Style {
				flex_direction: FlexDirection::Row,
				flex_basis: Val::Percent(100.),
				align_items: AlignItems::Start,
				justify_content: JustifyContent::FlexStart,
				..Default::default()
			},
			..Default::default()
		})
		.with_children(|parent| {
			let mut spawn_image = |texture| {
				let width = Val::Px(64.0);
				parent.spawn(ButtonBundle {
					image: UiImage {
						texture,
						..default()
					},
					style: Style {
						width,
						height: width,
						..Default::default()
					},
					..default()
				});
			};
			spawn_image(one);
			spawn_image(two);
			spawn_image(three);
			spawn_image(four);
			spawn_image(five);
			spawn_image(six);
			spawn_image(seven);
			spawn_image(volcano);
		});
}

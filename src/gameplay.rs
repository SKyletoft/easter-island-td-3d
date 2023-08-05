use std::{ops::Deref, sync::Mutex, time::Duration};

use bevy::{
	ecs::{query::ReadOnlyWorldQuery, system::EntityCommands},
	pbr::ScreenSpaceAmbientOcclusionBundle,
	prelude::*,
	utils::Instant,
};
use once_cell::sync::{Lazy, OnceCell};
use variantly::Variantly;

type Colour = bevy::prelude::Color;

use crate::easy::{self, Wave};

// ------------------------------- UTILS ---------------------------------

fn round_to_grid(v: Vec3) -> Vec3 {
	const ROUNDING_OFFSET: f32 = 1.0;
	const ROUND_TO: f32 = 2.0;

	let f = |x: f32| ((x + ROUNDING_OFFSET) / ROUND_TO).round() * ROUND_TO - ROUNDING_OFFSET;
	Vec3::new(f(v.x), v.y, f(v.z))
}

fn to_map_space(Vec3 { x, y: _, z }: Vec3) -> (usize, usize) {
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

fn with_height(v: Vec3) -> Vec3 {
	let (d_x, d_z) = to_map_space(v);
	let height_data = easy::HEIGHT_MAP[d_x][d_z];
	let d_y = 1.0 - height_data as f32 / 10.0;

	Vec3::new(v.x, d_y, v.z)
}

fn to_grid_with_height(v: Vec3) -> Vec3 {
	with_height(round_to_grid(v))
}

fn get_world_pos(
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

// ------------------------------ CLICKS ---------------------------------

#[derive(Copy, Clone, PartialEq, Event, Variantly)]
pub enum Click {
	Gui(Vec2),
	World(Vec3),
}

pub fn generate_clicks(
	button: Res<Input<MouseButton>>,
	windows: Query<&Window>,
	mut ev: EventWriter<Click>,
	mut cam_query: Query<
		(&Camera, &GlobalTransform),
		(With<Camera3d>, Without<Cursor>, Without<VisualMarker>),
	>,
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
		if p.y <= 64.0 {
			ev.send(Click::Gui(p));
		} else {
			let raw_ray = get_world_pos(p, &mut cam_query)
				.expect("How can a click be generated if the cursor isn't over the window?");
			let p3d = to_grid_with_height(raw_ray);
			dbg!(p3d);
			ev.send(Click::World(p3d));
		}
	}
}

// ------------------------------- PATH ----------------------------------

#[derive(Debug)]
pub struct Path(pub Vec<Vec3>);

impl Path {
	pub fn from_keyframes<const N: usize>(points: [(i32, i32, i32); N]) -> Self {
		let mut occupied_map = OCCUPIED_MAP.lock().unwrap();
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
			.inspect(|&v| {
				let (x, y) = to_map_space(v);
				occupied_map[x][y] = true;
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

// ------------------------------ TOWERS ---------------------------------

const BULLET_TRAVEL_TIME: Duration = Duration::from_millis(500);
pub static OCCUPIED_MAP: Mutex<[[bool; 20]; 16]> = Mutex::new([[false; 20]; 16]);

#[derive(Component, Copy, Clone, PartialEq, Debug)]
pub enum Tower {
	Land,
	All,
	Fire,
	Water,
	Air,
	Laser,
}

impl Tower {
	fn cost(&self) -> i32 {
		match self {
			Tower::Land => 30,
			Tower::All => 50,
			Tower::Fire => 56,
			Tower::Water => 50,
			Tower::Air => 70,
			Tower::Laser => 400,
		}
	}

	fn spawn<'a, 'b, 'c>(
		&self,
		location: Vec3,
		asset_server: &Res<AssetServer>,
		commands: &'c mut Commands<'a, 'b>,
		materials: &mut ResMut<Assets<StandardMaterial>>,
	) -> EntityCommands<'a, 'b, 'c> {
		match self {
			Tower::Land => commands.spawn(land_tower(location)),
			Tower::All => commands.spawn(all_tower(location)),
			Tower::Fire => commands.spawn(all_tower(location)),
			Tower::Water => commands.spawn(all_tower(location)),
			Tower::Air => commands.spawn(all_tower(location)),
			Tower::Laser => commands.spawn(all_tower(location)),
		}
	}
}

#[derive(Component, Debug, Deref, DerefMut)]
pub struct RangedShooterLand(f32);

#[derive(Component, Debug, Deref, DerefMut)]
pub struct RangedShooterAir(f32);

#[derive(Component, Debug, Deref, DerefMut)]
pub struct AoE(f32);

#[derive(Component, Debug)]
pub struct AttackSpeed(Timer);

#[derive(Component, Debug)]
pub struct Damage(i32);

#[derive(Component, Debug)]
pub struct Upgraded(i32);

#[derive(Bundle, Debug)]
pub struct TowerBundle {
	pub tower: Tower,
	pub attack_speed: AttackSpeed,
	pub damage: Damage,
	pub level: Upgraded,
}

#[derive(Component, Debug)]
pub struct Bullet {
	pub target: Entity,
	pub start_location: Vec3,
	pub target_location: Vec3,
	pub spawned_at: Instant,
	pub damage: i32,
}

static BULLET_MODEL: OnceCell<(Handle<Mesh>, Handle<StandardMaterial>)> = OnceCell::new();

pub fn init_bullet_model(
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
) {
	BULLET_MODEL.get_or_init(|| {
		(
			meshes.add(
				shape::UVSphere {
					radius: 0.1,
					..default()
				}
				.into(),
			),
			materials.add(Colour::rgb(1.0, 1.0, 1.0).into()),
		)
	});
}

static LAND_SCENE: OnceCell<Handle<Scene>> = OnceCell::new();
static ALL_SCENE: OnceCell<Handle<Scene>> = OnceCell::new();
static WATER_SCENE: OnceCell<Handle<Scene>> = OnceCell::new();
static FIRE_SCENE: OnceCell<Handle<Scene>> = OnceCell::new();
static LASER_SCENE: OnceCell<Handle<Scene>> = OnceCell::new();

pub fn init_tower_models(asset_server: Res<AssetServer>) {
	LAND_SCENE.get_or_init(|| asset_server.load("exported/Moai.gltf#Scene0"));
	ALL_SCENE.get_or_init(|| asset_server.load("exported/Moai.gltf#Scene0"));
	WATER_SCENE.get_or_init(|| asset_server.load("exported/Moai.gltf#Scene0"));
	FIRE_SCENE.get_or_init(|| asset_server.load("exported/Moai.gltf#Scene0"));
	LASER_SCENE.get_or_init(|| asset_server.load("exported/Moai.gltf#Scene0"));
}

pub fn move_bullets(
	mut commands: Commands,
	mut bullets: Query<(&mut Transform, &Bullet, Entity)>,
	mut enemies: Query<&mut Health, With<Enemy>>,
	time: Res<Time>,
) {
	let now = time.startup() + time.elapsed();
	for (mut trans, bullet, entity) in bullets.iter_mut() {
		let progress = ((now - bullet.spawned_at).as_secs_f32() / BULLET_TRAVEL_TIME.as_secs_f32())
			.clamp(0.0, 1.0);
		let new_pos = bullet.start_location.lerp(bullet.target_location, progress);
		trans.translation = new_pos;

		if progress >= 1.0 {
			commands.entity(entity).despawn_recursive();

			let mut health = match enemies.get_mut(bullet.target) {
				Ok(h) => h,
				Err(_) => {
					continue;
				}
			};
			health.current -= bullet.damage;

			if health.current <= 0 {
				commands.entity(bullet.target).despawn_recursive();
			}
		}
	}
}

pub fn land_attack(
	commands: Commands,
	towers: Query<(
		&Tower,
		&Transform,
		&RangedShooterLand,
		&Damage,
		&mut AttackSpeed,
	)>,
	enemies: Query<
		(Entity, &Transform, &Progress, &Speed, &PathSelection),
		(With<Enemy>, Without<Air>),
	>,
	time: Res<Time>,
) {
	ranged_attack(commands, towers, enemies, time)
}

pub fn air_attack(
	commands: Commands,
	towers: Query<(
		&Tower,
		&Transform,
		&RangedShooterAir,
		&Damage,
		&mut AttackSpeed,
	)>,
	enemies: Query<
		(Entity, &Transform, &Progress, &Speed, &PathSelection),
		(With<Enemy>, With<Air>),
	>,
	time: Res<Time>,
) {
	ranged_attack(commands, towers, enemies, time)
}

fn ranged_attack<Range, Filter>(
	mut commands: Commands,
	mut towers: Query<(&Tower, &Transform, &Range, &Damage, &mut AttackSpeed)>,
	mut enemies: Query<
		(Entity, &Transform, &Progress, &Speed, &PathSelection),
		(With<Enemy>, Filter),
	>,
	time: Res<Time>,
) where
	Range: Component + Deref<Target = f32>,
	Filter: ReadOnlyWorldQuery,
{
	let (mesh, material) = BULLET_MODEL
		.get()
		.expect("Bullet model should've been initialised");
	let spawned_at = time.startup() + time.elapsed();

	for (_, tower_pos, tower_range, tower_dmg, mut tower_timer) in towers.iter_mut() {
		if !tower_timer.0.tick(time.delta()).just_finished() {
			continue;
		}

		let range: f32 = *tower_range.deref();
		let Some(((entity, _, prog, speed, track), _)) = enemies
			.iter_mut()
			.map(|enemy| {
				let dist = enemy.1.translation.distance(tower_pos.translation);
				(enemy, dist)
			})
			.filter(|(_, dist)| *dist < range)
			.min_by_key(|(_, dist)| {
				// Cast distance to an integer for total ordering
				(*dist * 1000.0) as i32
			}) else {
				continue;
			};

		let target_progress = prog.0 + speed.0 * BULLET_TRAVEL_TIME.as_secs_f32();
		let target_location =
			easy::PATHS[track.0].interpolate(target_progress) + Vec3::new(0.0, 0.5, 0.0);

		commands.spawn((
			PbrBundle {
				mesh: mesh.clone(),
				material: material.clone(),
				transform: Transform {
					translation: tower_pos.translation,
					..default()
				},
				..default()
			},
			Bullet {
				target: entity,
				start_location: tower_pos.translation + Vec3::new(0.0, 1.5, 0.0),
				target_location,
				spawned_at,
				damage: tower_dmg.0,
			},
		));
	}
}

pub fn land_tower(location: Vec3) -> impl Bundle {
	(
		SceneBundle {
			scene: LAND_SCENE
				.get()
				.expect("Land scene should've been loaded")
				.clone(),
			transform: Transform::from_xyz(location.x, location.y, location.z)
				.looking_to(Vec3::X, Vec3::Y),
			..default()
		},
		ScreenSpaceAmbientOcclusionBundle { ..default() },
		TowerBundle {
			tower: Tower::Land,
			attack_speed: AttackSpeed(Timer::from_seconds(0.8, TimerMode::Repeating)),
			damage: Damage(30),
			level: Upgraded(0),
		},
		RangedShooterLand(5.0),
	)
}

pub fn all_tower(location: Vec3) -> impl Bundle {
	(
		SceneBundle {
			scene: ALL_SCENE
				.get()
				.expect("All scene should've been loaded")
				.clone(),
			transform: Transform::from_xyz(location.x, location.y, location.z)
				.looking_to(Vec3::X, Vec3::Y),
			..default()
		},
		ScreenSpaceAmbientOcclusionBundle { ..default() },
		TowerBundle {
			tower: Tower::All,
			attack_speed: AttackSpeed(Timer::from_seconds(0.15, TimerMode::Repeating)),
			damage: Damage(30),
			level: Upgraded(0),
		},
		RangedShooterLand(5.0),
		RangedShooterAir(5.0),
	)
}

#[derive(Resource)]
pub struct Banking {
	pub selection: Option<Tower>,
	pub balance: i32,
}

pub fn spawn_tower(
	mut clicks: EventReader<Click>,
	mut commands: Commands,
	asset_server: Res<AssetServer>,
	mut tower_selection: ResMut<Banking>,
	mut materials: ResMut<Assets<StandardMaterial>>,
	mut balance_label: Query<&mut Text, With<BalanceLabel>>,
) {
	let mut balance_label = balance_label.get_single_mut().unwrap();
	for location in clicks.iter().filter_map(|ev| ev.world()) {
		if location.y < 0.0 {
			continue;
		}

		let Some(selection) = tower_selection.selection else {
			continue;
		};

		let cost = selection.cost();
		if tower_selection.balance < cost {
			continue;
		}

		selection.spawn(location, &asset_server, &mut commands, &mut materials);

		tower_selection.selection = None;
		tower_selection.balance -= cost;

		balance_label.sections[0].value = format!("{}", tower_selection.balance);
	}
}

// ------------------------------ CURSOR ---------------------------------

#[derive(Component, Debug)]
pub struct Cursor;

#[derive(Component, Debug)]
pub struct Range;

#[derive(Component, Debug)]
pub struct SquareHighlight;

// move_camera
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
	const CAMERA_LIMITS_MIN: Vec3 = Vec3::new(-45.0, 10.0, -30.0);
	const CAMERA_LIMITS_MAX: Vec3 = Vec3::new(-10.0, 10.0, 32.0);

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
		let raw_ray = ray.get_point(dist);

		let v = with_height(raw_ray);

		let hl_coord =
			if (-20.0..20.0).contains(&v.z) && (-16.0..16.0).contains(&v.x) && v.y >= 0.11 {
				round_to_grid(v)
			} else {
				Vec3 {
					x: 0.0,
					y: -5.0,
					z: 0.0,
				}
			};

		cur.translation = v;
		hl.translation = hl_coord;

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
			commands.spawn(slow(path_selection));
		}
		EnemyType::Normal => {
			commands.spawn(normal(path_selection));
		}
		EnemyType::Fast => {
			commands.spawn(fast(path_selection));
		}
		EnemyType::Air => {
			commands.spawn(air(path_selection));
		}
		EnemyType::Split => {
			commands.spawn(split(path_selection));
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

// -------------------------------- UI -----------------------------------

#[derive(Component, Debug)]
pub struct BalanceLabel;

#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub enum ClickType {
	Buy(Tower),
	Volcano,
	Sell,
	Upgrade,
	Next,
}

static ONE_TEXTURE: OnceCell<Handle<Image>> = OnceCell::new();
static TWO_TEXTURE: OnceCell<Handle<Image>> = OnceCell::new();
static THREE_TEXTURE: OnceCell<Handle<Image>> = OnceCell::new();
static FOUR_TEXTURE: OnceCell<Handle<Image>> = OnceCell::new();
static FIVE_TEXTURE: OnceCell<Handle<Image>> = OnceCell::new();
static SIX_TEXTURE: OnceCell<Handle<Image>> = OnceCell::new();
static VOLCANO_TEXTURE: OnceCell<Handle<Image>> = OnceCell::new();
static SLOW_TEXTURE: OnceCell<Handle<Image>> = OnceCell::new();
static NORMAL_TEXTURE: OnceCell<Handle<Image>> = OnceCell::new();
static FAST_TEXTURE: OnceCell<Handle<Image>> = OnceCell::new();
static AIR_TEXTURE: OnceCell<Handle<Image>> = OnceCell::new();
static SPLIT_TEXTURE: OnceCell<Handle<Image>> = OnceCell::new();
static BOSS_TEXTURE: OnceCell<Handle<Image>> = OnceCell::new();

pub fn init_textures(asset_server: Res<AssetServer>) {
	ONE_TEXTURE.get_or_init(|| asset_server.load("exported/gui/one.png"));
	TWO_TEXTURE.get_or_init(|| asset_server.load("exported/gui/two.png"));
	THREE_TEXTURE.get_or_init(|| asset_server.load("exported/gui/three.png"));
	FOUR_TEXTURE.get_or_init(|| asset_server.load("exported/gui/four.png"));
	FIVE_TEXTURE.get_or_init(|| asset_server.load("exported/gui/five.png"));
	SIX_TEXTURE.get_or_init(|| asset_server.load("exported/gui/six.png"));
	VOLCANO_TEXTURE.get_or_init(|| asset_server.load("exported/gui/volcano.png"));
	SLOW_TEXTURE.get_or_init(|| asset_server.load("exported/gui/slow.png"));
	NORMAL_TEXTURE.get_or_init(|| asset_server.load("exported/gui/normal.png"));
	FAST_TEXTURE.get_or_init(|| asset_server.load("exported/gui/fast.png"));
	AIR_TEXTURE.get_or_init(|| asset_server.load("exported/gui/air.png"));
	SPLIT_TEXTURE.get_or_init(|| asset_server.load("exported/gui/split.png"));
	BOSS_TEXTURE.get_or_init(|| asset_server.load("exported/gui/boss.png"));
}

pub fn setup_ui(mut commands: Commands) {
	commands
		.spawn(NodeBundle {
			style: Style {
				flex_direction: FlexDirection::Column,
				..default()
			},
			..default()
		})
		.with_children(|p| {
			p.spawn(NodeBundle {
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
				const SIZE: f32 = 64.0;
				let mut spawn_image = |texture, ct| {
					let width = Val::Px(SIZE);
					parent.spawn((
						ButtonBundle {
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
						},
						ct,
					));
				};
				spawn_image(
					ONE_TEXTURE.get().unwrap().clone(),
					ClickType::Buy(Tower::Land),
				);
				spawn_image(
					TWO_TEXTURE.get().unwrap().clone(),
					ClickType::Buy(Tower::All),
				);
				spawn_image(
					THREE_TEXTURE.get().unwrap().clone(),
					ClickType::Buy(Tower::Fire),
				);
				spawn_image(
					FOUR_TEXTURE.get().unwrap().clone(),
					ClickType::Buy(Tower::Water),
				);
				spawn_image(
					FIVE_TEXTURE.get().unwrap().clone(),
					ClickType::Buy(Tower::Air),
				);
				spawn_image(
					SIX_TEXTURE.get().unwrap().clone(),
					ClickType::Buy(Tower::Laser),
				);
				spawn_image(VOLCANO_TEXTURE.get().unwrap().clone(), ClickType::Volcano);
				parent.spawn((
					TextBundle {
						text: Text::from_section(
							"0",
							TextStyle {
								font_size: SIZE - 12.0,
								color: Colour::BLACK,
								..default()
							},
						),
						style: Style {
							height: Val::Px(SIZE),
							min_width: Val::Px(SIZE),
							align_items: AlignItems::Center,
							justify_items: JustifyItems::Center, //content?
							..default()
						},
						background_color: Colour::WHITE.into(),
						..default()
					},
					BalanceLabel,
				));
			});
			p.spawn(NodeBundle {
				style: Style {
					flex_direction: FlexDirection::Row,
					flex_basis: Val::Percent(100.),
					align_items: AlignItems::End,
					justify_content: JustifyContent::FlexStart,
					..Default::default()
				},
				..Default::default()
			})
			.with_children(|parent| {
				let mut spawn_image = |texture| {
					let width = Val::Px(64.0);
					parent.spawn((
						ButtonBundle {
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
						},
						ClickType::Next,
					));
				};
				spawn_image(VOLCANO_TEXTURE.get().unwrap().clone());
			});
		});
}

pub fn run_shop(
	buttons: Query<(&Interaction, &ClickType), (With<Button>, Changed<Interaction>)>,
	mut banking: ResMut<Banking>,
) {
	for (_, ct) in buttons.iter().filter(|(&i, _)| i == Interaction::Pressed) {
		println!("{:?}", ct);
		match ct {
			ClickType::Buy(t) => {
				banking.selection = Some(*t);
			}
			ClickType::Volcano => todo!(),
			ClickType::Sell => todo!(),
			ClickType::Upgrade => todo!(),
			ClickType::Next => todo!(),
		}
	}
}

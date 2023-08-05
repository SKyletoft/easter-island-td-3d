use std::{
	ops::Deref,
	sync::Mutex,
	time::{Duration, Instant},
};

use bevy::{
	ecs::{query::ReadOnlyWorldQuery, system::EntityCommands},
	pbr::ScreenSpaceAmbientOcclusionBundle,
	prelude::*,
};
use once_cell::sync::OnceCell;

use crate::{
	easy,
	gameplay::{
		enemies::{Air, Enemy, Health, PathSelection, Progress, Speed},
		ui::{BalanceLabel, Click},
	},
};

type Colour = Color;

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
			})
		else {
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

use std::sync::Mutex;

use bevy::prelude::*;
use once_cell::sync::OnceCell;
use variantly::Variantly;

use crate::gameplay::{
	cursor::Cursor,
	towers::{Banking, Tower},
	utils::{self, VisualMarker},
};

type Colour = Color;

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
			let raw_ray = utils::get_world_pos(p, &mut cam_query)
				.expect("How can a click be generated if the cursor isn't over the window?");
			let p3d = utils::to_grid_with_height(raw_ray);
			dbg!(p3d);
			ev.send(Click::World(p3d));
		}
	}
}

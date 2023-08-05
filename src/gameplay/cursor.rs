use std::sync::Mutex;

use bevy::prelude::*;

use super::utils::VisualMarker;
use crate::gameplay::utils;

type Colour = Color;

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

		let v = utils::with_height(raw_ray);

		let hl_coord =
			if (-20.0..20.0).contains(&v.z) && (-16.0..16.0).contains(&v.x) && v.y >= 0.11 {
				utils::round_to_grid(v)
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

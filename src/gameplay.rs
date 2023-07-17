use bevy::{prelude::*, input::mouse::MouseWheel};

use crate::easy;

#[derive(Component, Debug)]
pub struct Slow(pub usize);

#[derive(Component, Debug)]
pub struct Normal(pub usize);

#[derive(Component, Debug)]
pub struct Fast(pub usize);

#[derive(Component, Debug)]
pub struct Air(pub usize);

#[derive(Component, Debug)]
pub struct SplitParent(pub usize);

#[derive(Component, Debug)]
pub struct SplitChild(pub usize);

#[derive(Component, Debug)]
pub enum Enemy {
	Slow(Slow),
	Normal(Normal),
	Fast(Fast),
	Air(Air),
	SplitParent(SplitParent),
	SplitChild(SplitChild),
}

#[derive(Component, Debug)]
pub struct Health {
	pub max: u32,
	pub current: u32,
}

#[derive(Component, Debug)]
pub struct Cursor;

#[derive(Component, Debug)]
pub struct VCursor;

#[derive(Component, Debug)]
pub struct Progress(pub f32);

#[derive(Component, Debug)]
pub struct Speed(pub f32);

#[derive(Bundle, Debug)]
pub struct EnemyBundle {
	pub enemy: Enemy,
	pub health: Health,
	pub progress: Progress,
}

pub fn move_enemies(
	mut query: Query<(&mut Transform, &Speed, &mut Progress), With<Enemy>>,
	d_time: Res<Time>,
) {
	for (mut loc, speed, mut prog) in query.iter_mut() {
		prog.0 += speed.0 * d_time.delta_seconds();
		loc.translation = interpolate(prog.0, &easy::MIDDLE_PATH);

		let towards = interpolate(prog.0 + 0.02, &easy::MIDDLE_PATH);
		loc.look_at(towards, Vec3::Y);
	}
}

pub fn move_cursor_and_camera(
	button: Res<Input<MouseButton>>,
	mut scroll_evr: EventReader<MouseWheel>,
	win_query: Query<&Window>,
	mut cam_query: Query<
		(&Camera, &GlobalTransform, &mut Transform),
		(With<Camera3d>, Without<Cursor>, Without<VCursor>),
	>,
	mut cur_query: Query<&mut Transform, (With<Cursor>, Without<Camera3d>, Without<VCursor>)>,
	mut v_cur_query: Query<&mut Transform, (With<VCursor>, Without<Camera3d>, Without<Cursor>)>,
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
			dbg!(&v_cur.translation);
		}
		if button.pressed(MouseButton::Left) {
			let diff = v_cur.translation - cur.translation;
			trans.translation += diff;
		}

		// dbg!(trans.scale);
		// for ev in scroll_evr.iter() {
		// 	dbg!(&ev);
		// 	let dist = match ev.unit {
		// 		MouseScrollUnit::Line => ev.y * 0.01,
		// 		MouseScrollUnit::Pixel => ev.y,
		// 	};

		// 	trans.scale += dist;
		// 	dbg!(trans.scale);
		// }

		Some(())
	})();
}

pub fn interpolate(dt: f32, range: &[Vec3]) -> Vec3 {
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

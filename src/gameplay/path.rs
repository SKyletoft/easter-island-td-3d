use bevy::prelude::*;

use crate::gameplay::{towers, utils};

type Colour = Color;

// ------------------------------- PATH ----------------------------------

#[derive(Debug)]
pub struct Path(pub Vec<Vec3>);

impl Path {
	pub fn from_keyframes<const N: usize>(points: [(i32, i32, i32); N]) -> Self {
		let mut occupied_map = towers::OCCUPIED_MAP.lock().unwrap();
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
				let (x, y) = utils::to_map_space(v);
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

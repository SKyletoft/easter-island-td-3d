#![allow(clippy::type_complexity)]

use bevy::{
	input::mouse::MouseWheel,
	prelude::*,
};

mod easy;
mod normal;
mod hard;
mod gameplay;


fn main() {
	App::new()
		.add_plugins(DefaultPlugins)
		.add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
			.add_systems(Startup, easy::setup)
		.add_systems(Update, (gameplay::move_enemies, gameplay::move_cursor_and_camera))
		.run();
}

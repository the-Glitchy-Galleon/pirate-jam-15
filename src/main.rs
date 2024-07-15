#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::prelude::*;
use fps_counter::FpsCounterPlugin;
use free_camera::FreeCameraPlugin;
use scene_preview::ScenePreviewPlugin;

mod runner;

mod fps_counter;
mod free_camera;
mod scene_preview;

fn main() -> AppExit {
	let mut app = runner::create_app();

	app.add_plugins(DefaultPlugins);

	app.add_plugins(ScenePreviewPlugin)
		.add_plugins(FreeCameraPlugin)
		.add_plugins(FpsCounterPlugin);

	runner::run_app(&mut app)
}

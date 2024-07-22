#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
// use game::GamePlugin;
use tooling::prelude::*;

pub mod framework;
mod game;
mod runner;
pub mod tooling;

fn main() -> AppExit {
    let mut app = runner::create_app();

    /* Add the base plugins */
    app.add_plugins((EguiPlugin, FpsCounterPlugin, WorldInspectorPlugin::new()));

    /* Add the plugins depending on our config */
    app.add_plugins(ScenePreviewPlugin);

    runner::run_app(&mut app)
}

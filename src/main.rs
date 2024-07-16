#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use tooling::prelude::*;

mod runner;
pub mod tooling;

fn main() -> AppExit {
    let mut app = runner::create_app();

    app.add_plugins((EguiPlugin, WorldInspectorPlugin::new()))
        .add_plugins(CursorGrabAndCenterPlugin)
        .add_plugins(PointerCaptureCheckPlugin)
        .add_plugins(FreeCameraPlugin)
        .add_plugins(FpsCounterPlugin)
        .add_plugins(ScenePreviewPlugin);

    runner::run_app(&mut app)
}

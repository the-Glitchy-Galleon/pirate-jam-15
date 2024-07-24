#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use game::GamePlugin;
use tooling::prelude::*;

pub mod framework;
mod game;
mod runner;
pub mod tooling;

fn main() -> AppExit {
    let (mut app, init_game) = runner::create_app();

    /* Add the base plugins */
    app.add_plugins((EguiPlugin, FpsCounterPlugin));

    /* Add the plugins depending on our config */
    if init_game {
        app.add_plugins(GamePlugin);
    }

    runner::run_app(&mut app)
}

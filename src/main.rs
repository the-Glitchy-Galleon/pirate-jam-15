#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use game::{
    level::{self, UserDefinedStartupLevel},
    GamePlugin,
};
use tooling::prelude::*;

pub mod framework;
mod game;
mod runner;
pub mod tooling;

pub struct GameRunArgs {
    pub init: bool,
    pub level: Option<String>,
}

impl Default for GameRunArgs {
    fn default() -> Self {
        Self {
            init: true,
            level: None,
        }
    }
}

fn main() -> AppExit {
    let (mut app, run_args) = runner::create_app();

    /* Add the base plugins */
    app.add_plugins((EguiPlugin, FpsCounterPlugin));

    /* Add the plugins depending on our config */
    if run_args.init {
        app.add_plugins(GamePlugin);

        if let Some(level) = run_args.level {
            app.insert_resource(UserDefinedStartupLevel(level));
            app.add_systems(Startup, level::load_user_defined_startup_level);
        }
    }
    runner::run_app(&mut app)
}

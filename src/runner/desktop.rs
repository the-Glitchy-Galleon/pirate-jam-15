use crate::tooling::prelude::*;
use bevy::prelude::*;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about = "Runs the game", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    #[command()]
    Editor,
}

pub fn create_app() -> (App, bool) {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);

    // Check for editor
    let args = Cli::parse();
    match args.command {
        Some(Command::Editor) => {
            // std::env::set_var("RUST_BACKTRACE", "1");
            app.add_plugins(LevelEditorPlugin);
            (app, false)
        }
        None => (app, true),
    }
}

pub fn run_app(app: &mut App) -> AppExit {
    app.run()
}

use bevy::prelude::*;

pub fn create_app() -> App {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app
}

pub fn run_app(app: &mut App) -> AppExit {
    app.run()
}

use crate::GameRunArgs;
use bevy::prelude::*;
use web_sys::Document;

static LAUNCHER_TITLE: &'static str = "pirate ship";

fn set_window_title(title: &str) {
    web_sys::window()
        .map(|w| w.document())
        .flatten()
        .expect("Unable to get DOM")
        .set_title(title);
}

pub fn create_app() -> (App, bool) {
    let mut app = App::new();

    app.add_plugins(
        DefaultPlugins
            .set(AssetPlugin {
                // Todo: Need to disable the meta check for drag-and-drop loading to work in webgl
                // take this out once not required anymore
                meta_check: bevy::asset::AssetMetaCheck::Never,
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    fit_canvas_to_parent: true,
                    canvas: Some("#bevyscreen".to_owned()),
                    ..default()
                }),
                ..default()
            }),
    );

    (app, GameRunArgs::default())
}

fn setup_dom(document: &Document) {
    let load = document
        .query_selector("#bevyload")
        .expect("Cannot query for canvas element.");
    let load = load.expect("Expected load screen");

    let canvas = document
        .create_element("canvas")
        .expect("Cannot create canvas.");

    canvas.set_id("bevyscreen");

    load.insert_adjacent_element("beforebegin", &canvas)
        .expect("Cannot insert canvas");
    load.remove();
}

pub fn run_app(app: &mut App) -> AppExit {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    set_window_title(LAUNCHER_TITLE);
    setup_dom(&document);

    app.run()
}

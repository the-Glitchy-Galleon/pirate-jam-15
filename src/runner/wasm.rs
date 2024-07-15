use bevy::prelude::*;
use stylist::yew::styled_component;
use stylist::{css, global_style};
use yew::prelude::*;

static LAUNCHER_TITLE: &'static str = "pirate ship";

fn set_window_title(title: &str) {
    web_sys::window()
        .map(|w| w.document())
        .flatten()
        .expect("Unable to get DOM")
        .set_title(title);
}

fn set_global_css() {
    global_style! {
        r#"
        html {
            min-height: 100%;
            position: relative;
        }
        body {
            height: 100%;
            padding: 0;
            margin: 0;
        }
        "#
    }
    .expect("Unable to mount global style");
}

#[styled_component(Root)]
fn view() -> Html {
    set_window_title(LAUNCHER_TITLE);
    set_global_css();

    let css = css!(
        r#"
        position: absolute;
        overflow: hidden;
        width: 100%;
        height: 100%;
        "#
    );

    html! {
        <div class={ css }>
            <canvas id="bevy"></canvas>
        </div>
    }
}

pub fn create_app() -> App {
    let mut app = App::new();

    app.add_plugins(bevy_web_file_drop::WebFileDropPlugin);

    app.add_plugins(DefaultPlugins.set(AssetPlugin {
        // Todo: Need to disable the meta check for drag-and-drop loading to work in webgl
        // take this out once not required anymore
        meta_check: bevy::asset::AssetMetaCheck::Never,
        ..default()
    }));

    app
}

pub fn run_app(app: &mut App) -> AppExit {
    // Mount the DOM
    yew::Renderer::<Root>::new().render();

    app.run()
}

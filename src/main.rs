#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::prelude::*;
use fps_counter::FpsCounterPlugin;
use free_camera::FreeCameraPlugin;
use scene_preview::ScenePreviewPlugin;

mod fps_counter;
mod free_camera;
mod scene_preview;

#[cfg(target_family = "wasm")]
mod web {
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
}

fn main() -> AppExit {
	// Mount the DOM
	#[cfg(target_family = "wasm")]
	yew::Renderer::<web::Root>::new().render();

	// Start the Bevy App
	let mut app = App::new();

	cfg_if::cfg_if! {
		// Probably overkill since the tooling won't end up in the web build,
		// but it's the only thing this app can do right now ¯\_(ツ)_/¯
		if #[cfg(target_family = "wasm")] {
			app.add_plugins((
				bevy_web_file_drop::WebFileDropPlugin, // must be added before AssetPlugin
				// bevy_blob_loader::BlobLoaderPlugin, // already added by WebFileDropPlugin
				DefaultPlugins.set(AssetPlugin {
					meta_check: bevy::asset::AssetMetaCheck::Never,
					..default()
				})
			));
		} else {
			app.add_plugins(DefaultPlugins);
		}
	}

	app.add_plugins(ScenePreviewPlugin)
		.add_plugins(FreeCameraPlugin)
		.add_plugins(FpsCounterPlugin);

	app.run()
}

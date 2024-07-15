use bevy::prelude::*;
use fps_counter::FpsCounterPlugin;
use free_camera::FreeCameraPlugin;
use scene_preview::ScenePreviewPlugin;

mod fps_counter;
mod free_camera;
mod scene_preview;

fn main() -> AppExit {
	#[allow(unused_mut)]
	let mut app = App::new();

	#[cfg(target_family = "wasm")]
	{
		// Probably overkill since the tooling won't end up in the web build,
		// but it's the only thing this app can do right now ¯\_(ツ)_/¯
		app.add_plugins((
			// bevy_blob_loader::BlobLoaderPlugin, // already added by WebFileDropPlugin
			bevy_web_file_drop::WebFileDropPlugin, // must be added before AssetPlugin
			DefaultPlugins.set(AssetPlugin {
				meta_check: bevy::asset::AssetMetaCheck::Never,
				..default()
			}),
		));
	}
	
	#[cfg(not(target_family = "wasm"))]
	{
		app.add_plugins(DefaultPlugins);
	}

	app.add_plugins(ScenePreviewPlugin)
		.add_plugins(FreeCameraPlugin)
		.add_plugins(FpsCounterPlugin);

	app.run()
}

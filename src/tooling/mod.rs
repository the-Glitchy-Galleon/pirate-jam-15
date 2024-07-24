pub mod fps_counter;
pub mod free_camera;

#[cfg(not(target_family = "wasm"))]
pub mod editor;

pub mod prelude {
    #[cfg(not(target_family = "wasm"))]
    pub use super::editor::LevelEditorPlugin;

    pub use super::{fps_counter::FpsCounterPlugin, free_camera::FreeCameraPlugin};
}

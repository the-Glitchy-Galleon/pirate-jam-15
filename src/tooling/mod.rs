pub mod plugins;

#[cfg(not(target_family = "wasm"))]
pub mod editor;

pub mod prelude {
    #[cfg(not(target_family = "wasm"))]
    pub use super::editor::LevelEditorPlugin;

    pub use super::plugins::{
        cursor_grab_and_center::CursorGrabAndCenterPlugin,
        fps_counter::FpsCounterPlugin,
        free_camera::FreeCameraPlugin,
        pointer_capture_check::{GlobalUiState, PointerCaptureCheckPlugin},
        scene_preview::ScenePreviewPlugin,
    };
}

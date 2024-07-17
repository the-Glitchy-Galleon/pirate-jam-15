pub mod plugins;

pub mod prelude {
    pub use super::plugins::{
        cursor_grab_and_center::CursorGrabAndCenterPlugin, fps_counter::FpsCounterPlugin,
        free_camera::FreeCameraPlugin, pointer_capture_check::PointerCaptureCheckPlugin,
        scene_preview::ScenePreviewPlugin,
    };
}

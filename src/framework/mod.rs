pub mod audio;
pub mod easing;
pub mod global_ui_state;
pub mod grid;
pub mod logical_cursor;
pub mod tilemap;
pub mod tileset;

pub mod prelude {
    // Full audio control should be possible by just using these definitions, and not the audio libs directly
    pub use super::{
        audio::{
            Audio, AudioAsset, AudioChannel, AudioChannels, AudioControl, AudioEmitterBundle,
            AudioInstanceControl, AudioInstanceState, AudioPlugin, AudioReceiver,
            AudioSpatialRange, PlaybackState, Volume,
        },
        easing::Easing,
        global_ui_state::{GlobalUiState, GlobalUiStatePlugin, NoPointerCapture},
        logical_cursor::{LogicalCursorPlugin, LogicalCursorPosition},
        tilemap::Tilemap,
        tileset::Tileset,
    };
}

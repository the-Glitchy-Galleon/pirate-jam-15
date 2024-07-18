pub mod audio;
pub mod easing;
pub mod logical_cursor;

pub mod prelude {
    // Full audio control should be possible by just using these definitions, and not the audio libs directly
    pub use super::audio::{
        Audio, AudioAsset, AudioChannel, AudioChannels, AudioEmitterBundle, AudioInstanceControl,
        AudioInstanceState, AudioPlugin, AudioSpatialRange,
    };
    pub use bevy_kira_audio::prelude::AudioControl as _;
    pub use bevy_kira_audio::prelude::{AudioReceiver, PlaybackState, Volume};

    pub use super::easing::Easing;

    pub use super::logical_cursor::{LogicalCursorPlugin, LogicalCursorPosition};
}

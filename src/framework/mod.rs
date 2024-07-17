pub mod audio;

pub mod prelude {
    // re-export some bevy_kira_audio definitions here,
    // to prevent using functionality that isn't supported by the framework.
    pub use super::audio::AudioPlugin;
    pub use bevy_kira_audio::prelude::AudioControl as _;
    pub use bevy_kira_audio::prelude::{
        Audio, AudioEmitter, AudioInstance, AudioReceiver, AudioSource, AudioTween, PlaybackState,
    };
    // Use `AudioAsset` to disambiguate from the `AudioSource` exported by bevy::prelude
    pub use bevy_kira_audio::prelude::AudioSource as AudioAsset;
}

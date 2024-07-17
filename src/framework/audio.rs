use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy_kira_audio::AudioPlugin)
            .init_asset::<bevy_kira_audio::AudioSource>()
            .insert_resource(SpatialAudio { max_distance: 15. });
    }
}

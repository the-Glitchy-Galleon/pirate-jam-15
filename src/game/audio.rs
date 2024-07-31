use crate::framework::{
    audio::{Audio, AudioAsset, AudioChannel, AudioChannels, LoopConfig, Volume},
    easing::Easing,
};
use bevy::{asset::LoadState, prelude::*, time::Real};
use std::time::Duration;

#[derive(Resource)]
pub struct AudioAssets {
    pub silence: Handle<AudioAsset>,
    pub theme: Handle<AudioAsset>,
    pub activate_1: Handle<AudioAsset>,
    pub button_click_1: Handle<AudioAsset>,
    pub collect_minion_1: Handle<AudioAsset>,
    pub deactivate_1: Handle<AudioAsset>,
    pub destruction_1: Handle<AudioAsset>,
    pub minion_kill_1: Handle<AudioAsset>,
    pub player_kill_1: Handle<AudioAsset>,
    pub respawn_1: Handle<AudioAsset>,
    pub send_minion_1: Handle<AudioAsset>,
}

pub fn load_audio_assets(mut cmd: Commands, ass: Res<AssetServer>) {
    let silence = ass.load("silence.ogg");
    let theme = ass.load("Gamejam - Theme - 1.ogg");
    let activate_1 = ass.load("sfx/activate_1.ogg");
    let button_click_1 = ass.load("sfx/button click_1.ogg");
    let collect_minion_1 = ass.load("sfx/collect minion_1.ogg");
    let deactivate_1 = ass.load("sfx/deactivate_1.ogg");
    let destruction_1 = ass.load("sfx/destruction_1.ogg");
    let minion_kill_1 = ass.load("sfx/minion kill_1.ogg");
    let player_kill_1 = ass.load("sfx/player kill_1.ogg");
    let respawn_1 = ass.load("sfx/respawn_1.ogg");
    let send_minion_1 = ass.load("sfx/send minion_1.ogg");

    cmd.insert_resource(AudioAssets {
        silence,
        theme,
        activate_1,
        button_click_1,
        collect_minion_1,
        deactivate_1,
        destruction_1,
        minion_kill_1,
        player_kill_1,
        respawn_1,
        send_minion_1,
    });
}

#[rustfmt::skip]
pub fn are_audio_assets_ready(
    ass: &AssetServer,
    assets: &AudioAssets,
) -> bool {
    ass.load_state(&assets.theme)            != LoadState::Loading &&
    ass.load_state(&assets.activate_1)       != LoadState::Loading &&
    ass.load_state(&assets.button_click_1)   != LoadState::Loading &&
    ass.load_state(&assets.collect_minion_1) != LoadState::Loading &&
    ass.load_state(&assets.deactivate_1)     != LoadState::Loading &&
    ass.load_state(&assets.destruction_1)    != LoadState::Loading &&
    ass.load_state(&assets.minion_kill_1)    != LoadState::Loading &&
    ass.load_state(&assets.player_kill_1)    != LoadState::Loading &&
    ass.load_state(&assets.respawn_1)        != LoadState::Loading &&
    ass.load_state(&assets.send_minion_1)    != LoadState::Loading &&
    ass.load_state(&assets.silence)          != LoadState::Loading
}

pub fn fade_in_channels(mut channels: ResMut<AudioChannels>) {
    channels.fade_to(
        AudioChannel::BGM,
        Volume::Amplitude(0.4),
        Duration::from_secs_f32(25.0),
        Easing::InPowf(3.0),
    );
    channels.fade_to(
        AudioChannel::AMB,
        Volume::Amplitude(0.4),
        Duration::from_secs_f32(25.0),
        Easing::InPowf(3.0),
    );
    channels.fade_to(
        AudioChannel::SFX,
        Volume::Amplitude(0.35),
        Duration::from_secs_f32(25.0),
        Easing::InPowf(3.0),
    );
}

pub fn start_bgm_delayed(
    bgm: Res<AudioAssets>,
    mut audio: ResMut<Audio>,
    time: Res<Time<Real>>,
    mut timer: Local<Timer>,
) {
    if timer.duration() == Duration::ZERO {
        *timer = Timer::new(Duration::from_secs_f32(20.0), TimerMode::Once)
    }
    timer.tick(time.delta());
    if timer.just_finished() {
        audio.play_loop(
            bgm.theme.clone(),
            AudioChannel::BGM,
            LoopConfig {
                looping: true,
                ..Default::default()
            },
        );
    }
}

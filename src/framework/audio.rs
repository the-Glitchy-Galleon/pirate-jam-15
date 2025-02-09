//! contains a lot of duplicate implementations from `kira` and `bevy_kira_audio`
//! because people think pub(crate) is a reasonable thing to do. might just have forked it...
use crate::framework::easing::Easing;
use bevy_kira_audio::prelude::*;
use std::time::Duration;
use {bevy::prelude::*, bevy::time::Real};

// Use `AudioAsset` to disambiguate from the `AudioSource` exported by bevy::prelude
pub use bevy_kira_audio::prelude::AudioControl;
pub use bevy_kira_audio::prelude::AudioSource as AudioAsset;
pub use bevy_kira_audio::prelude::{AudioReceiver, PlaybackState, Volume};

#[derive(Resource)]
pub struct GlobalVolume {
    volume: Volume,
    tween: Option<VolumeTween>,
}

impl GlobalVolume {
    pub fn new(volume: f64) -> Self {
        Self {
            volume: Volume::Amplitude(volume),
            tween: None,
        }
    }
}

pub struct AudioPlugin;
impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy_kira_audio::AudioPlugin)
            .init_asset::<bevy_kira_audio::AudioSource>()
            .init_resource::<Audio>()
            .init_resource::<AudioInstances>()
            .insert_resource(AudioChannels::zero_volume())
            .insert_resource(GlobalVolume::new(0.5))
            .add_systems(Startup, stop_audio)
            .add_systems(
                PreUpdate,
                (
                    update_global_tween.before(update_audio_instances),
                    update_channel_tweens.before(update_audio_instances),
                    update_audio_instances,
                ),
            )
            .add_systems(PostUpdate, exec_audio_commands);
    }
}

pub fn stop_audio(track: Res<bevy_kira_audio::AudioChannel<MainTrack>>) {
    track.stop();
}

// masterplan: stop audio and never resume it = still plays back garbage
// pub fn start_audio(track: Res<bevy_kira_audio::AudioChannel<MainTrack>>) {
//     track.resume();
// }

// Todo: could group some components
#[derive(Bundle, Default)]
pub struct AudioEmitterBundle {
    pub emitter: AudioEmitter,
    pub control: AudioInstanceControl,
    pub state: AudioInstanceState,
    // pub channel: AudioInstanceChannel,
    pub volume: AudioInstanceVolume,
}

#[derive(Component)]
pub struct AudioInstanceState(PlaybackState);

#[derive(Default)]
pub struct LoopConfig {
    pub looping: bool,
    pub start: Option<f64>,
    pub end: Option<f64>,
}

#[derive(Default)]
pub struct PlayCommand {
    asset: Handle<AudioAsset>,
    loop_config: LoopConfig,
    panning: Option<f64>, // 0.0 = left, 1.0 = right
    volume: Volume,
    channel: Option<AudioChannel>,
    emitter: Option<Entity>,
}

pub enum AudioCommand {
    Play(PlayCommand),
}

// main audio controls, use this to spawn sounds
#[derive(Resource, Default)]
pub struct Audio {
    commands: Vec<AudioCommand>,
}

impl Audio {
    pub fn play(&mut self, asset: Handle<AudioAsset>, channel: AudioChannel) {
        self.commands.push(AudioCommand::Play(PlayCommand {
            asset,
            channel: Some(channel),
            ..Default::default()
        }));
    }
    pub fn play_vol(&mut self, asset: Handle<AudioAsset>, channel: AudioChannel, volume: Volume) {
        self.commands.push(AudioCommand::Play(PlayCommand {
            asset,
            channel: Some(channel),
            volume,
            ..Default::default()
        }));
    }
    pub fn play_loop(
        &mut self,
        asset: Handle<AudioAsset>,
        channel: AudioChannel,
        loop_config: LoopConfig,
    ) {
        self.commands.push(AudioCommand::Play(PlayCommand {
            asset,
            channel: Some(channel),
            loop_config,
            ..Default::default()
        }));
    }
    pub fn play_pan(&mut self, asset: Handle<AudioAsset>, channel: AudioChannel, pan: f64) {
        self.commands.push(AudioCommand::Play(PlayCommand {
            asset,
            channel: Some(channel),
            panning: Some(pan),
            ..Default::default()
        }));
    }
    pub fn play_loop_pan(
        &mut self,
        asset: Handle<AudioAsset>,
        channel: AudioChannel,
        loop_config: LoopConfig,
        pan: f64,
    ) {
        self.commands.push(AudioCommand::Play(PlayCommand {
            asset,
            channel: Some(channel),
            loop_config,
            panning: Some(pan),
            ..Default::default()
        }));
    }

    pub fn play_spatial(
        &mut self,
        asset: Handle<AudioAsset>,
        channel: AudioChannel,
        entity: Entity,
    ) {
        self.commands.push(AudioCommand::Play(PlayCommand {
            asset,
            channel: Some(channel),
            emitter: Some(entity),
            // volume: Volume::Amplitude(0.0),
            ..Default::default()
        }));
    }
    pub fn play_spatial_vol(
        &mut self,
        asset: Handle<AudioAsset>,
        channel: AudioChannel,
        entity: Entity,
        volume: Volume,
    ) {
        self.commands.push(AudioCommand::Play(PlayCommand {
            asset,
            channel: Some(channel),
            emitter: Some(entity),
            volume,
            ..Default::default()
        }));
    }
    pub fn play_spatial_loop(
        &mut self,
        channel: AudioChannel,
        asset: Handle<AudioAsset>,
        emitter: Entity,
        loop_config: LoopConfig,
    ) {
        self.commands.push(AudioCommand::Play(PlayCommand {
            asset,
            channel: Some(channel),
            loop_config,
            emitter: Some(emitter),
            // volume: Volume::Amplitude(0.0),
            ..Default::default()
        }));
    }
}

fn exec_audio_commands(
    mut audio: ResMut<Audio>,
    bevy_audio: Res<bevy_kira_audio::prelude::Audio>,
    mut instances: ResMut<AudioInstances>,
    channels: ResMut<AudioChannels>,
    global_vol: Res<GlobalVolume>,
    mut sfx_dampening_fix: Local<f64>,
    time: Res<Time<Real>>,
) {
    let global_vol = global_vol.volume.as_amplitude();

    // prevents too many sfx from playing too loudly
    *sfx_dampening_fix = f64::min(*sfx_dampening_fix + time.delta_seconds() as f64 * 0.4, 1.0);

    for cmd in audio.commands.drain(..) {
        match cmd {
            AudioCommand::Play(PlayCommand {
                asset,
                loop_config,
                panning,
                emitter,
                channel,
                volume,
            }) => {
                let volume = match channel {
                    Some(AudioChannel::SFX) => {
                        *sfx_dampening_fix = f64::max(*sfx_dampening_fix - 0.15, 0.0);
                        Volume::Amplitude(volume.as_amplitude() * *sfx_dampening_fix)
                    }
                    _ => volume,
                };
                info!("{}", volume.as_amplitude());

                let chan_vol = channel
                    .map(|i| channels.get(i).as_amplitude())
                    .unwrap_or(1.0);

                let mut cmd = bevy_audio.play(asset);
                cmd.with_volume(Volume::Amplitude(
                    volume.as_amplitude() * chan_vol * global_vol,
                ));
                if loop_config.looping {
                    cmd.looped();
                    if let Some(start) = loop_config.start {
                        cmd.loop_from(start);
                    }
                    if let Some(end) = loop_config.end {
                        cmd.loop_until(end);
                    }
                }
                if let Some(pan) = panning {
                    cmd.with_panning(pan);
                }
                instances.0.push(AudioInstance {
                    volume: AudioInstanceVolume(volume),
                    handle: cmd.handle().clone(),
                    channel,
                    emitter,
                });
                drop(cmd);
            }
        }
    }
}

impl Default for AudioInstanceState {
    fn default() -> Self {
        Self(PlaybackState::Stopped)
    }
}

impl AudioInstanceState {
    pub fn state(&self) -> PlaybackState {
        self.0
    }
}

#[derive(Component, Default)]
pub struct AudioInstanceVolume(Volume);

#[derive(Debug, Clone, Copy)]
#[repr(usize)]
pub enum AudioChannel {
    BGM = 0,
    AMB = 1,
    SFX = 2,
    VOX = 3,
}

impl AudioChannel {
    pub const COUNT: usize = 4;
}

#[derive(Debug)]
enum Control {
    Pause,
    Resume,
}

#[derive(Component, Default, Debug)]
pub struct AudioInstanceControl(Option<Control>);

impl AudioInstanceControl {
    pub fn pause(&mut self) {
        self.0 = Some(Control::Pause);
    }
    pub fn resume(&mut self) {
        self.0 = Some(Control::Resume);
    }
}

#[derive(Component)]
pub struct AudioSpatialRange(pub f64);

#[derive(Resource, Default)]
pub struct AudioChannels {
    volumes: [Volume; AudioChannel::COUNT],
    tweens: [Option<VolumeTween>; AudioChannel::COUNT],
}

impl AudioChannels {
    pub const fn zero_volume() -> Self {
        const TWEEN: Option<VolumeTween> = None;
        Self {
            volumes: [Volume::Amplitude(0.0); AudioChannel::COUNT],
            tweens: [TWEEN; AudioChannel::COUNT],
        }
    }
}

#[derive(Debug, Clone)]
pub struct VolumeTween {
    source: f64,
    target: f64,
    timer: Timer,
    duration: Duration,
    easing: Easing,
}

impl AudioChannels {
    pub fn get(&self, channel: AudioChannel) -> Volume {
        self.volumes[channel as usize]
    }

    // Todo: would be nice if the user didn't have to feed the time in,
    // but i can't find any way to ask bevy for the current time.
    // `std::time::SystemTime` and `std::time::Instant` are not supported on wasm, it seems.
    pub fn fade_to(
        &mut self,
        channel: AudioChannel,
        target: Volume,
        duration: Duration,
        easing: Easing,
    ) {
        self.tweens[channel as usize] = Some(VolumeTween {
            source: self.volumes[channel as usize].as_amplitude(),
            target: target.as_amplitude(),
            timer: Timer::new(duration, TimerMode::Once),
            duration,
            easing,
        });
    }
}

fn update_global_tween(mut global: ResMut<GlobalVolume>, time: Res<Time<Real>>) {
    let Some(tween) = &mut global.tween else {
        return;
    };
    let res = {
        tween.timer.tick(time.delta());
        let t = tween.timer.elapsed();
        if t >= tween.duration {
            (Volume::Amplitude(tween.target), true)
        } else {
            let t = t.as_secs_f64() / tween.duration.as_secs_f64();
            let t = tween.easing.apply(t);
            assert!(t >= 0.0 && t <= 1.0);
            (
                Volume::Amplitude(
                    tween.source + tween.easing.apply(t) * (tween.target - tween.source),
                ),
                false,
            )
        }
    };
    let (vol, clear) = res;
    global.volume = vol;
    if clear {
        global.tween.take();
    }
}

fn update_channel_tweens(mut channel_volumes: ResMut<AudioChannels>, time: Res<Time<Real>>) {
    for i in 0..AudioChannel::COUNT {
        let res = channel_volumes.tweens[i].as_mut().map(|tween| {
            tween.timer.tick(time.delta());
            let t = tween.timer.elapsed();
            if t >= tween.duration {
                (Volume::Amplitude(tween.target), true)
            } else {
                let t = t.as_secs_f64() / tween.duration.as_secs_f64();
                let t = tween.easing.apply(t);
                assert!(t >= 0.0 && t <= 1.0);
                (
                    Volume::Amplitude(
                        tween.source + tween.easing.apply(t) * (tween.target - tween.source),
                    ),
                    false,
                )
            }
        });

        if let Some((vol, clear)) = res {
            channel_volumes.volumes[i] = vol;
            if clear {
                channel_volumes.tweens[i].take();
            }
        }
    }
}

pub struct AudioInstance {
    handle: Handle<bevy_kira_audio::AudioInstance>,
    channel: Option<AudioChannel>,
    emitter: Option<Entity>,
    volume: AudioInstanceVolume,
}

#[derive(Resource, Default)]
pub struct AudioInstances(Vec<AudioInstance>);

/// Overwrite instance audio volumes.
/// Because the current implementation in bevy_kira doesn't consider channel and global values in its Spatial implementation
fn update_audio_instances(
    mut emitters: Query<(
        Option<&GlobalTransform>,
        Option<&mut AudioInstanceState>,
        Option<&mut AudioInstanceControl>,
        Option<&AudioSpatialRange>,
    )>,
    instances: ResMut<AudioInstances>,
    mut bevy_instances: ResMut<Assets<bevy_kira_audio::AudioInstance>>,
    spatial_receiver: Query<&GlobalTransform, With<AudioReceiver>>,
    global_vol: Res<GlobalVolume>,
    channels: Res<AudioChannels>,
) {
    let receiver_transform = spatial_receiver.get_single().ok();
    let global_vol = global_vol.volume.as_amplitude();

    for instance in instances.0.iter() {
        // Todo: Check if there needs to be some cleanup for stopped audio instances
        let Some(bevy_instance) = bevy_instances.get_mut(&instance.handle) else {
            // warn!("Failed to get audio instance");
            continue;
        };

        let chan_vol = instance
            .channel
            .map(|i| channels.get(i).as_amplitude())
            .unwrap_or(1.0);

        if let Some(emitter) = instance.emitter {
            if let Ok((transform, mut state, mut ctrl, range)) = emitters.get_mut(emitter) {
                if let Some(ctrl) = &mut ctrl {
                    match ctrl.0.take() {
                        Some(Control::Pause) => {
                            bevy_instance.pause(AudioTween::default());
                        }
                        Some(Control::Resume) => {
                            bevy_instance.resume(AudioTween::default());
                        }
                        None => {}
                    }
                }

                let mut spatial_vol = 1.0;
                let mut spatial_pan = 0.5;

                if let Some(range) = range {
                    if let Some(transform) = transform {
                        if let Some(receiver_transform) = receiver_transform {
                            let sound_path =
                                transform.translation() - receiver_transform.translation();
                            let volume =
                                f64::clamp(1. - sound_path.length() as f64 / range.0, 0.0, 1.0)
                                    .powi(2);

                            let right_ear_angle =
                                receiver_transform.right().angle_between(sound_path);
                            let panning = (right_ear_angle.cos() + 1.) / 2.;

                            spatial_vol = volume as f64;
                            spatial_pan = panning as f64;
                        }
                    }
                }
                if let Some(state) = &mut state {
                    state.0 = bevy_instance.state();
                }
                // let vol = vol.map(|v| v.0.as_amplitude()).unwrap_or(1.0);
                let vol = instance.volume.0.as_amplitude();

                bevy_instance.set_volume(
                    chan_vol * global_vol * vol * spatial_vol,
                    AudioTween::default(),
                );
                bevy_instance.set_panning(spatial_pan, AudioTween::default());
            }
        } else {
            let volume = instance.volume.0.as_amplitude() * global_vol * chan_vol;
            // info!("VOL: {}", volume);
            bevy_instance.set_volume(volume, AudioTween::default());
        }
    }
}

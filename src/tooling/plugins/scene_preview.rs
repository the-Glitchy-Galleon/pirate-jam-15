use crate::framework::prelude::*;
use bevy::{asset::LoadState, prelude::*};
use bevy_rapier3d::geometry::Collider;
use bevy_rapier3d::pipeline::QueryFilter;
use bevy_rapier3d::plugin::{NoUserData, RapierContext, RapierPhysicsPlugin};
use itertools::izip;
use rand::Rng;
use std::{f32::consts::PI, ffi::OsStr, path::Path, time::Duration};
use ui::{CurrentAnimationTextTag, UiHeaderText, UiRoot, TRANSPARENT};

pub const BOX_EMISSIVE_ACTIVE: LinearRgba = LinearRgba::rgb(1.5, 1.5, 1.5);

pub struct ScenePreviewPlugin;

impl Plugin for ScenePreviewPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
            .init_resource::<LoadQueue>()
            .insert_resource(AmbientLight {
                color: Color::WHITE,
                brightness: 2000.,
            })
            .init_resource::<AudioFiles>()
            .add_event::<DespawnAllScenesEvent>()
            .add_event::<SpawnGltfEvent>()
            .add_event::<InitAnimationPlayerEvent>()
            .add_systems(Startup, ui::setup)
            .add_systems(Startup, (setup_audio_receiver, setup_audio_emitters))
            .add_systems(
                Update,
                (
                    file_drag_and_drop,
                    load_queue,
                    spawn_gltf,
                    initial_channel_fade_in,
                    setup_animation_master.after(spawn_gltf),
                    start_animation_on_load.after(setup_animation_master),
                    despawn_all_gltfs,
                    change_animation,
                    toggle_sound_emitter_on_click,
                ),
            )
            .add_systems(Update, draw_gizmos);
    }
}

#[derive(Resource)]
struct AudioFiles {
    fire_with_crackles: Handle<AudioAsset>,
    jungle_amb_1: Handle<AudioAsset>,
    footsteps: [Handle<AudioAsset>; 5],
    loopable_ambience_1: Handle<AudioAsset>,
}

impl FromWorld for AudioFiles {
    fn from_world(world: &mut World) -> Self {
        Self {
            fire_with_crackles: world
                .load_asset("tooling/scene_preview/test_sounds/fire_with crackles_1.ogg"),
            jungle_amb_1: world.load_asset("tooling/scene_preview/test_sounds/jungle_amb_1.ogg"),
            footsteps: [
                world.load_asset("tooling/scene_preview/test_sounds/footsteps_1.ogg"),
                world.load_asset("tooling/scene_preview/test_sounds/footsteps_2.ogg"),
                world.load_asset("tooling/scene_preview/test_sounds/footsteps_3.ogg"),
                world.load_asset("tooling/scene_preview/test_sounds/footsteps_4.ogg"),
                world.load_asset("tooling/scene_preview/test_sounds/footsteps_5.ogg"),
            ],
            loopable_ambience_1: world
                .load_asset("tooling/scene_preview/test_sounds/loopable_ambience_1.ogg"),
        }
    }
}

fn setup_audio_emitters(
    mut cmd: Commands,
    mut audio: ResMut<Audio>,
    assets: Res<AudioFiles>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
) {
    let ent = cmd.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid {
                half_size: Vec3::new(0.25, 0.5, 0.25),
            }),
            material: mats.add(StandardMaterial {
                emissive: BOX_EMISSIVE_ACTIVE,
                ..StandardMaterial::from_color(LinearRgba::RED)
            }),
            transform: Transform::from_xyz(-3.0, 0.5, 1.0),
            ..default()
        },
        Collider::cuboid(0.25, 0.5, 0.25),
        AudioEmitterBundle::default(),
        AudioSpatialRange(15.0),
    ));
    audio.play_spatial(
        assets.fire_with_crackles.clone(),
        AudioChannel::AMB,
        ent.id(),
    );
    let ent = cmd.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid {
                half_size: Vec3::new(0.25, 0.5, 0.25),
            }),
            material: mats.add(StandardMaterial {
                emissive: BOX_EMISSIVE_ACTIVE,
                ..StandardMaterial::from_color(LinearRgba::GREEN)
            }),
            transform: Transform::from_xyz(3.0, 0.5, 1.0),
            ..default()
        },
        Collider::cuboid(0.25, 0.5, 0.25),
        AudioEmitterBundle::default(),
        AudioSpatialRange(15.0),
    ));
    audio.play_spatial(assets.jungle_amb_1.clone(), AudioChannel::AMB, ent.id());
    let ent = cmd.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid {
                half_size: Vec3::new(0.25, 0.5, 0.25),
            }),
            material: mats.add(StandardMaterial {
                emissive: BOX_EMISSIVE_ACTIVE,
                ..StandardMaterial::from_color(LinearRgba::WHITE)
            }),
            transform: Transform::from_xyz(0.0, 0.5, -2.0),
            ..default()
        },
        AudioEmitterBundle::default(),
        Collider::cuboid(0.25, 0.5, 0.25),
        AudioSpatialRange(15.0),
    ));
    audio.play_spatial(
        assets.loopable_ambience_1.clone(),
        AudioChannel::BGM,
        ent.id(),
    );
}

fn initial_channel_fade_in(
    input: Res<ButtonInput<MouseButton>>,
    mut channels: ResMut<AudioChannels>,
    time: Res<Time<Real>>,
    mut initialized: Local<bool>,
) {
    if !*initialized && input.just_pressed(MouseButton::Left) {
        channels.fade_to(
            AudioChannel::BGM,
            Volume::Amplitude(0.4),
            Duration::from_secs_f32(2.0),
            Easing::InPowf(3.0),
            time.clone(),
        );
        channels.fade_to(
            AudioChannel::AMB,
            Volume::Amplitude(0.4),
            Duration::from_secs_f32(2.0),
            Easing::InPowf(3.0),
            time.clone(),
        );
        channels.fade_to(
            AudioChannel::SFX,
            Volume::Amplitude(0.6),
            Duration::from_secs_f32(2.0),
            Easing::InPowf(3.0),
            time.clone(),
        );
        *initialized = true;
    }
}

fn setup_audio_receiver(mut cmd: Commands, cam: Query<Entity, With<Camera>>) {
    for cam in cam.iter() {
        cmd.entity(cam).insert(AudioReceiver);
    }
}

fn toggle_sound_emitter_on_click(
    camera_query: Query<(&Camera, &GlobalTransform)>,
    rapier_context: Res<RapierContext>,
    mut emitters: Query<(
        &AudioInstanceState,
        &mut AudioInstanceControl,
        &Handle<StandardMaterial>,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mouse: Res<ButtonInput<MouseButton>>,
    sfx: Res<AudioFiles>,
    mut audio: ResMut<Audio>,
    cursor_position: Res<LogicalCursorPosition>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    let Some(cursor_position) = cursor_position.0 else {
        return;
    };
    let (camera, camera_global_transform) = camera_query.single();
    let Some(ray) = camera.viewport_to_world(camera_global_transform, cursor_position) else {
        return;
    };

    if let Some((entity, toi)) = rapier_context.cast_ray(
        ray.origin,
        *ray.direction,
        50.0,
        true,
        QueryFilter::default(),
    ) {
        let _hit_point = ray.origin + *ray.direction * toi;

        if let Ok((state, mut control, mat)) = emitters.get_mut(entity) {
            let mut rng = rand::thread_rng();
            audio.play(
                sfx.footsteps[rng.gen_range(0..sfx.footsteps.len())].clone(),
                AudioChannel::SFX,
            );

            match state.state() {
                PlaybackState::Paused { .. } => {
                    control.resume();
                    if let Some(mat) = materials.get_mut(mat) {
                        mat.emissive = BOX_EMISSIVE_ACTIVE;
                    }
                }
                PlaybackState::Playing { .. } => {
                    control.pause();

                    if let Some(mat) = materials.get_mut(mat) {
                        mat.emissive = LinearRgba::NONE;
                    }
                }
                _ => {}
            }
        }
    };
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum SupportedAsset {
    Gltf(Handle<Gltf>),
}

#[cfg(target_family = "wasm")]
fn make_asset_path(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

#[cfg(not(target_family = "wasm"))]
fn make_asset_path(path: &Path) -> bevy::asset::AssetPath {
    bevy::asset::AssetPath::from_path(path)
}

impl SupportedAsset {
    pub fn load<P: AsRef<Path>>(ass: &Res<AssetServer>, path: P) -> Option<SupportedAsset> {
        match SupportedAssetKind::could_load(&path) {
            Some(SupportedAssetKind::Gltf) => {
                let path = make_asset_path(path.as_ref());
                return Some(SupportedAsset::Gltf(ass.load(path)));
            }
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum SupportedAssetKind {
    Gltf,
}
impl SupportedAssetKind {
    pub fn could_load<P: AsRef<Path>>(path: P) -> Option<Self> {
        let Some(ext) = path.as_ref().extension() else {
            return None;
        };
        Self::from_osstr_ext(ext)
    }
    pub fn from_osstr_ext(ext: &OsStr) -> Option<Self> {
        let s = ext.to_str()?.to_ascii_lowercase();
        match s.as_str() {
            "gltf" | "glb" => Some(SupportedAssetKind::Gltf),
            ext => {
                warn!("could_load: unsupported asset kind: {ext}");
                None
            }
        }
    }
}

#[derive(Resource, Default)]
pub struct LoadQueue {
    queue: Vec<SupportedAsset>,
}

#[allow(unused_variables)]
fn file_drag_and_drop(
    ass: Res<AssetServer>,
    mut evs: EventReader<FileDragAndDrop>,
    mut queue: ResMut<LoadQueue>,
    mut root: Query<&mut BackgroundColor, With<UiRoot>>,
    mut text: Query<&mut Text, With<UiHeaderText>>,
) {
    let Ok(mut root_bg) = root.get_single_mut() else {
        error!("no root UI");
        return;
    };
    let Ok(mut text) = text.get_single_mut() else {
        error!("no header text in UI");
        return;
    };

    for event in evs.read() {
        match event {
            FileDragAndDrop::DroppedFile { window, path_buf } => {
                info!(
                    "FileDragAndDrop::DroppedFile => window: {:?}, path_buf: {:?}",
                    window, path_buf
                );
                if let Some(asset) = SupportedAsset::load(&ass, path_buf) {
                    queue.queue.push(asset);
                } else {
                    warn!("Asset not supported: {path_buf:?}");
                }
                *root_bg = TRANSPARENT.into();
            }
            FileDragAndDrop::HoveredFile { window, path_buf } => {
                text.sections[0].value =
                    format!("Load {}", path_buf.file_name().unwrap().to_string_lossy());
                *root_bg = match SupportedAssetKind::could_load(&path_buf) {
                    Some(_) => BackgroundColor(bevy::color::palettes::tailwind::BLUE_800.into()),
                    _ => BackgroundColor(bevy::color::palettes::tailwind::RED_800.into()),
                };
            }
            FileDragAndDrop::HoveredFileCanceled { window } => {
                // *root_vis = Visibility::Hidden;
                *root_bg = bevy::color::palettes::tailwind::RED_800
                    .with_alpha(0.2)
                    .into();
            }
        }
    }
}

fn load_queue(
    ass: Res<AssetServer>,
    mut queue: ResMut<LoadQueue>,
    mut despawn_evs: EventWriter<DespawnAllScenesEvent>,
    mut spawn_evs: EventWriter<SpawnGltfEvent>,
) {
    let mut dequeue = vec![];

    for asset in &queue.queue {
        match asset {
            SupportedAsset::Gltf(h) => match ass.get_load_state(h.id()) {
                Some(LoadState::Loaded) => {
                    if !ass.is_loaded_with_dependencies(h) {
                        warn!("Not all dependencies loaded yet");
                        continue;
                    }
                    dequeue.push(asset.clone());
                    spawn_evs.send(SpawnGltfEvent(h.clone()));
                    despawn_evs.send(DespawnAllScenesEvent);
                }
                Some(LoadState::Failed(e)) => error!("Failed to load {h:?}: {e:?}"),
                Some(LoadState::NotLoaded) => {}
                Some(LoadState::Loading) => {}
                None => error!("No load state for queued asset {h:?}"),
            },
        }
    }
    queue.queue.retain(|a| !dequeue.contains(&a));
}

#[derive(Event)]
struct SpawnGltfEvent(Handle<Gltf>);

fn spawn_gltf(
    mut cmd: Commands,
    mut evs: EventReader<SpawnGltfEvent>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    mut init_evs: EventWriter<InitAnimationPlayerEvent>,
    gltfs: Res<Assets<Gltf>>,
) {
    for ev in evs.read() {
        let Some(gltf) = gltfs.get(ev.0.id()) else {
            eprintln!("Failed to find GLTF data for {:?}", ev.0);
            return;
        };
        let mut graph = AnimationGraph::new();

        let animations = izip!(
            gltf.named_animations.keys().cloned(),
            graph.add_clips(
                gltf.named_animations.values().map(|x| x.clone()),
                1.0,
                graph.root
            )
        )
        .collect::<Vec<_>>();

        let graph = graphs.add(graph);

        init_evs.send(InitAnimationPlayerEvent {
            list: animations,
            graph,
        });

        // Spawning the gltf scene in will also spawn in the
        // `AnimationPlayer` and other goodies when available
        cmd.spawn((SceneBundle {
            scene: gltf.scenes[0].clone(),
            transform: Transform::IDENTITY,
            ..Default::default()
        },));
    }
}

#[derive(Event)]
struct InitAnimationPlayerEvent {
    list: Vec<(Box<str>, AnimationNodeIndex)>,
    graph: Handle<AnimationGraph>,
}

#[derive(Component)]
struct AnimationMaster {
    list: Vec<(Box<str>, AnimationNodeIndex)>,
    player_entity: Entity,
    main_anim_idx: usize,
}

impl AnimationMaster {
    pub fn from_event(ev: &InitAnimationPlayerEvent, player_entity: Entity) -> Self {
        Self {
            list: ev.list.clone(),
            player_entity,
            main_anim_idx: 0,
        }
    }
}

fn setup_animation_master(
    mut cmd: Commands,
    mut qry: Query<Entity, Added<AnimationPlayer>>,
    mut evs: EventReader<InitAnimationPlayerEvent>,
) {
    for ent in qry.iter_mut() {
        let transitions = AnimationTransitions::new();

        // wonky but whatever, i can't find a way to sync it up
        // since the entity of the AnimationPlayer is somewhere buried in the scene
        let Some(ev) = evs.read().next() else {
            return;
        };

        cmd.entity(ent)
            .insert(AnimationMaster::from_event(ev, ent))
            .insert(transitions)
            .insert(ev.graph.clone());
    }
}

fn start_animation_on_load(
    mut text: Query<&mut Text, With<CurrentAnimationTextTag>>,
    mut master: Query<&mut AnimationMaster, Added<AnimationMaster>>,
    mut player: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
) {
    let Ok(mut text) = text.get_single_mut() else {
        return;
    };
    for master in master.iter_mut() {
        let Ok((mut player, mut transitions)) = player.get_mut(master.player_entity) else {
            error!("Found no AnimationPlayer for AnimationMaster");
            continue;
        };

        if let Some((name, idx)) = master.list.get(master.main_anim_idx) {
            info!("Starting Animation: {name}");
            transitions.play(&mut player, *idx, Duration::ZERO).repeat();
            player.start(*idx);
        }
        let anim = master.list.get(master.main_anim_idx).unwrap();
        text.sections[0].value = format!("[1] < {} > [2]", anim.0);
    }
}

fn change_animation(
    mut text: Query<&mut Text, With<CurrentAnimationTextTag>>,
    mut master: Query<&mut AnimationMaster>,
    mut player: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
    input: Res<ButtonInput<KeyCode>>,
) {
    let Ok(mut text) = text.get_single_mut() else {
        return;
    };
    for mut master in master.iter_mut() {
        let Ok((mut player, mut transitions)) = player.get_mut(master.player_entity) else {
            continue;
        };

        let old_idx = master.main_anim_idx;

        if input.just_pressed(KeyCode::Digit1) {
            master.main_anim_idx = (old_idx + 1) % master.list.len();
        }

        if input.just_pressed(KeyCode::Digit2) {
            master.main_anim_idx = (if old_idx == 0 {
                master.list.len()
            } else {
                old_idx
            }) - 1;
        }

        if old_idx != master.main_anim_idx {
            let anim = master.list.get(master.main_anim_idx).unwrap();
            transitions.play(&mut player, anim.1, Duration::from_secs_f32(0.1));
            text.sections[0].value = format!("[1] < {} > [2]", anim.0);
        }
    }
}

#[derive(Event)]
struct DespawnAllScenesEvent;

fn despawn_all_gltfs(
    mut cmd: Commands,
    mut evs: EventReader<DespawnAllScenesEvent>,
    scenes: Query<Entity, With<Handle<Scene>>>,
) {
    for _ in evs.read() {
        for ent in scenes.iter() {
            cmd.entity(ent).despawn();
        }
    }
}

mod ui {
    use super::super::pointer_capture_check::NoPointerCapture;
    use bevy::{
        prelude::*,
        text::{JustifyText, TextStyle},
        ui::{node_bundles::TextBundle, PositionType, Val},
    };

    pub const TRANSPARENT: Color = Color::LinearRgba(LinearRgba::NONE);

    #[derive(Component)]
    pub struct CurrentAnimationTextTag;

    #[derive(Component)]
    pub struct UiRoot;

    #[derive(Component)]
    pub struct UiHeaderText;

    pub fn setup(mut cmd: Commands) {
        cmd.spawn((
            UiRoot,
            NoPointerCapture,
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::SpaceBetween,
                    display: Display::Block,
                    padding: UiRect::all(Val::Px(24.0)),
                    ..default()
                },
                visibility: Visibility::Visible,
                ..default()
            },
        ))
        .with_children(|p| {
            p.spawn((
                CurrentAnimationTextTag,
                NoPointerCapture,
                UiHeaderText,
                TextBundle::from_section(
                    "Hello, World!",
                    TextStyle::default(), // doesn't seem to change anything?
                )
                .with_text_justify(JustifyText::Center)
                .with_background_color(TRANSPARENT)
                .with_style(Style {
                    position_type: PositionType::Absolute,
                    left: Val::Vh(0.5),
                    top: Val::Vh(0.2),
                    right: Val::Vh(0.5),
                    bottom: Val::Vh(0.5),
                    ..default()
                }),
            ));
        });
    }
}

fn draw_gizmos(mut gizmos: Gizmos) {
    gizmos.grid(
        Vec3::ZERO,
        Quat::from_axis_angle(Vec3::X, PI * 0.5),
        UVec2::splat(20),
        Vec2::splat(1.),
        LinearRgba::gray(0.65),
    );
}

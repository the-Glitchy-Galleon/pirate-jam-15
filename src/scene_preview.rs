use bevy::{asset::LoadState, prelude::*};
use std::{f32::consts::PI, ffi::OsStr, path::Path};
use ui::{UiHeaderText, UiRoot};

pub struct ScenePreviewPlugin;

impl Plugin for ScenePreviewPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LoadQueue>()
            .insert_resource(AmbientLight {
                color: Color::WHITE,
                brightness: 2000.,
            })
            .add_event::<DespawnAllScenesEvent>()
            .add_event::<SpawnGltfEvent>()
            .add_systems(Startup, ui::setup)
            .add_systems(
                Update,
                (
                    file_drag_and_drop,
                    load_queue,
                    spawn_gltf,
                    despawn_all_gltfs,
                    draw_gizmos,
                ),
            );
    }
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
    mut root: Query<(&mut Visibility, &mut BackgroundColor), With<UiRoot>>,
    mut text: Query<&mut Text, With<UiHeaderText>>,
) {
    let Ok((mut root_vis, mut root_bg)) = root.get_single_mut() else {
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

                *root_vis = Visibility::Hidden;
            }
            FileDragAndDrop::HoveredFile { window, path_buf } => {
                let t = format!("Load {}", path_buf.file_name().unwrap().to_string_lossy());
                *text = Text::from_section(t, TextStyle::default());
                *root_vis = Visibility::Visible;
                *root_bg = match SupportedAssetKind::could_load(&path_buf) {
                    Some(_) => BackgroundColor(bevy::color::palettes::tailwind::BLUE_800.into()),
                    _ => BackgroundColor(bevy::color::palettes::tailwind::RED_800.into()),
                };
            }
            FileDragAndDrop::HoveredFileCanceled { window } => {
                *root_vis = Visibility::Hidden;
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
    gltfs: Res<Assets<Gltf>>,
) {
    for ev in evs.read() {
        let Some(gltf) = gltfs.get(ev.0.id()) else {
            eprintln!("Failed to find GLTF data for {:?}", ev.0);
            return;
        };
        let mut graph = AnimationGraph::new();
        let _animations = graph
            .add_clips(gltf.animations.iter().map(|x| x.clone()), 1.0, graph.root)
            .collect::<Vec<_>>();
        let _graph = graphs.add(graph);

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
    use bevy::{
        prelude::*,
        text::{JustifyText, TextStyle},
        ui::{node_bundles::TextBundle, PositionType, Val},
    };

    #[derive(Component)]
    pub struct UiRoot;

    #[derive(Component)]
    pub struct UiHeaderText;

    pub fn setup(mut cmd: Commands) {
        cmd.spawn((
            UiRoot,
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::SpaceBetween,
                    display: Display::Block,
                    padding: UiRect::all(Val::Px(24.0)),
                    ..default()
                },
                visibility: Visibility::Hidden,
                ..default()
            },
        ))
        .with_children(|p| {
            p.spawn((
                UiHeaderText,
                TextBundle::from_section(
                    "Hello, World!",
                    TextStyle::default(), // doesn't seem to change anything?
                )
                .with_text_justify(JustifyText::Center)
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

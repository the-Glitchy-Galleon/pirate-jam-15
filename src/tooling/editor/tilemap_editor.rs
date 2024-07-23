use super::tilemap_controls::TilemapControls;
use super::tilemap_mesh_builder::{self, RawMeshBuilder};
use crate::framework::level_asset::{LevelAsset, LevelAssetData, WallData};
use crate::framework::prelude::*;
use crate::framework::tilemap::{Pnormal3, SLOPE_HEIGHT, WALL_HEIGHT};
use crate::framework::tileset::{TILESET_PATH_DIFFUSE, TILESET_PATH_NORMAL, TILESET_TILE_NUM};
use bevy::color::palettes::tailwind::*;
use bevy::{ecs::system::SystemId, prelude::*};
use bevy_rapier3d::geometry::{CollisionGroups, Group};
use bevy_rapier3d::math::Real;
use bevy_rapier3d::pipeline::QueryFilter;
use bevy_rapier3d::plugin::RapierContext;
use std::f32::consts::PI;

const DEFAULT_EDITOR_SAVE_PATH: &str = "./level_editor_scenes";
const DEFAULT_EDITOR_EXPORT_PATH: &str = "./assets/level";

#[derive(Component, Reflect)]
pub struct TilemapGroundMesh;

#[derive(Component, Reflect)]
pub struct TilemapWallMesh;

#[derive(Resource)]
pub struct EditorState {
    tilemap: Tilemap,
    tileset: Tileset,
    hovered_ground_face: Option<u32>,
    hovered_wall_ground: Option<u32>,
    hovered_wall_height: Option<u32>,
    hovered_wall_normal: Option<Pnormal3>,
    selected_tile_coords: Option<UVec2>,
}

#[derive(Resource)]
pub struct EditorControls {
    tilemap: TilemapControls,
}

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ControlMode {
    ShapeTerrain,
    FlattenTerrain,
    ShapeWalls,
    Paint2D,
    PaintTerrain3D,
    PaintWalls3D,
    PlaceGameObjects,
    #[default]
    AdminStuff,
}

pub struct TilemapEditorPlugin;

impl Plugin for TilemapEditorPlugin {
    fn build(&self, app: &mut App) {
        let tilemap = Tilemap::new(UVec2::new(32, 32)).unwrap();
        let tileset = Tileset::new(UVec2::new(TILESET_TILE_NUM[0], TILESET_TILE_NUM[1])).unwrap();

        app.init_resource::<Systems>()
            .init_state::<ControlMode>()
            .init_resource::<ExportLevelScenePath>()
            .insert_resource(EditorState {
                tilemap,
                tileset,
                hovered_ground_face: None,
                hovered_wall_ground: None,
                hovered_wall_height: None,
                hovered_wall_normal: None,
                selected_tile_coords: None,
            })
            .insert_resource(ui::EguiState {
                ..Default::default()
            })
            .insert_resource(EditorControls {
                tilemap: TilemapControls::new(12, 3),
            })
            .add_systems(
                Update,
                (
                    update_hovered_states,
                    perform_click_actions,
                    change_control_mode,
                    ui::render_egui,
                    draw_vert_gizmos,
                    draw_hovered_tile_gizmo,
                    ui::update_info_text,
                    ui::check_open_file_dialog,
                ),
            )
            .add_systems(Startup, (setup, ui::setup));
    }
}

#[derive(Resource)]
struct Systems {
    recreate_ground_mesh: SystemId,
    export_level_scene: SystemId,
}

impl FromWorld for Systems {
    fn from_world(world: &mut World) -> Self {
        Self {
            recreate_ground_mesh: world.register_system(recreate_ground_and_wall_meshes),
            export_level_scene: world.register_system(export_level_scene),
        }
    }
}

fn setup(mut cmd: Commands, sys: Res<Systems>) {
    cmd.run_system(sys.recreate_ground_mesh);
}

fn recreate_ground_and_wall_meshes(
    mut cmd: Commands,
    ass: Res<AssetServer>,
    state: Res<EditorState>,
    mut egui_state: ResMut<ui::EguiState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    grounds: Query<Entity, With<TilemapGroundMesh>>,
    walls: Query<Entity, With<TilemapWallMesh>>,
) {
    // despawn existing
    for ex in grounds.iter() {
        cmd.entity(ex).despawn_recursive();
    }
    for ex in walls.iter() {
        cmd.entity(ex).despawn_recursive();
    }
    let diffuse: Handle<Image> = ass.load(TILESET_PATH_DIFFUSE);
    let normal: Option<Handle<Image>> = TILESET_PATH_NORMAL.map(|f| ass.load(f));

    let builder = RawMeshBuilder::new(&state.tilemap);
    let mesh = builder.make_ground_mesh(&state.tileset).into();
    // let collider = builder.build_rapier_heightfield_collider();
    let collider = tilemap_mesh_builder::build_rapier_convex_collider_for_preview(&mesh);
    let handle: Handle<Mesh> = meshes.add(mesh);

    cmd.spawn((
        PbrBundle {
            mesh: handle,
            material: mats.add(StandardMaterial {
                base_color_texture: Some(diffuse.clone()),
                normal_map_texture: normal.clone(),
                perceptual_roughness: 0.9,
                metallic: 0.0,
                ..default()
            }),
            transform: Transform::IDENTITY,
            ..default()
        },
        collider,
        CollisionGroups::new(Group::GROUP_15, Group::GROUP_15),
        TilemapGroundMesh,
    ));

    for mesh in builder.make_wall_meshes(&state.tileset) {
        let mesh = mesh.into();
        let collider = tilemap_mesh_builder::build_rapier_convex_collider_for_preview(&mesh);
        // let collider = Collider::from_bevy_mesh(&mesh, &ComputedColliderShape::TriMesh).unwrap();
        let handle: Handle<Mesh> = meshes.add(mesh);
        cmd.spawn((
            PbrBundle {
                mesh: handle,
                material: mats.add(StandardMaterial {
                    base_color_texture: Some(diffuse.clone()),
                    normal_map_texture: normal.clone(),
                    perceptual_roughness: 0.9,
                    metallic: 0.0,
                    ..default()
                }),
                transform: Transform::IDENTITY,
                ..default()
            },
            collider,
            CollisionGroups::new(Group::GROUP_16, Group::GROUP_16),
            TilemapWallMesh,
        ));
    }

    if let Some(widget) = &mut egui_state.resize_widget {
        widget.set_dims(state.tilemap.dims());
    }
}

fn draw_vert_gizmos(
    mut gizmos: Gizmos,
    tilemap: Res<EditorState>,
    transform: Query<&Transform, With<TilemapGroundMesh>>,
) {
    let transform = transform.single();

    for vert in tilemap.tilemap.vert_iter() {
        gizmos.cuboid(
            Transform::from_translation(vert + transform.translation).with_scale(Vec3::splat(0.1)),
            BLUE_700,
        );
    }
}

fn update_hovered_states(
    camera: Query<(&Camera, &GlobalTransform)>,
    rapier: Res<RapierContext>,
    cursor: Res<LogicalCursorPosition>,
    ground: Query<&Transform, With<TilemapGroundMesh>>,
    mut state: ResMut<EditorState>,
) {
    state.hovered_ground_face = None;
    state.hovered_wall_ground = None;
    state.hovered_wall_height = None;
    state.hovered_wall_normal = None;

    let offset = ground.single().translation;
    let Some(cursor_position) = cursor.0 else {
        return;
    };

    let (camera, transform) = camera.single();
    let Some(ray) = camera.viewport_to_world(transform, cursor_position) else {
        return;
    };

    let ground_ray = rapier.cast_ray(
        ray.origin,
        *ray.direction,
        Real::MAX,
        true,
        QueryFilter::new().groups(CollisionGroups::new(Group::GROUP_15, Group::GROUP_15)),
    );

    let wall_ray = rapier.cast_ray_and_get_normal(
        ray.origin,
        *ray.direction,
        Real::MAX,
        true,
        QueryFilter::new().groups(CollisionGroups::new(Group::GROUP_16, Group::GROUP_16)),
    );

    let mut ground_toi = f32::MAX;
    if let Some((_ent, toi)) = ground_ray {
        let pos = ray.origin + *ray.direction * toi;
        ground_toi = toi;

        if let Some(i) = state
            .tilemap
            .pos_to_face_id(offset.x + pos.x, offset.z + pos.z)
        {
            state.hovered_ground_face = Some(i);
        }
    }

    if let Some((_ent, res)) = wall_ray {
        if ground_toi > res.time_of_impact {
            if let Some(normal) = Pnormal3::from_normal(res.normal) {
                let poi = res.point - res.normal * 0.01; // move inwards a little

                if let Some(fid) = state
                    .tilemap
                    .pos_to_face_id(offset.x + poi.x, offset.z + poi.z)
                {
                    state.hovered_wall_ground = Some(fid);
                    state.hovered_wall_normal = Some(normal);

                    let y_inc = ((state.tilemap.face_base_elevation(fid) as f32 * SLOPE_HEIGHT)
                        / WALL_HEIGHT) as u32;
                    let base_y = y_inc as f32 * WALL_HEIGHT;
                    state.hovered_wall_height = Some(((poi.y - base_y) / WALL_HEIGHT) as u32);
                }
            } else {
                warn!("somebody slanted the walls. {:?}", res.normal);
            }
        }
    }
}

fn change_control_mode(
    // mut controls: ResMut<EditorControls>,
    keys: Res<ButtonInput<KeyCode>>,
    // control_mode: Res<State<ControlMode>>,
    mut next_mode: ResMut<NextState<ControlMode>>,
    global_ui_state: Res<GlobalUiState>,
) {
    if global_ui_state.is_any_focused {
        return;
    }

    const X: bool = true;
    match (
        keys.just_pressed(KeyCode::Digit1),
        keys.just_pressed(KeyCode::Digit2),
        keys.just_pressed(KeyCode::Digit3),
        keys.just_pressed(KeyCode::Digit0),
    ) {
        (X, _, _, _) => next_mode.set(ControlMode::ShapeTerrain),
        (_, X, _, _) => next_mode.set(ControlMode::ShapeWalls),
        (_, _, X, _) => next_mode.set(ControlMode::Paint2D),
        (_, _, _, X) => next_mode.set(ControlMode::AdminStuff),
        _ => {}
    }
}

fn perform_click_actions(
    mut cmd: Commands,
    mut state: ResMut<EditorState>,
    mut egui_state: ResMut<ui::EguiState>,
    controls: Res<EditorControls>,
    control_mode: Res<State<ControlMode>>,
    global_ui_state: Res<GlobalUiState>,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    systems: Res<Systems>,
) {
    let over_ui = global_ui_state.is_pointer_captured || global_ui_state.is_any_focused;
    match control_mode.get() {
        ControlMode::ShapeTerrain => {
            let Some(fid) = state.hovered_ground_face else {
                return;
            };
            if !over_ui && mouse.just_pressed(MouseButton::Left) {
                match keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight) {
                    false => {
                        controls
                            .tilemap
                            .raise_face_elevation(&mut state.tilemap, fid, 1);
                        cmd.run_system(systems.recreate_ground_mesh);
                    }
                    true => {
                        controls
                            .tilemap
                            .lower_face_elevation(&mut state.tilemap, fid, 1);
                        cmd.run_system(systems.recreate_ground_mesh);
                    }
                }
            }
        }
        ControlMode::ShapeWalls => {
            let fid = match state.hovered_wall_ground {
                Some(fid) => fid,
                None => match state.hovered_ground_face {
                    Some(fid) => fid,
                    None => return,
                },
            };

            if !over_ui && mouse.just_pressed(MouseButton::Left) {
                match keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight) {
                    false => {
                        controls.tilemap.raise_wall_height(&mut state.tilemap, fid);
                        cmd.run_system(systems.recreate_ground_mesh);
                    }
                    true => {
                        controls.tilemap.lower_wall_height(&mut state.tilemap, fid);
                        cmd.run_system(systems.recreate_ground_mesh);
                    }
                }
            }
        }
        ControlMode::FlattenTerrain => todo!(),
        ControlMode::PaintTerrain3D => todo!(),
        ControlMode::Paint2D => {
            if keys.just_pressed(KeyCode::Tab) {
                egui_state.paint_widget_open = !egui_state.paint_widget_open;
            }

            if !egui_state.paint_widget_open {
                if !over_ui && mouse.just_pressed(MouseButton::Left) {
                    if let Some(fid) = state.hovered_wall_ground {
                        // paint walls
                        let Some(normal) = state.hovered_wall_normal else {
                            return;
                        };
                        let Some(height) = state.hovered_wall_height else {
                            return;
                        };
                        if let Some(coord) = state.selected_tile_coords {
                            let tid = state.tileset.grid().coord_to_id(coord);
                            controls.tilemap.paint_wall_face(
                                &mut state.tilemap,
                                fid,
                                normal,
                                height,
                                tid,
                            );
                            cmd.run_system(systems.recreate_ground_mesh);
                        }
                    } else {
                        // paint terrain
                        let Some(fid) = state.hovered_ground_face else {
                            return;
                        };

                        if let Some(coord) = state.selected_tile_coords {
                            let tid = state.tileset.grid().coord_to_id(coord);
                            controls
                                .tilemap
                                .paint_ground_face(&mut state.tilemap, fid, tid);
                            cmd.run_system(systems.recreate_ground_mesh);
                        }
                    }
                }
            }
        }
        ControlMode::PaintWalls3D => todo!(),
        ControlMode::PlaceGameObjects => todo!(),
        ControlMode::AdminStuff => {}
    }
}

fn draw_hovered_tile_gizmo(
    mut gizmos: Gizmos,
    tilemap: Res<EditorState>,
    transform: Query<&Transform, With<TilemapGroundMesh>>,
    state: Res<EditorState>,
) {
    let offset = transform.single().translation;
    let map = &tilemap.tilemap;
    let Some(hovered) = state.hovered_ground_face else {
        return;
    };
    let Some(pos) = map.face_id_to_center_pos(hovered) else {
        return;
    };
    gizmos.rect(
        Vec3::new(pos.x, 0.0, pos.y) + offset,
        Quat::from_rotation_x(PI * 0.5),
        Vec2::splat(1.),
        LIME_300,
    );
    let from = Vec3::new(pos.x, 0.5, pos.y) + offset;
    let to = Vec3::new(pos.x, 0.0, pos.y) + offset;

    gizmos.arrow(from, to, YELLOW_800);
}

pub(super) mod ui {
    use super::{
        ControlMode, EditorState, ExportLevelScenePath, Systems, DEFAULT_EDITOR_EXPORT_PATH,
        DEFAULT_EDITOR_SAVE_PATH, TILESET_PATH_DIFFUSE,
    };
    use crate::{
        framework::tileset::{TILESET_TEXTURE_DIMS, TILESET_TILE_DIMS},
        tooling::editor::{
            file_selector_widget::{
                FileSelectorWidget, FileSelectorWidgetResult, FileSelectorWidgetSettings,
            },
            tilemap_asset::TilemapRon,
            tilemap_size_widget::TilemapSizeWidget,
            tileset_widget::TilesetWidget,
        },
    };
    use bevy::{prelude::*, window::PrimaryWindow};
    use bevy_egui::{egui, EguiContexts};
    use std::path::Path;

    #[derive(Component)]
    pub struct LevelEditorInfoText;

    #[derive(Resource, Default)]
    pub(super) struct EguiState {
        pub paint_widget: Option<TilesetWidget>,
        pub paint_widget_open: bool,
        pub file_widget: Option<FileWidget>,
        pub resize_widget: Option<TilemapSizeWidget>,
    }

    pub(super) struct FileWidget {
        pub mode: FileWidgetMode,
        pub widget: FileSelectorWidget,
    }

    impl FileWidget {
        pub fn save_tilemap() -> Self {
            Self {
                mode: FileWidgetMode::SaveTilemap,
                widget: FileSelectorWidget::new(
                    DEFAULT_EDITOR_SAVE_PATH,
                    FileSelectorWidgetSettings::SAVE,
                ),
            }
        }
        pub fn load_tilemap() -> Self {
            Self {
                mode: FileWidgetMode::LoadTilemap,
                widget: FileSelectorWidget::new(
                    DEFAULT_EDITOR_SAVE_PATH,
                    FileSelectorWidgetSettings::LOAD,
                ),
            }
        }
        pub fn load_tilemap_autosave() -> Self {
            Self {
                mode: FileWidgetMode::LoadTilemap,
                widget: FileSelectorWidget::new(
                    Path::new(DEFAULT_EDITOR_SAVE_PATH).join("autosave"),
                    FileSelectorWidgetSettings {
                        select_text: "Restore",
                        ..FileSelectorWidgetSettings::LOAD
                    },
                ),
            }
        }
        pub fn export_tilemap() -> Self {
            Self {
                mode: FileWidgetMode::ExportLevel,
                widget: FileSelectorWidget::new(
                    Path::new(DEFAULT_EDITOR_EXPORT_PATH),
                    FileSelectorWidgetSettings {
                        select_text: "Export",
                        file_extension: "level",
                        ..FileSelectorWidgetSettings::SAVE
                    },
                ),
            }
        }
    }

    pub(super) enum FileWidgetMode {
        LoadTilemap,
        SaveTilemap,
        ExportLevel,
    }

    pub(super) fn setup(
        mut cmd: Commands,
        mut ui_state: ResMut<EguiState>,
        mut contexts: EguiContexts,
        ass: Res<AssetServer>,
    ) {
        let handle = ass.load(TILESET_PATH_DIFFUSE);
        let id = contexts.add_image(handle);
        ui_state.paint_widget = Some(TilesetWidget::new(
            id,
            TILESET_TEXTURE_DIMS.into(),
            TILESET_TILE_DIMS.into(),
        ));

        cmd.spawn((
            TextBundle::from_sections([TextSection::new("Level Editor", TextStyle::default())])
                .with_style(Style {
                    position_type: PositionType::Absolute,
                    top: Val::Px(5.0),
                    right: Val::Px(15.0),
                    ..default()
                }),
            LevelEditorInfoText,
        ));
    }

    pub(super) fn check_open_file_dialog(
        mut state: ResMut<EguiState>,
        keys: Res<ButtonInput<KeyCode>>,
        editor_state: Res<EditorState>,
    ) {
        if keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight) {
            if keys.just_pressed(KeyCode::KeyS) {
                state.file_widget = Some(FileWidget::save_tilemap());
            }
            if keys.just_pressed(KeyCode::KeyO) {
                state.file_widget = Some(FileWidget::load_tilemap());
            }
            if keys.just_pressed(KeyCode::KeyL) {
                state.file_widget = Some(FileWidget::load_tilemap_autosave());
            }
            if keys.just_pressed(KeyCode::KeyR) {
                state.resize_widget = match state.resize_widget.is_some() {
                    true => None,
                    false => Some(TilemapSizeWidget::new(editor_state.tilemap.dims())),
                }
            }
            if keys.just_pressed(KeyCode::KeyE) {
                state.file_widget = Some(FileWidget::export_tilemap());
            }
        }
    }

    pub(super) fn render_egui(
        mut ctxs: EguiContexts,
        mut state: ResMut<EguiState>,
        mut editor_state: ResMut<EditorState>,
        editor_mode: Res<State<ControlMode>>,
        win: Query<Entity, With<PrimaryWindow>>,
        keys: Res<ButtonInput<KeyCode>>,
        mut cmd: Commands,
        sys: Res<Systems>,
        mut export_level_scene_path: ResMut<ExportLevelScenePath>,
    ) {
        let win = win.single();
        let ctx = ctxs.ctx_for_window_mut(win);

        let mut file_select_open = true;
        match &mut state.file_widget {
            Some(widget) => {
                let mut response = None;
                let title = match widget.mode {
                    FileWidgetMode::LoadTilemap => "Load Tilemap",
                    FileWidgetMode::SaveTilemap => "Save Tilemap",
                    FileWidgetMode::ExportLevel => "Export Tilemap",
                };
                egui::Window::new(title)
                    .id("file_selector_widget_window".into())
                    .open(&mut file_select_open)
                    .show(ctx, |ui| {
                        response = widget.widget.show(ui);
                    });

                match response {
                    Some(FileSelectorWidgetResult::FileSelected(path)) => {
                        match widget.mode {
                            FileWidgetMode::LoadTilemap => match TilemapRon::read(&path) {
                                Ok(ron) => {
                                    editor_state.tilemap = ron.tilemap;
                                    cmd.run_system(sys.recreate_ground_mesh);
                                }
                                Err(e) => {
                                    error!("Failed to load tilemap {path:?}: {e:?}",)
                                }
                            },
                            FileWidgetMode::SaveTilemap => {
                                if let Err(e) =
                                    TilemapRon::new(editor_state.tilemap.clone()).write(&path)
                                {
                                    error!("Failed to save tilemap to {path:?}. {e:?}",);
                                }
                            }
                            FileWidgetMode::ExportLevel => {
                                export_level_scene_path.0 = path.to_string_lossy().into();
                                cmd.run_system(sys.export_level_scene);
                            }
                        }
                        file_select_open = false;
                    }
                    Some(FileSelectorWidgetResult::CloseRequested) => {
                        file_select_open = false;
                    }
                    None => {}
                }
                if keys.just_pressed(KeyCode::Escape) {
                    file_select_open = false;
                }
            }
            None => {}
        }
        if !file_select_open {
            let _ = state.file_widget.take();
        }

        let mut resize_widget_open = true;
        if let Some(widget) = &mut state.resize_widget {
            egui::Window::new("Resize Tilemap")
                .open(&mut resize_widget_open)
                .show(ctx, |ui| {
                    if let Some((anchor, dims)) = widget.show(ui) {
                        editor_state.tilemap.resize_anchored(dims, anchor);
                        cmd.run_system(sys.recreate_ground_mesh);
                    }
                });
            if keys.just_pressed(KeyCode::Escape) {
                resize_widget_open = false;
            }
        }
        if !resize_widget_open {
            let _ = state.resize_widget.take();
        }

        match editor_mode.get() {
            ControlMode::Paint2D => {
                if state.paint_widget_open {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        if let Some(widget) = &mut state.paint_widget {
                            if let Some(new_tile_coord) = widget.show(ui) {
                                editor_state.selected_tile_coords = Some(new_tile_coord);
                                state.paint_widget_open = false;
                            }
                        }
                    });
                } else {
                    egui::SidePanel::left("tileset_widget_panel").show(ctx, |ui| {
                        if let Some(widget) = &mut state.paint_widget {
                            if let Some(new_tile_coord) = widget.show(ui) {
                                editor_state.selected_tile_coords = Some(new_tile_coord);
                                state.paint_widget_open = false;
                            }
                        }
                    });
                }
            }
            ControlMode::AdminStuff => {}
            _ => {}
        }
    }

    pub(super) fn update_info_text(
        mode: Res<State<ControlMode>>,
        mut text: Query<&mut Text, With<LevelEditorInfoText>>,
    ) {
        let text = &mut text.single_mut().sections[0].value;
        match mode.get() {
            ControlMode::AdminStuff => *text = ["Admin Stuff"].join("\n"),
            ControlMode::ShapeTerrain => *text = ["Shape Terrain"].join("\n"),
            ControlMode::FlattenTerrain => *text = ["Flatten Terrain"].join("\n"),
            ControlMode::ShapeWalls => *text = ["Shape Walls"].join("\n"),
            ControlMode::Paint2D => *text = ["Paint 2D"].join("\n"),
            ControlMode::PaintTerrain3D => *text = ["Paint Terrain 3D"].join("\n"),
            ControlMode::PaintWalls3D => *text = ["Shape Walls 3D"].join("\n"),
            ControlMode::PlaceGameObjects => *text = ["Place Game Objects"].join("\n"),
        }
    }
}

#[derive(Resource, Default)]
struct ExportLevelScenePath(String);

fn export_level_scene(world: &mut World) {
    // let mut ass = world.resource_mut::<AssetServer>();
    let state = world.resource::<EditorState>();
    let path = world.resource::<ExportLevelScenePath>().0.clone();

    let builder = RawMeshBuilder::new(&state.tilemap);
    let mesh = builder.make_ground_mesh(&state.tileset);
    let collider =
        tilemap_mesh_builder::build_rapier_convex_collider_for_preview(&mesh.clone().into());

    let walls = builder
        .make_wall_meshes(&state.tileset)
        .into_iter()
        .map(|mesh| {
            let collider = tilemap_mesh_builder::build_rapier_convex_collider_for_preview(
                &mesh.clone().into(),
            );
            WallData { mesh, collider }
        })
        .collect();

    let level_asset = LevelAsset::new(LevelAssetData {
        ground_collider: collider,
        ground_mesh: mesh,
        walls,
    });

    match level_asset.save(path.as_str()) {
        Ok(()) => info!("Export successful!"),
        Err(e) => error!("error saving level: {e:?}"),
    }
}

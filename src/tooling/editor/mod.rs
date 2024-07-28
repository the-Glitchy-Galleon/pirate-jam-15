#![cfg(not(target_family = "wasm"))]
use crate::{
    framework::{
        global_ui_state::GlobalUiStatePlugin, logical_cursor::LogicalCursorPlugin,
        tilemap::SLOPE_HEIGHT,
    },
    game::{
        objects::{assets::GameObjectAssets, camera},
        player::AddPlayerRespawnEvent,
        LevelResources,
    },
    tooling::free_camera::FreeCameraPlugin,
};
use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::{
    plugin::{NoUserData, RapierPhysicsPlugin},
    render::{DebugRenderStyle, RapierDebugRenderPlugin},
};
use tilemap_editor::TilemapEditorPlugin;

pub mod object_def_builder;
pub mod tilemap_asset;
pub mod tilemap_controls;
pub mod tilemap_editor;
pub mod tilemap_mesh_builder;
pub mod widgets {
    pub mod file_selector;
    pub mod object_def;
    pub mod tilemap_size;
    pub mod tileset;
}

pub struct LevelEditorPlugin;

impl Plugin for LevelEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            WorldInspectorPlugin::default(),
            RapierPhysicsPlugin::<NoUserData>::default(),
            RapierDebugRenderPlugin {
                style: DebugRenderStyle {
                    // subdivisions: 1,
                    // border_subdivisions: 2,
                    collider_dynamic_color: [340.0, 1.0, 0.2, 1.0],
                    ..Default::default()
                },
                ..Default::default()
            },
            GlobalUiStatePlugin,
            LogicalCursorPlugin,
            FreeCameraPlugin {
                transform: Transform::from_xyz(0.0, SLOPE_HEIGHT * 12.0, 10.0)
                    .looking_at(Vec3::ZERO, Vec3::Y),
            },
            TilemapEditorPlugin,
        ))
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 500.,
        })
        .init_resource::<GameObjectAssets>()
        .init_resource::<LevelResources>()
        .add_event::<AddPlayerRespawnEvent>()
        .add_systems(Startup, setup);

        camera::add_systems_and_resources(app);
    }
}

fn setup(mut cmd: Commands) {
    cmd.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 10_000.0,
            shadows_enabled: true,

            ..Default::default()
        },
        transform: Transform::IDENTITY
            .with_translation(Vec3::new(5.0, 10.0, -5.0))
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
}

#![cfg(not(target_family = "wasm"))]
use crate::{
    framework::{logical_cursor::LogicalCursorPlugin, prelude::GlobalUiStatePlugin},
    FreeCameraPlugin,
};
use bevy::prelude::*;
use bevy_rapier3d::{
    plugin::{NoUserData, RapierPhysicsPlugin},
    render::RapierDebugRenderPlugin,
};
use tilemap_editor::TilemapEditorPlugin;

pub mod file_selector_widget;
pub mod tilemap_asset;
pub mod tilemap_controls;
pub mod tilemap_editor;
pub mod tilemap_mesh;
pub mod tileset_widget;
pub struct LevelEditorPlugin;

impl Plugin for LevelEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            RapierPhysicsPlugin::<NoUserData>::default(),
            RapierDebugRenderPlugin::default(),
            GlobalUiStatePlugin,
            LogicalCursorPlugin,
            FreeCameraPlugin,
            TilemapEditorPlugin,
        ))
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 2000.,
        });
    }
}

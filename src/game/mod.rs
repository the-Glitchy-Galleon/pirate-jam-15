use crate::{
    framework::{
        audio::AudioPlugin,
        global_ui_state::GlobalUiStatePlugin,
        level_asset::{LevelAsset, LevelAssetLoader},
        loading_queue::{self, AssetLoadingCompleted, AssetLoadingQueue, WatchAssetLoading},
        logical_cursor::LogicalCursorPlugin,
    },
    game::{
        game_cursor::GameCursorPlugin,
        kinematic_char::{CharacterWalkControl, CharacterWalkState},
        minion::{
            collector::{MinionInteractionRequirement, MinionStorage},
            minion_builder::MinionAssets,
            MinionKind, MinionStartedInteraction, MinionState, MinionTarget,
        },
        objects::{assets::GameObjectAssets, camera::CameraObjPlugin, cauldron},
        player::{
            minion_storage::{MinionStorageInput, MinionThrowTarget, PlayerCollector},
            player_builder::{self, PlayerAssets},
            AddPlayerRespawnEvent,
        },
        top_down_camera::{TopDownCameraBuilder, TopDownCameraPlugin},
    },
};
use bevy::{prelude::*, window::CursorGrabMode};
use bevy_rapier3d::prelude::*;
use vleue_navigator::{NavMesh, VleueNavigatorPlugin};

pub mod collision_groups;
pub mod common;
pub mod game_cursor;
pub mod kinematic_char;
pub mod level;
pub mod minion;
pub mod objects;
pub mod player;
pub mod top_down_camera;

#[derive(Debug, Default, Resource)]
pub struct LevelResources {
    pub navmesh: Option<Handle<NavMesh>>,
    pub spawnpoints: Option<Vec<(Vec3, u32, bool)>>,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            RapierPhysicsPlugin::<NoUserData>::default(),
            AudioPlugin,
            VleueNavigatorPlugin,
            LogicalCursorPlugin {
                target_grab_mode: Some((CursorGrabMode::Confined, false)),
            },
            GlobalUiStatePlugin,
            GameCursorPlugin,
            TopDownCameraPlugin,
            CameraObjPlugin,
        ))
        // Level Asset Loader
        .init_asset::<LevelAsset>()
        .init_asset_loader::<LevelAssetLoader>()
        .insert_resource(LevelResources::default())
        // Loading queue
        .init_resource::<AssetLoadingQueue<LevelAsset>>()
        .add_event::<WatchAssetLoading<LevelAsset>>()
        .add_event::<AssetLoadingCompleted<LevelAsset>>()
        .add_systems(Update, loading_queue::add_watches::<LevelAsset>)
        .add_systems(
            Update,
            loading_queue::process_asset_loading_queue::<LevelAsset>
                .after(loading_queue::add_watches::<LevelAsset>),
        )
        // Types
        .register_type::<CharacterWalkControl>()
        .register_type::<PlayerCollector>()
        .register_type::<CharacterWalkState>()
        .register_type::<MinionKind>()
        .register_type::<MinionStorage>()
        .register_type::<MinionState>()
        .register_type::<MinionTarget>()
        .register_type::<MinionThrowTarget>()
        .register_type::<MinionInteractionRequirement>()
        .insert_resource(MinionStorageInput {
            chosen_ty: MinionKind::Void,
            want_to_throw: false,
            to_where: MinionThrowTarget::Location(Vec3::ZERO),
            do_pickup: false,
        })
        .init_resource::<MinionAssets>()
        .init_resource::<GameObjectAssets>()
        .init_resource::<PlayerAssets>()
        .add_event::<MinionStartedInteraction>()
        .add_event::<AddPlayerRespawnEvent>()
        .add_systems(
            Startup,
            (
                player::setup_player,
                spawn_gameplay_camera.after(player::setup_player),
                minion::setup_chosen_minion_ui,
                level::load_preview_scene,
            ),
        )
        .add_systems(
            PreUpdate,
            (
                level::init_level,
                common::link_root_parents,
                player::player_controls.after(game_cursor::update_game_cursor),
            ),
        )
        .add_systems(
            FixedUpdate,
            (
                minion::minion_walk,
                // sneak in before the kinematic char, because the walk information gets erased
                minion::update_animation.after(minion::minion_walk),
                player_builder::update_animation.after(minion::update_animation),
                kinematic_char::update_kinematic_character.after(player_builder::update_animation),
            ),
        )
        .add_systems(
            Update,
            (
                minion::cleanup_minion_state,
                minion::update_minion_state,
                minion::minion_update_path.after(minion::update_minion_state),
                minion::walk_target::walk_target_update.after(minion::update_minion_state),
                minion::collector::update_minion_interaction_requirements
                    .after(minion::update_minion_state),
                minion::display_navigator_path,
                minion::update_chosen_minion_debug_ui,
                player::minion_storage::minion_storage_throw,
                player::minion_storage::minion_storage_pickup,
                player::add_player_respawn,
                player::process_player_respawning.after(player::add_player_respawn),
                objects::destructible_target_test::update_destructble_target,
                cauldron::process_cauldron_queue,
                cauldron::queue_minion_for_cauldron,
            ),
        )
        .add_systems(
            PostUpdate,
            minion::minion_build_path.after(TransformSystem::TransformPropagate),
        );

        /* Gizmos */
        #[cfg(feature = "debug_visuals")]
        {
            app.add_plugins((
                RapierDebugRenderPlugin::default(),
                bevy_inspector_egui::quick::WorldInspectorPlugin::new(),
            ));
            app.add_systems(
                Update,
                (
                    player::minion_storage::debug_minion_to_where_ui,
                    minion::debug_navmesh,
                    player::show_player_control_gizmos,
                    common::show_forward_gizmo,
                ),
            );
        }
    }
}

pub fn spawn_gameplay_camera(mut cmd: Commands) {
    TopDownCameraBuilder::new(7.5, 10.0).build(&mut cmd);
}

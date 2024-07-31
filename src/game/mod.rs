use crate::{
    framework::{
        audio::AudioPlugin,
        global_ui_state::GlobalUiStatePlugin,
        level_asset::{LevelAsset, LevelAssetLoader},
        loading_queue,
        logical_cursor::LogicalCursorPlugin,
    },
    game::{
        common::PrimaryCamera,
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
            AddPlayerRespawnEvent, PlayerTag,
        },
        top_down_camera::TopDownCameraControls,
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
            GlobalUiStatePlugin,
            LogicalCursorPlugin {
                target_grab_mode: Some((CursorGrabMode::Confined, false)),
            },
            GameCursorPlugin,
        ));

        #[cfg(feature = "debug_visuals")]
        app.add_plugins((
            RapierDebugRenderPlugin::default(),
            bevy_inspector_egui::quick::WorldInspectorPlugin::new(),
        ));

        loading_queue::initialize::<LevelAsset>(app);

        app.register_type::<CharacterWalkControl>()
            .register_type::<PlayerCollector>()
            .register_type::<CharacterWalkState>()
            .register_type::<MinionKind>()
            .register_type::<MinionStorage>()
            .register_type::<MinionState>()
            .register_type::<MinionTarget>()
            .register_type::<MinionThrowTarget>()
            .register_type::<MinionInteractionRequirement>()
            .init_asset::<LevelAsset>()
            .init_asset_loader::<LevelAssetLoader>()
            .insert_resource(LevelResources::default())
            .insert_resource(MinionStorageInput {
                chosen_ty: MinionKind::Void,
                want_to_throw: false,
                to_where: MinionThrowTarget::Location(Vec3::ZERO),
                do_pickup: false,
            })
            .init_resource::<MinionAssets>()
            .init_resource::<GameObjectAssets>()
            .add_event::<MinionStartedInteraction>()
            .add_event::<AddPlayerRespawnEvent>();

        /* Setup */
        app.add_systems(
            Startup,
            (
                player::setup_player,
                spawn_gameplay_camera.after(player::setup_player),
                minion::setup_chosen_minion_ui,
                level::load_preview_scene,
            ),
        );

        /* Common systems */
        app.add_systems(PreUpdate, common::link_root_parents);

        app.add_systems(
            FixedUpdate,
            (
                minion::minion_walk,
                minion::update_animation.after(minion::minion_walk),
                kinematic_char::update_kinematic_character.after(minion::update_animation),
            ),
        );

        /* Minion systems */
        app.add_systems(Update, minion::cleanup_minion_state)
            .add_systems(Update, minion::update_minion_state)
            .add_systems(
                Update,
                minion::minion_update_path
                    .run_if(resource_exists::<LevelResources>)
                    .after(minion::update_minion_state),
            )
            .add_systems(
                PostUpdate,
                minion::minion_build_path
                    .run_if(resource_exists::<LevelResources>)
                    .after(TransformSystem::TransformPropagate),
            )
            .add_systems(
                Update,
                (minion::walk_target::walk_target_update.after(minion::update_minion_state),),
            )
            .add_systems(
                Update,
                minion::collector::update_minion_interaction_requirements
                    .after(minion::update_minion_state),
            )
            .add_systems(
                Update,
                (
                    minion::display_navigator_path,
                    minion::update_chosen_minion_ui,
                ),
            );

        /* Player systems */
        app.add_systems(
            PreUpdate,
            (player::player_controls.after(game_cursor::update_game_cursor),),
        )
        .add_systems(
            Update,
            (
                player::minion_storage::minion_storage_throw,
                player::minion_storage::minion_storage_pickup,
                player::minion_storage::debug_minion_to_where_ui,
                player::add_player_respawn,
                player::process_player_respawning.after(player::add_player_respawn),
                top_down_camera::update,
            ),
        );

        /* Level and Objects */
        app.add_systems(PreUpdate, level::init_level);

        app.add_systems(
            Update,
            objects::destructible_target_test::update_destructble_target,
        );
        app.add_plugins(CameraObjPlugin);
        app.add_systems(
            Update,
            (
                cauldron::process_cauldron_queue,
                cauldron::queue_minion_for_cauldron,
            ),
        );

        /* Gizmos */
        #[cfg(feature = "debug_visuals")]
        {
            app.add_systems(
                Update,
                (
                    minion::debug_navmesh,
                    player::show_player_control_gizmos,
                    common::show_forward_gizmo,
                ),
            );
        }
    }
}

pub fn spawn_gameplay_camera(mut commands: Commands, player: Query<Entity, With<PlayerTag>>) {
    let player = player.single();
    commands.spawn((
        PrimaryCamera,
        TopDownCameraControls {
            target: Some(player),
            offset: Vec3::new(0.0, 10.0, 10.0),
        },
        Camera3dBundle {
            transform: Transform::from_xyz(-30.0, 30.0, 30.0)
                .looking_at(Vec3::new(10.0, 0.0, 7.0), Vec3::Y),
            ..Default::default()
        },
    ));
}

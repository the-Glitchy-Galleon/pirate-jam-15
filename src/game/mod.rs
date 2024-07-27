use crate::{
    framework::{
        audio::AudioPlugin,
        level_asset::{LevelAsset, LevelAssetLoader},
        loading_queue,
    },
    game::{
        kinematic_char::{CharacterWalkControl, CharacterWalkState},
        minion::{
            collector::{MinionInteractionRequirement, MinionStorage},
            MinionKind, MinionState, MinionTarget,
        },
        objects::{assets::GameObjectAssets, camera},
        player::minion_storage::{MinionStorageInput, MinionThrowTarget, PlayerCollector},
    },
};
use bevy::{input::InputSystem, prelude::*};
use bevy_rapier3d::prelude::*;
use player::{AddPlayerRespawnEvent, PlayerTag};
use top_down_camera::TopDownCameraControls;
use vleue_navigator::{NavMesh, VleueNavigatorPlugin};

pub mod collision_groups;
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
            RapierDebugRenderPlugin::default(),
            AudioPlugin,
            VleueNavigatorPlugin,
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
            .register_type::<MinionInteractionRequirement>();

        app.insert_resource(LevelResources::default());
        app.insert_resource(MinionStorageInput {
            chosen_ty: MinionKind::Void,
            want_to_throw: false,
            to_where: MinionThrowTarget::Location(Vec3::ZERO),
            do_pickup: false,
        });

        // app.insert_resource(LevelResources {
        //     navmesh: Handle::inva
        // });

        /* Setup */
        app.add_event::<AddPlayerRespawnEvent>()
            // .add_systems(Startup, setup_physics)
            .add_systems(Startup, player::setup_player)
            .add_systems(Startup, spawn_gameplay_camera.after(player::setup_player))
            .add_systems(
                Update,
                (
                    player::add_player_respawn,
                    player::process_player_respawning.after(player::add_player_respawn),
                ),
            );

        /* Common systems */
        app.add_systems(FixedUpdate, kinematic_char::update_kinematic_character);

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
                minion::minion_walk.run_if(resource_exists::<LevelResources>),
            )
            .add_systems(
                Update,
                minion::walk_target::walk_target_update.after(minion::update_minion_state),
            )
            .add_systems(
                Update,
                minion::collector::update_minion_interaction_requirements
                    .after(minion::update_minion_state),
            )
            .add_systems(
                Update,
                (minion::debug_navmesh, minion::display_navigator_path),
            );

        /* Player systems */
        app.add_systems(PreUpdate, player::player_controls.after(InputSystem))
            .add_systems(Update, player::minion_storage::minion_storage_throw)
            .add_systems(Update, player::minion_storage::minion_storage_pickup);

        app.init_asset::<LevelAsset>()
            .init_asset_loader::<LevelAssetLoader>()
            .init_resource::<GameObjectAssets>();

        app.add_systems(Startup, level::load_preview_scene);
        app.add_systems(PreUpdate, level::init_level);
        app.add_systems(Update, top_down_camera::update);

        app.add_systems(
            Update,
            objects::destructible_target_test::update_destructble_target,
        );
        camera::add_systems_and_resources(app);
    }
}

pub fn spawn_gameplay_camera(mut commands: Commands, player: Query<Entity, With<PlayerTag>>) {
    let player = player.single();

    commands.spawn((
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

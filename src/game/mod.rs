use crate::framework::prelude::{AudioPlugin, LevelAsset, LevelAssetLoader};
use bevy::prelude::*;
use bevy::{input::InputSystem, utils::HashMap};
use bevy_rapier3d::prelude::*;

mod kinematic_char;
pub mod level;
mod minion;
mod player;
pub mod top_down_camera;
pub mod objects {
    pub mod assets;
    pub mod camera;
    pub mod definitions;
}

pub use kinematic_char::*;
pub use minion::*;
use objects::{
    assets::GameObjectAssets,
    camera::{self},
};
pub use player::*;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            RapierPhysicsPlugin::<NoUserData>::default(),
            RapierDebugRenderPlugin::default(),
            AudioPlugin,
        ));

        app.register_type::<CharacterWalkControl>()
            .register_type::<PlayerCollector>()
            .register_type::<CharacterWalkState>()
            .register_type::<MinionKind>()
            .register_type::<MinionStorage>()
            .register_type::<MinionState>()
            .register_type::<MinionTarget>()
            .register_type::<MinionThrowTarget>()
            .register_type::<MinionInteractionRequirement>();

        app.insert_resource(MinionStorageInput {
            chosen_ty: MinionKind::Void,
            want_to_throw: false,
            to_where: MinionThrowTarget::Location(Vec3::ZERO),
            do_pickup: false,
        });

        /* Setup */
        app.add_systems(Startup, spawn_gameplay_camera)
            .add_systems(Startup, setup_physics)
            .add_systems(Startup, setup_player);

        /* Common systems */
        app.add_systems(FixedUpdate, update_kinematic_character);

        /* Minion systems */
        app.add_systems(Update, cleanup_minion_state)
            .add_systems(Update, update_minion_state)
            .add_systems(Update, minion_walk.after(update_minion_state))
            .add_systems(Update, walk_target_update.after(update_minion_state))
            .add_systems(
                Update,
                update_minion_interaction_requirements.after(update_minion_state),
            )
            .add_systems(Update, update_destructble_target);

        /* Player systems */
        app.add_systems(PreUpdate, player_controls.after(InputSystem))
            .add_systems(Update, minion_storage_throw)
            .add_systems(Update, minion_storage_pickup);

        app.init_asset::<LevelAsset>()
            .init_asset_loader::<LevelAssetLoader>()
            .init_resource::<GameObjectAssets>();

        app.add_systems(Startup, level::load_preview_scene);
        app.add_systems(PreUpdate, level::init_level);
        app.add_systems(Update, top_down_camera::update);
        camera::add_systems_and_resources(app);
    }
}

pub fn spawn_gameplay_camera(mut commands: Commands) {
    commands.spawn((
        // TopDownCameraControls,
        Camera3dBundle {
            transform: Transform::from_xyz(-30.0, 30.0, 100.0)
                .looking_at(Vec3::new(0.0, 10.0, 0.0), Vec3::Y),
            ..Default::default()
        },
    ));
}

fn setup_physics(mut commands: Commands) {
    /*
     * Ground
     */
    let ground_size = 200.1;
    let ground_height = 0.1;

    commands.spawn((
        TransformBundle::from(Transform::from_xyz(0.0, -ground_height, 0.0)),
        Collider::cuboid(ground_size, ground_height, ground_size),
    ));

    commands.spawn((
        DestructibleTargetBundle {
            requirement: {
                let mut map = HashMap::new();
                map.insert(MinionKind::Void, 2);

                MinionInteractionRequirement::new(map)
            },
            ..default()
        },
        TransformBundle::from(Transform::from_xyz(4.0, 0.0, 4.0)),
        Collider::cuboid(1.0, 1.0, 1.0),
        Sensor,
    ));

    /*
     * Create the cubes
     */
    let num = 2;
    let rad = 1.0;

    let shift = rad * 2.0 + rad;
    let centerx = shift * (num / 2) as f32;
    let centery = shift / 2.0;
    let centerz = shift * (num / 2) as f32;

    let mut offset = -(num as f32) * (rad * 2.0 + rad) * 0.5;
    let mut color = 0;
    let colors = [
        Hsla::hsl(220.0, 1.0, 0.3),
        Hsla::hsl(180.0, 1.0, 0.3),
        Hsla::hsl(260.0, 1.0, 0.7),
    ];

    for j in 0usize..2 {
        for i in 0..num {
            for k in 0usize..num {
                let x = i as f32 * shift - centerx + offset;
                let y = j as f32 * shift + centery + 3.0;
                let z = k as f32 * shift - centerz + offset;
                color += 1;

                commands.spawn((
                    TransformBundle::from(Transform::from_xyz(x, y, z)),
                    RigidBody::Dynamic,
                    Collider::cuboid(rad, rad, rad),
                    ColliderDebugColor(colors[color % 3]),
                ));
            }
        }

        offset -= 0.05 * rad * (num as f32 - 1.0);
    }
}

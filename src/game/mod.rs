use bevy::{
    input::InputSystem,
    prelude::*,
};
use bevy_rapier3d::prelude::*;

mod player;

pub use player::*;

use crate::framework::prelude::AudioPlugin;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<PlayerDirection>()
            .register_type::<PlayerCollector>()
            .register_type::<MovementInput>()
            .register_type::<MinionKind>()
            .register_type::<MinionStorage>();

        app.insert_resource(PlayerDirection(Dir3::X))
            .insert_resource(MinionInput {
                chosen_ty: MinionKind::Doink,
                want_to_throw: false,
                to_where: Vec3::ZERO,
            });

        app.add_plugins((
            RapierPhysicsPlugin::<NoUserData>::default(),
            RapierDebugRenderPlugin::default(),
        ))
        .init_resource::<MovementInput>()
        .add_systems(Startup, spawn_gameplay_camera)
        .add_systems(Startup, setup_physics)
        .add_systems(Startup, setup_player)
        .add_systems(PreUpdate, mouse_tap.after(InputSystem))
        .add_systems(FixedUpdate, player_movement)
        .add_systems(Update, player_minion)
        .add_systems(Update, player_minion_pickup);

        app.add_plugins(AudioPlugin);
    }
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

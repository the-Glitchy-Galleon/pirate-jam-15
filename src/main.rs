#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;
use tooling::prelude::*;

mod runner;
pub mod tooling;

fn spawn_gameplay_camera(mut commands: Commands) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-30.0, 30.0, 100.0)
            .looking_at(Vec3::new(0.0, 10.0, 0.0), Vec3::Y),
        ..Default::default()
    });
}

fn spawn_player(mut commands: Commands) {
    commands.spawn(
        (
            TransformBundle::from(Transform::from_xyz(0., 1., 0.)),
            RigidBody::Dynamic,
            Collider::cuboid(0.5, 1.0, 0.5),
        )
    );
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

                commands
                    .spawn((
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

fn main() -> AppExit {
    let mut app = runner::create_app();

    app
        //.add_plugins((EguiPlugin, WorldInspectorPlugin::new()))
        .add_plugins((
            RapierPhysicsPlugin::<NoUserData>::default(),
            RapierDebugRenderPlugin::default(),
        ))
        .add_systems(Startup, spawn_gameplay_camera)
        .add_systems(Startup, setup_physics)
        .add_systems(Startup, spawn_player)
        // .add_plugins(CursorGrabAndCenterPlugin)
        // .add_plugins(PointerCaptureCheckPlugin)
        // .add_plugins(FreeCameraPlugin)
        .add_plugins(FpsCounterPlugin);
        // .add_plugins(ScenePreviewPlugin);

    runner::run_app(&mut app)
}

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::{
    input::InputSystem,
    prelude::*,
    render::camera::RenderTarget,
    window::{PrimaryWindow, WindowRef},
};
// use bevy_egui::{egui::epaint::text::cursor, EguiPlugin};
// use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;
use tooling::prelude::*;

mod player_minion;
mod player_movement;
mod runner;
pub mod tooling;

use player_minion::*;
use player_movement::*;

fn spawn_gameplay_camera(mut commands: Commands) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-30.0, 30.0, 100.0)
            .looking_at(Vec3::new(0.0, 10.0, 0.0), Vec3::Y),
        ..Default::default()
    });
}

pub fn setup_player(mut commands: Commands) {
    let mut minion_st = MinionStorage::new();

    minion_st.add_minion(MinionKind::Doink);
    minion_st.add_minion(MinionKind::Doink);
    minion_st.add_minion(MinionKind::Doink);
    minion_st.add_minion(MinionKind::Doink);

    commands
        .spawn((
            SpatialBundle {
                transform: Transform::from_xyz(0.0, 5.0, 0.0),
                ..default()
            },
            Collider::round_cylinder(0.9, 0.3, 0.2),
            KinematicCharacterController {
                custom_mass: Some(5.0),
                up: Vec3::Y,
                offset: CharacterLength::Absolute(0.01),
                slide: true,
                autostep: Some(CharacterAutostep {
                    max_height: CharacterLength::Relative(0.3),
                    min_width: CharacterLength::Relative(0.5),
                    include_dynamic_bodies: false,
                }),
                // Don’t allow climbing slopes larger than 45 degrees.
                max_slope_climb_angle: 45.0_f32.to_radians(),
                // Automatically slide down on slopes smaller than 30 degrees.
                min_slope_slide_angle: 30.0_f32.to_radians(),
                apply_impulse_to_dynamic_bodies: true,
                snap_to_ground: None,
                ..default()
            },
            minion_st,
        ))
        .with_children(|b| {
            b.spawn((SpatialBundle { ..default() }, PlayerCollector))
                .with_children(|b| {
                    b.spawn((
                        SpatialBundle {
                            transform: Transform::from_rotation(Quat::from_rotation_z(
                                std::f32::consts::FRAC_PI_2,
                            ))
                            .with_translation(Vec3::new(3.0, -1.0, 0.0)),
                            ..default()
                        },
                        Collider::cone(3.0, 4.5),
                        RigidBody::Fixed,
                        Sensor,
                    ));
                });
        });
}

pub fn mouse_tap(
    window: Query<&Window, With<PrimaryWindow>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    rap_ctx: ResMut<RapierContext>,
    cam: Query<(&GlobalTransform, &Camera)>,
    mut gizmos: Gizmos,
    mut player: Query<(
        &mut Transform,
        &mut KinematicCharacterController,
        Option<&KinematicCharacterControllerOutput>,
    )>,
    mut movement: ResMut<MovementInput>,
    mut dir: ResMut<PlayerDirection>,
    mut minion: ResMut<MinionInput>,
) {
    let Ok(window) = window.get_single() else {
        return;
    };
    let Some(pos) = window.cursor_position() else {
        return;
    };
    let Some((cam_tf, cam)) = cam
        .iter()
        .filter(|(_, cam)| matches!(cam.target, RenderTarget::Window(WindowRef::Primary)))
        .next()
    else {
        return;
    };
    let Some(cursor_ray) = cam.viewport_to_world(cam_tf, pos) else {
        return;
    };

    // gizmos.circle(
    //     cursor_ray.origin + 10.0 * cursor_ray.direction.as_vec3(),
    //     cursor_ray.direction,
    //     1.0,
    //     Color::linear_rgb(1.0, 0.0, 0.0),
    // );

    let Some((_, ray_hit)) = rap_ctx.cast_ray_and_get_normal(
        cursor_ray.origin,
        cursor_ray.direction.as_vec3(),
        1000.0,
        true,
        default(),
    ) else {
        return;
    };

    let Ok(hit_dir) = Dir3::new(ray_hit.normal) else {
        return;
    };

    gizmos.arrow(
        ray_hit.point + ray_hit.normal * 10.0,
        ray_hit.point,
        Color::linear_rgb(1.0, 0.0, 0.0),
    );

    gizmos.circle(
        ray_hit.point,
        hit_dir,
        3.0,
        Color::linear_rgb(1.0, 0.0, 0.0),
    );

    let Ok((player_tf, _, _)) = player.get_single_mut() else {
        return;
    };
    let walk_dir = (ray_hit.point - player_tf.translation).normalize_or_zero();

    if let Ok(walk_dir) = Dir3::new(walk_dir) {
        dir.0 = walk_dir;
    }

    if mouse_buttons.pressed(MouseButton::Left) {
        movement.0 = walk_dir;
    }

    if mouse_buttons.just_pressed(MouseButton::Right) {
        minion.to_where = ray_hit.point;
        minion.want_to_throw = true;
    }
}

// fn move_player

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

// fn handle_input(
//     keyboard: Res<ButtonInput<KeyCode>>,
//     mut movement: ResMut<MovementInput>,
// ) {
//     if keyboard.pressed(KeyCode::KeyW) {
//         movement.z -= 1.0;
//     }
//     if keyboard.pressed(KeyCode::KeyS) {
//         movement.z += 1.0
//     }
//     if keyboard.pressed(KeyCode::KeyA) {
//         movement.x -= 1.0;
//     }
//     if keyboard.pressed(KeyCode::KeyD) {
//         movement.x += 1.0
//     }
//     **movement = movement.normalize_or_zero();
//     if keyboard.pressed(KeyCode::ShiftLeft) {
//         **movement *= 2.0;
//     }
//     if keyboard.pressed(KeyCode::Space) {
//         movement.y = 1.0;
//     }
// }

fn main() -> AppExit {
    let mut app = runner::create_app();

    app.insert_resource(PlayerDirection(Dir3::X))
        .insert_resource(MinionInput {
            chosen_ty: MinionKind::Doink,
            want_to_throw: false,
            to_where: Vec3::ZERO,
        });

    app
        //.add_plugins((EguiPlugin, WorldInspectorPlugin::new()))
        .add_plugins((
            RapierPhysicsPlugin::<NoUserData>::default(),
            RapierDebugRenderPlugin::default(),
        ))
        .init_resource::<MovementInput>()
        .add_systems(Startup, spawn_gameplay_camera)
        .add_systems(Startup, setup_physics)
        .add_systems(Startup, setup_player)
        // .add_systems(PreUpdate, handle_input.after(InputSystem))
        .add_systems(PreUpdate, mouse_tap.after(InputSystem))
        // .add_systems(Update, mouse_tap)
        .add_systems(FixedUpdate, player_movement)
        .add_systems(Update, player_minion)
        .add_systems(Update, player_minion_pickup)
        // .add_plugins(CursorGrabAndCenterPlugin)
        // .add_plugins(PointerCaptureCheckPlugin)
        // .add_plugins(FreeCameraPlugin)
        .add_plugins(FpsCounterPlugin);
    // .add_plugins(ScenePreviewPlugin);

    runner::run_app(&mut app)
}

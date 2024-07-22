pub mod player_minion;
pub mod player_movement;

pub use player_minion::*;
pub use player_movement::*;

use bevy::{
    prelude::*,
    render::camera::RenderTarget,
    window::{PrimaryWindow, WindowRef},
};
use bevy_rapier3d::prelude::*;

pub fn spawn_gameplay_camera(mut commands: Commands) {
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

pub fn player_controls(
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
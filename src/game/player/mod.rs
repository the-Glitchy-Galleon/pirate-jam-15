pub mod minion_storage;

pub use minion_storage::*;

use bevy::{
    prelude::*,
    render::camera::RenderTarget,
    window::{PrimaryWindow, WindowRef},
};
use bevy_rapier3d::prelude::*;

use super::{
    collision_groups::*, CharacterWalkControl, KinematicCharacterBundle, MinionKind, MinionStorage,
    MinionTarget,
};

#[derive(Clone, Copy, Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct PlayerTag;

pub fn setup_player(mut commands: Commands) {
    let mut minion_st = MinionStorage::new();

    minion_st.add_minion(MinionKind::Void);
    minion_st.add_minion(MinionKind::Void);
    minion_st.add_minion(MinionKind::Void);
    minion_st.add_minion(MinionKind::Void);

    commands
        .spawn((
            PlayerTag,
            minion_st,
            Collider::round_cylinder(0.9, 0.3, 0.2),
            CollisionGroups {
                memberships: G_PLAYER,
                filters: G_ALL,
            },
            SpatialBundle {
                transform: Transform::from_xyz(0.0, 5.0, 0.0),
                ..default()
            },
            KinematicCharacterBundle::default(),
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
                        ActiveCollisionTypes::KINEMATIC_STATIC,
                        Collider::cone(3.0, 4.5),
                        CollisionGroups {
                            memberships: G_SENSOR,
                            filters: G_MINION,
                        },
                        RigidBody::Fixed,
                        Sensor,
                    ));
                });
        });
}

pub fn player_controls(
    window: Query<&Window, With<PrimaryWindow>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    rap_ctx: ResMut<RapierContext>,
    cam: Query<(&GlobalTransform, &Camera)>,
    mut gizmos: Gizmos,
    mut player: Query<(&mut Transform, &mut CharacterWalkControl), With<PlayerTag>>,
    mut minion: ResMut<MinionStorageInput>,
    mut minion_targets: Query<Entity, With<MinionTarget>>,
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

    let Some((ent_hit, ray_hit)) = rap_ctx.cast_ray_and_get_normal(
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

    let Ok((player_tf, mut walk)) = player.get_single_mut() else {
        return;
    };
    let walk_dir = (ray_hit.point - player_tf.translation).normalize_or_zero();

    walk.direction = walk_dir;
    walk.do_move = mouse_buttons.pressed(MouseButton::Right);

    minion.to_where = MinionThrowTarget::Location(ray_hit.point);
    if minion_targets.contains(ent_hit) {
        minion.to_where = MinionThrowTarget::Ent(ent_hit);
    }

    minion.want_to_throw = mouse_buttons.just_pressed(MouseButton::Left);
    minion.do_pickup = keyboard.pressed(KeyCode::KeyQ);
}

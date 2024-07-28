use std::time::Duration;

use crate::game::{
    collision_groups::{ACTOR_GROUP, DETECTION_GROUP, GROUND_GROUP, TARGET_GROUP, WALL_GROUP},
    kinematic_char::KinematicCharacterBundle,
    minion::collector::MinionStorage,
    player::minion_storage::{MinionStorageInput, MinionThrowTarget, PlayerCollector},
    CharacterWalkControl, LevelResources, MinionKind, MinionTarget,
};
use bevy::{
    prelude::Real,
    prelude::*,
    render::camera::RenderTarget,
    window::{PrimaryWindow, WindowRef},
};
use bevy_rapier3d::prelude::*;
use vleue_navigator::NavMesh;

use super::objects::camera::Shineable;

pub mod minion_storage;

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
            CollisionGroups::new(ACTOR_GROUP, GROUND_GROUP | WALL_GROUP),
            SpatialBundle {
                transform: Transform::from_xyz(0.0, 5.0, 0.0),
                visibility: Visibility::Hidden,
                ..default()
            },
            KinematicCharacterBundle::default(),
            Shineable,
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
                        CollisionGroups::new(DETECTION_GROUP, ACTOR_GROUP),
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
    minion_targets: Query<Entity, With<MinionTarget>>,
    level_reses: Option<Res<LevelResources>>,
    navmeshes: Res<Assets<NavMesh>>,
) {
    let Ok(window) = window.get_single() else {
        return;
    };
    let Some(pos) = window.cursor_position() else {
        return;
    };
    let Some(level_reses) = level_reses else {
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
    let Some(navmesh) = &level_reses.navmesh else {
        return;
    };
    let Some(navmesh) = navmeshes.get(navmesh) else {
        return;
    };

    let Some((ent_hit, ray_hit)) = rap_ctx.cast_ray_and_get_normal(
        cursor_ray.origin,
        cursor_ray.direction.as_vec3(),
        bevy_rapier3d::math::Real::INFINITY,
        true,
        QueryFilter {
            groups: Some(CollisionGroups::new(
                Group::all(),
                GROUND_GROUP | WALL_GROUP | TARGET_GROUP,
            )),
            ..default()
        },
    ) else {
        return;
    };

    let Ok(hit_dir) = Dir3::new(ray_hit.normal) else {
        return;
    };

    let color = if navmesh.transformed_is_in_mesh(ray_hit.point) {
        Color::linear_rgb(0.0, 1.0, 0.0)
    } else {
        Color::linear_rgb(1.0, 0.0, 0.0)
    };

    gizmos.arrow(ray_hit.point + ray_hit.normal * 10.0, ray_hit.point, color);

    gizmos.circle(ray_hit.point, hit_dir, 3.0, color);

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

#[derive(Component)]
pub struct PlayerRespawning {
    pub origin: Vec3,
    position: Vec3,
    timer: Timer,
}

impl PlayerRespawning {
    pub fn new(origin: Vec3, position: Vec3) -> Self {
        Self {
            origin,
            position,
            timer: Timer::new(Duration::from_secs_f32(2.0), TimerMode::Once),
        }
    }
}

#[derive(Event)]
pub struct AddPlayerRespawnEvent {
    pub position: Vec3,
}

pub fn add_player_respawn(
    mut cmd: Commands,
    player: Query<(Entity, &GlobalTransform), With<PlayerTag>>,
    mut respawn: EventReader<AddPlayerRespawnEvent>,
) {
    let Some(respawn) = respawn.read().last() else {
        return;
    };

    let (player, gx) = player.single();
    cmd.entity(player).insert((
        PlayerRespawning::new(gx.translation(), respawn.position + Vec3::Y * 1.2),
        ColliderDisabled,
    ));
}

pub fn process_player_respawning(
    mut cmd: Commands,
    mut respawn: Query<(
        Entity,
        &mut Visibility,
        &mut Transform,
        &GlobalTransform,
        &mut PlayerRespawning,
    )>,
    time: Res<Time<Real>>,
) {
    for (ent, mut vis, mut tx, gx, mut respawn) in respawn.iter_mut() {
        respawn.timer.tick(time.delta());
        let offset = tx.translation - gx.translation();
        let origin = respawn.origin + offset;
        let target = respawn.position + offset;

        if respawn.timer.finished() {
            tx.translation = target;
            *vis = Visibility::Inherited;
            cmd.entity(ent)
                .remove::<PlayerRespawning>()
                .remove::<ColliderDisabled>();
        } else {
            tx.translation = Vec3::lerp(origin, target, respawn.timer.fraction());
        }
    }
}

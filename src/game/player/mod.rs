use crate::game::{
    collision_groups::{ACTOR_GROUP, DETECTION_GROUP, GROUND_GROUP, TARGET_GROUP, WALL_GROUP},
    game_cursor::GameCursor,
    kinematic_char::KinematicCharacterBundle,
    minion::collector::MinionStorage,
    objects::{camera::Shineable, definitions::ColorDef},
    player::minion_storage::{MinionStorageInput, MinionThrowTarget, PlayerCollector},
    CharacterWalkControl, MinionKind,
};
use bevy::prelude::{Real, *};
use bevy_rapier3d::prelude::*;
use std::time::Duration;

#[cfg(feature = "debug_visuals")]
use {crate::game::LevelResources, vleue_navigator::NavMesh};

pub mod minion_storage;

#[derive(Clone, Copy, Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct PlayerTag;

pub fn setup_player(mut commands: Commands) {
    let mut minion_st = MinionStorage::new();

    for color in ColorDef::VARIANTS {
        let kind = MinionKind::from(color);
        for _ in 0..5 {
            minion_st.add_minion(kind);
        }
    }

    commands
        .spawn((
            PlayerTag,
            minion_st,
            Collider::round_cylinder(0.9, 0.3, 0.2),
            CollisionGroups::new(ACTOR_GROUP | TARGET_GROUP, GROUND_GROUP | WALL_GROUP),
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
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player: Query<(&mut Transform, &mut CharacterWalkControl), With<PlayerTag>>,
    mut minion: ResMut<MinionStorageInput>,

    cursor: Res<GameCursor>,
) {
    let Ok((player_tf, mut walk)) = player.get_single_mut() else {
        return;
    };

    walk.direction = match &cursor.hit {
        Some(hit) => (hit.point - player_tf.translation).normalize_or_zero(),
        None => {
            // Should probably clear the direction, cursor not in window.
            walk.direction
        }
    };
    walk.do_move = mouse_buttons.pressed(MouseButton::Right);

    minion.to_where = match (cursor.lock, &cursor.hit) {
        (Some(lock), _) => MinionThrowTarget::Ent(lock),
        (None, Some(hit)) => MinionThrowTarget::Location(hit.point),
        _ => {
            // Should probably clear to_where, cursor not in window.
            minion.to_where
        }
    };

    minion.want_to_throw = mouse_buttons.just_pressed(MouseButton::Left);
    minion.do_pickup = keyboard.pressed(KeyCode::KeyQ);

    if keyboard.just_pressed(KeyCode::Digit1) {
        minion.chosen_ty = MinionKind::Void;
    }
    if keyboard.just_pressed(KeyCode::Digit2) {
        minion.chosen_ty = MinionKind::Red;
    }
    if keyboard.just_pressed(KeyCode::Digit3) {
        minion.chosen_ty = MinionKind::Green;
    }
    if keyboard.just_pressed(KeyCode::Digit4) {
        minion.chosen_ty = MinionKind::Blue;
    }
    if keyboard.just_pressed(KeyCode::Digit5) {
        minion.chosen_ty = MinionKind::Yellow;
    }
    if keyboard.just_pressed(KeyCode::Digit6) {
        minion.chosen_ty = MinionKind::Magenta;
    }
    if keyboard.just_pressed(KeyCode::Digit7) {
        minion.chosen_ty = MinionKind::Cyan;
    }
    if keyboard.just_pressed(KeyCode::Digit8) {
        minion.chosen_ty = MinionKind::White;
    }
}

#[cfg(feature = "debug_visuals")]
pub fn show_player_control_gizmos(
    cursor: Res<GameCursor>,
    mut gizmos: Gizmos,
    navmeshes: Res<Assets<NavMesh>>,
    level_reses: Option<Res<LevelResources>>,
) {
    let Some(level_reses) = level_reses else {
        return;
    };
    let Some(navmesh) = &level_reses.navmesh else {
        return;
    };
    let Some(navmesh) = navmeshes.get(navmesh) else {
        return;
    };
    match &cursor.hit {
        Some(hit) => {
            let color = if navmesh.transformed_is_in_mesh(hit.point) {
                Color::linear_rgb(0.0, 1.0, 0.0)
            } else {
                Color::linear_rgb(1.0, 0.0, 0.0)
            };

            gizmos.arrow(hit.point + hit.normal * 5.0, hit.point, color);
            if let Ok(dir) = Dir3::new(hit.normal) {
                gizmos.circle(hit.point, dir, 3.0, color);
            }
        }
        _ => {}
    }
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

use crate::game::{
    game_cursor::GameCursor,
    player::minion_storage::{MinionStorageInput, MinionThrowTarget},
    CharacterWalkControl, MinionKind,
};
use bevy::prelude::{Real, *};
use bevy_rapier3d::prelude::*;
use player_builder::{PlayerAssets, PlayerBuilder, PlayerMeshTag, COLLIDER_HALF_HEIGHT};
use std::time::Duration;

#[cfg(feature = "debug_visuals")]
use {crate::game::LevelResources, vleue_navigator::NavMesh};

pub mod minion_storage;
pub mod player_builder;

#[derive(Clone, Copy, Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct PlayerTag;

pub fn setup_player(mut cmd: Commands, assets: Res<PlayerAssets>) {
    PlayerBuilder::new().build(&mut cmd, &assets);
}

pub fn player_controls(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player: Query<(&mut Transform, &mut CharacterWalkControl), With<PlayerTag>>,
    mut minion: ResMut<MinionStorageInput>,
    cursor: Res<GameCursor>,
    // mut camera: Query<&mut TopDownCamera>,
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

    // if keyboard.just_pressed(KeyCode::Space) {
    //     let mut camera = camera.single_mut();

    //     if let Some(hit) = &cursor.hit {
    //         let direction = (hit.point - player_tf.translation).normalize();
    //         camera.set_target_angle_from_direction(direction);
    //     } else {
    //         camera.set_target_angle_from_direction(*player_tf.forward());
    //     }
    // }

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
    mesh: Query<Entity, With<PlayerMeshTag>>,
    mut respawn: EventReader<AddPlayerRespawnEvent>,
) {
    let Some(respawn) = respawn.read().last() else {
        return;
    };

    let (player, gx) = player.single();
    let mesh = mesh.single();
    cmd.entity(player).insert((
        PlayerRespawning::new(
            gx.translation(),
            // idk why he still gets stuck in the floor
            respawn.position + Vec3::Y * (COLLIDER_HALF_HEIGHT + 3.0),
        ),
        ColliderDisabled,
    ));
    cmd.entity(mesh).insert(Visibility::Hidden);
}

pub fn process_player_respawning(
    mut cmd: Commands,
    mut respawn: Query<(
        Entity,
        &mut Transform,
        &GlobalTransform,
        &mut PlayerRespawning,
    ), (With<PlayerTag>, Without<PlayerMeshTag>)>,
    mut mesh: Query<(Entity, &mut Transform), (With<PlayerMeshTag>, Without<PlayerTag>)>,
    time: Res<Time<Real>>,
) {
    for (ent, mut tx, gx, mut respawn) in respawn.iter_mut() {
        respawn.timer.tick(time.delta());
        let offset = tx.translation - gx.translation();
        let origin = respawn.origin + offset;
        let target = respawn.position + offset;

        if respawn.timer.finished() {
            tx.translation = target;
            cmd.entity(ent)
                .remove::<PlayerRespawning>()
                .remove::<ColliderDisabled>();

            let (mesh, mut mesh_tx) = mesh.single_mut();
            mesh_tx.translation = tx.translation;
            cmd.entity(mesh).insert(Visibility::Visible);
        } else {
            tx.translation = Vec3::lerp(origin, target, respawn.timer.fraction());
        }
    }
}

use bevy::prelude::*;
use bevy_rapier3d::prelude::Collider;

mod collector;
mod destructible_target;
mod walk_target;

pub use collector::*;
pub use destructible_target::*;
use polyanya::Path;
use vleue_navigator::{NavMesh, TransformedPath};
pub use walk_target::*;

use super::{CharacterWalkControl, KinematicCharacterBundle, LevelResources, PlayerTag};

const MINION_INTERRACTION_RANGE: f32 = 0.5;
const MINION_NODE_DIST: f32 = 0.1;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Component, Reflect, Default)]
pub enum MinionKind {
    #[default]
    Void,
    Red,
    Green,
    Blue,
    Yellow,
    Magenta,
    Cyan,
    White,
}

/// A component to mark an eligible target for the minions. The presence of that
/// component does not mean that it is currently being targetted.
#[derive(Clone, Copy, Default, Debug, Component, Reflect)]
#[reflect(Component)]
pub struct MinionTarget;

#[derive(Clone, Copy, Default, Debug, Component, Reflect, PartialEq, Eq)]
#[reflect(Component)]
pub enum MinionState {
    #[default]
    Idling,
    GoingToPlayer,
    GoingTo(Entity),
    Interracting(Entity),
}

#[derive(Bundle, Default)]
pub struct MinionBundle {
    pub spatial: SpatialBundle,
    pub collider: Collider,
    pub character: KinematicCharacterBundle,
    pub kind: MinionKind,
    pub state: MinionState,
}

#[derive(Component)]
pub struct MinionPath(TransformedPath);

// TODO: render it more aligned to the level
pub fn debug_navmesh(
    level_reses: Option<Res<LevelResources>>,
    navmeshes: Res<Assets<NavMesh>>,
    mut gizmos: Gizmos,
) {
    let Some(navmesh) = level_reses.as_ref()
            .map(|x| &x.navmesh)
        else { return; };
    let Some(navmesh) = navmeshes.get(navmesh.id())
        else { return; };
    let red = LinearRgba {
        red: 1.0,
        green: 0.0,
        blue: 0.0,
        alpha: 1.0,
    };
    let verts = &navmesh.get().vertices;

    for poly in &navmesh.get().polygons {
        let fst = poly.vertices.iter()
            .map(|x| *x)
            .map(|x| verts[x as usize].coords)
            .map(|v| Vec3::new(v.x, 0.0, v.y));
        let snd = poly.vertices.iter()
            .map(|x| *x)
            .skip(1)
            .chain(std::iter::once(poly.vertices[0]))
            .map(|x| verts[x as usize].coords)
            .map(|v| Vec3::new(v.x, 0.0, v.y));
        for (start, end) in fst.zip(snd) {
            gizmos.line(start, end, red);

            let center = (start + end) / 2.0;
            let dir = end - start;
            let ort = Vec3::new(-dir.z, 0.0, dir.x);
            gizmos.line(
                center,
                center + 0.3 * ort.normalize_or_zero(),
                red,
            );
        }
    }
}

pub fn minion_walk(
    level_reses: Res<LevelResources>,
    navmeshes: Res<Assets<NavMesh>>,
    mut minion_q: Query<(Entity, &GlobalTransform, &mut CharacterWalkControl, &mut MinionState, Option<&mut MinionPath>)>,
    target_q: Query<&GlobalTransform, With<MinionTarget>>,
    player_q: Query<&GlobalTransform, With<PlayerTag>>,
    mut commands: Commands,
) {
    let Ok(player_tf) = player_q.get_single() else {
        return;
    };
    let Some(navmesh) = navmeshes.get(&level_reses.navmesh) else {
        return;
    };
    let mut local_path = None;

    for (ent, tf, mut walk, mut state, mut path) in minion_q.iter_mut() {
        let target_pos = match state.as_ref() {
            MinionState::GoingToPlayer => player_tf.translation(),
            MinionState::GoingTo(target) => match target_q.get(*target) {
                Ok(tf) => tf.translation(),
                Err(e) => {
                    warn!("Failed to get target pos: {e}");
                    continue;
                }
            },
            _ => continue,
        };
        let validate_path = |path: &TransformedPath| {
            let Some(last) = path.path.last().map(|x| *x)
                else { return false; };

            let target_pos = Vec3::new(target_pos.x, 0.0, target_pos.z);
            last.distance(target_pos) < MINION_NODE_DIST
        };

        let path = match path.as_deref_mut() {
            Some(p) if validate_path(&p.0) => &mut p.0,
            _ => {
                if !navmesh.transformed_is_in_mesh(tf.translation()) {
                    error!("Minion is not in the navigation");
                    continue;
                }
                if !navmesh.transformed_is_in_mesh(target_pos) {
                    error!("Minion target is not in the navigation");
                    continue;
                }

                local_path = navmesh.transformed_path(
                    Vec3::new(tf.translation().x, 0.0, tf.translation().z),
                    Vec3::new(target_pos.x, 0.0, target_pos.z),
                );
                match &mut local_path {
                    Some(x) => x,
                    None => {
                        // *state = MinionState::Idling;
                        continue;
                    },
                }
            },
        };

        if let Some(p) = path.path.first().map(|x| *x) {
            let minion_pos = navmesh.transform().transform_point(tf.translation()).xy();
            let p = navmesh.transform().transform_point(p).xy();
            if p.distance(minion_pos) <= MINION_NODE_DIST {
                path.path.pop();
            }
        }

        if let Some(next) = path.path.first().map(|x| *x) {
            walk.do_move = true;
            walk.direction = next - Vec3::new(tf.translation().x, 0.0, tf.translation().z);
        }

        if let Some(p) = local_path.take() {
            commands.entity(ent).insert(MinionPath(p));
        }
    }
}

pub fn update_minion_state(
    mut minion_q: Query<(&GlobalTransform, &mut MinionState)>,
    target_q: Query<&GlobalTransform, With<MinionTarget>>,
    player_q: Query<&GlobalTransform, With<PlayerTag>>,
) {
    let Ok(player_tf) = player_q.get_single() else {
        return;
    };

    for (tf, mut state) in minion_q.iter_mut() {
        let dist_check = |p| tf.translation().distance(p) < MINION_INTERRACTION_RANGE;

        match *state {
            MinionState::GoingToPlayer => {
                if dist_check(player_tf.translation()) {
                    *state = MinionState::Idling;
                }
            }
            MinionState::GoingTo(target) => {
                let Ok(target_tf) = target_q.get(target) else {
                    *state = MinionState::Idling;
                    continue;
                };

                if dist_check(target_tf.translation()) {
                    *state = MinionState::Interracting(target);
                }
            }
            _ => (),
        }
    }
}

pub fn cleanup_minion_state(
    mut minion_q: Query<&mut MinionState>,
    target_q: Query<(), With<MinionTarget>>,
) {
    for mut st in minion_q.iter_mut() {
        match *st {
            MinionState::GoingTo(target) if !target_q.contains(target) => {
                *st = MinionState::Idling;
            }
            MinionState::Interracting(target) if !target_q.contains(target) => {
                *st = MinionState::Idling;
            }
            _ => continue,
        }
    }
}

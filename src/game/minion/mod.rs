use bevy::prelude::*;
use bevy_rapier3d::prelude::Collider;

mod walk_target;
mod collector;
mod destructible_target;

pub use collector::*;
pub use walk_target::*;
pub use destructible_target::*;

use super::{CharacterWalkControl, KinematicCharacterBundle, PlayerTag};

const MINION_INTERRACTION_RANGE: f32 = 0.5;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Component, Reflect, Default)]
pub enum MinionKind {
    #[default]
    Spoink,
    Doink,
    Woink,
}

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

pub fn minion_walk(
    mut minion_q: Query<(&GlobalTransform, &mut CharacterWalkControl, &MinionState)>,
    target_q: Query<&GlobalTransform, With<MinionTarget>>,
    player_q: Query<&GlobalTransform, With<PlayerTag>>,
) {
    let Ok(player_tf) = player_q.get_single() else {
        return;
    };

    for (tf, mut walk, state) in minion_q.iter_mut() {
        let target_pos = match state {
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

        walk.do_move = true;
        walk.direction = target_pos - tf.translation();
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

pub fn cleanup_minion_target(
    minion_q: Query<&MinionState>,
    target_q: Query<Entity, With<MinionTarget>>,
    mut commands: Commands,
) {
    for target in target_q.iter() {
        let count = minion_q
            .iter()
            .filter_map(|st| match st {
                MinionState::GoingTo(t) => Some(t),
                MinionState::Interracting(t) => Some(t),
                _ => None,
            })
            .filter(|t| **t == target)
            .count();

        if count > 0 {
            continue;
        }

        commands.entity(target).remove::<MinionTarget>();
    }
}

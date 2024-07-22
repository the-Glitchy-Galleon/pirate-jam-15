use bevy::prelude::*;

use super::PlayerTag;

const MINION_TARGET_RANGE: f32 = 0.1;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Component, Reflect)]
pub enum MinionKind {
    Spoink,
    Doink,
    Woink,
}

#[derive(Clone, Copy, Default, Debug)]
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct MinionTarget;

#[derive(Clone, Copy, Default, Debug)]
#[derive(Component, Reflect)]
#[reflect(Component)]
pub enum MinionState {
    #[default]
    Idling,
    GoingToPlayer,
    GoingTo(Entity),
    Interracting(Entity),
}

pub fn update_minion_state(
    mut minion_q: Query<(&GlobalTransform, &mut MinionState)>,
    target_q: Query<&GlobalTransform, With<MinionTarget>>,
    player_q: Query<&GlobalTransform, With<PlayerTag>>,
) {
    let Ok(player_tf) = player_q.get_single()
        else { return; };

    for (tf, mut state) in minion_q.iter_mut() {
        let dist_check = |p| {
            tf.translation().distance(p) < MINION_TARGET_RANGE
        };

        match *state {
            MinionState::GoingToPlayer => if dist_check(player_tf.translation()) {
                *state = MinionState::Idling;
            },
            MinionState::GoingTo(target) => {
                let Ok(target_tf) = target_q.get(target)
                    else { *state = MinionState::Idling; continue; };

                if dist_check(target_tf.translation()) {
                    *state = MinionState::Interracting(target);
                }
            },
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
            },
            MinionState::Interracting(target) if !target_q.contains(target) => {
                *st = MinionState::Idling;
            },
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
        let count = minion_q.iter()
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
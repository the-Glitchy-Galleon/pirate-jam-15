use bevy::prelude::*;

use super::{MinionState, MinionTarget};

#[derive(Bundle)]
pub struct WalkTargetBundle {
    pub spatial: SpatialBundle,
    pub target_tag: MinionTarget,
}

/// Unlike other potential targets -- the walk target is purely temporary
/// and should be removed the moment all minions reach it.
pub fn walk_target_update(
    minion_q: Query<&MinionState>,
    target_q: Query<Entity, With<MinionTarget>>,
    mut commands: Commands,
) {
    for target in target_q.iter() {
        let count = minion_q
            .iter()
            .filter_map(|st| match st {
                MinionState::GoingTo(t) => Some(t),
                _ => None,
            })
            .filter(|t| **t == target)
            .count();

        if count > 0 {
            continue;
        }

        commands.entity(target).despawn_recursive();
    }
}

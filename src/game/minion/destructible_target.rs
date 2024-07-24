//! Example to demonstrate simple usage of minion requirement

use bevy::prelude::*;

use super::{MinionInteractionRequirement, MinionTarget};

#[derive(Clone, Copy, Component, Debug, Reflect)]
pub struct DestructibleTargetTag;

#[derive(Bundle)]
pub struct DestructibleTargetBundle {
    pub destro_tag: DestructibleTargetTag,
    pub requirement: MinionInteractionRequirement,
    pub target_tag: MinionTarget,
}

pub fn update_destructble_target(
    mut commands: Commands,
    destro_q: Query<(Entity, &MinionInteractionRequirement), With<DestructibleTargetTag>>
) {
    for (ent, req) in destro_q.iter() {
        if !req.is_satisfied {
            continue;
        }

        commands.entity(ent).despawn_recursive();
    }
}
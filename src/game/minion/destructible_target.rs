//! Example to demonstrate simple usage of minion requirement

use crate::game::minion::{collector::MinionInteractionRequirement, MinionTarget};
use bevy::prelude::*;

#[derive(Clone, Copy, Component, Debug, Reflect, Default)]
pub struct DestructibleTargetTag;

#[derive(Bundle, Default)]
pub struct DestructibleTargetBundle {
    pub destro_tag: DestructibleTargetTag,
    pub requirement: MinionInteractionRequirement,
    pub target_tag: MinionTarget,
}

pub fn update_destructble_target(
    mut commands: Commands,
    destro_q: Query<(Entity, &MinionInteractionRequirement), With<DestructibleTargetTag>>,
) {
    for (ent, req) in destro_q.iter() {
        if !req.is_satisfied {
            continue;
        }

        commands.entity(ent).despawn_recursive();
    }
}

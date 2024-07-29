//! Example to demonstrate simple usage of minion requirement

use crate::game::{
    collision_groups::{ACTOR_GROUP, GROUND_GROUP, TARGET_GROUP},
    minion::{collector::MinionInteractionRequirement, MinionKind, MinionTarget},
    objects::{assets::GameObjectAssets, definitions::ObjectDef},
};
use bevy::{prelude::*, utils::HashMap};
use bevy_rapier3d::prelude::*;

#[derive(Clone, Copy, Component, Debug, Reflect, Default)]
pub struct DestructibleTargetTag;

#[derive(Bundle, Default)]
pub struct DestructibleTargetBundle {
    pub destro_tag: DestructibleTargetTag,
    pub requirement: MinionInteractionRequirement,
    pub target_tag: MinionTarget,
}

pub struct DestructibleTargetTestBuilder<'a>(pub &'a ObjectDef);

impl DestructibleTargetTestBuilder<'_> {
    pub fn build(self, cmd: &mut Commands, _assets: &GameObjectAssets) -> Entity {
        cmd.spawn((
            DestructibleTargetBundle {
                requirement: {
                    let mut map = HashMap::new();
                    map.insert(MinionKind::Void, 2);

                    MinionInteractionRequirement::new(map)
                },
                ..default()
            },
            TransformBundle::from(Transform::from_translation(self.0.position)),
            Collider::cuboid(1.0, 1.0, 1.0),
            CollisionGroups::new(TARGET_GROUP | GROUND_GROUP, GROUND_GROUP | ACTOR_GROUP),
        ))
        .id()
    }
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

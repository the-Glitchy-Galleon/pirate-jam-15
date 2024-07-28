use crate::game::{
    collision_groups::TARGET_GROUP,
    minion::{MinionPath, MinionState, MinionTarget},
};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct WalkTargetTag;

#[derive(Bundle, Debug)]
pub struct WalkTargetBundle {
    pub spatial: SpatialBundle,
    pub target_tag: MinionTarget,
    pub walk_tag: WalkTargetTag,
    pub walk_collider: Collider,
    pub group: CollisionGroups,
}

impl Default for WalkTargetBundle {
    fn default() -> Self {
        Self {
            spatial: Default::default(),
            target_tag: Default::default(),
            walk_tag: Default::default(),
            walk_collider: Collider::cuboid(2.0, 10.0, 2.0),
            group: CollisionGroups {
                memberships: TARGET_GROUP,
                filters: TARGET_GROUP,
            },
        }
    }
}

/// Unlike other potential targets -- the walk target is purely temporary
/// and should be removed the moment all minions reach it.
pub fn walk_target_update(
    minion_q: Query<(&MinionState, Option<&MinionPath>)>,
    target_q: Query<Entity, (With<MinionTarget>, With<WalkTargetTag>)>,
    mut commands: Commands,
) {
    for target in target_q.iter() {
        let count = minion_q
            .iter()
            .filter_map(|(st, pt)| match (st, pt) {
                (MinionState::GoingTo(t), _) => Some(*t),
                (_, Some(pt)) if pt.0.path.len() > 0 => Some(target), // keep as long theres a path?
                _ => None,
            })
            .filter(|t| *t == target)
            .count();

        if count > 0 {
            continue;
        }
        info!("Removing path through walk_target_update");
        commands.entity(target).despawn_recursive();
    }
}

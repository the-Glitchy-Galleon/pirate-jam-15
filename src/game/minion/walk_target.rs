use crate::game::minion::{MinionPath, MinionState, MinionTarget};
use bevy::prelude::*;

#[derive(Component, Default, Debug, Reflect)]
#[reflect(Component)]
pub struct WalkTargetTag;

#[derive(Bundle, Default)]
pub struct WalkTargetBundle {
    pub spatial: SpatialBundle,
    pub target_tag: MinionTarget,
    pub walk_tag: WalkTargetTag,
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
                // (_, Some(pt)) if pt.0.path.len() > 0 => Some(target), // keep as long theres a path?
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

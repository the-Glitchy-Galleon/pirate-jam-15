use bevy::prelude::*;

use super::MinionTarget;

#[derive(Bundle)]
pub struct WalkTargetBundle {
    pub spatial: SpatialBundle,
    pub target_tag: MinionTarget,
}
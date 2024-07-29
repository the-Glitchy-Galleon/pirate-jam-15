use crate::game::{
    collision_groups::TARGET_GROUP,
    minion::{MinionPath, MinionState, MinionTarget},
    objects::assets::GameObjectAssets,
};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct WalkTargetTag;

pub struct WalkTargetBuilder {
    position: Vec3,
}
impl WalkTargetBuilder {
    pub fn new(position: Vec3) -> Self {
        Self { position }
    }
    pub fn build(self, cmd: &mut Commands, assets: &GameObjectAssets) -> Entity {
        let root = (
            SpatialBundle {
                transform: Transform::from_translation(self.position),
                ..Default::default()
            },
            MinionTarget,
            WalkTargetTag,
            Collider::cuboid(2.0, 10.0, 2.0),
            CollisionGroups {
                memberships: TARGET_GROUP,
                filters: TARGET_GROUP,
            },
        );
        let flag_base = PbrBundle {
            mesh: assets.flag_meshes[0].clone(),
            material: assets.flag_materials[0].clone(),
            ..Default::default()
        };
        let flag = PbrBundle {
            mesh: assets.flag_meshes[1].clone(),
            material: assets.flag_materials[1].clone(),
            ..Default::default()
        };
        cmd.spawn(root)
            .with_children(|cmd| {
                cmd.spawn(flag_base);
                cmd.spawn(flag);
            })
            .id()
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

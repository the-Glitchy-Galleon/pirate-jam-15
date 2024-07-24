use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::game::{
    CharacterWalkControl, MinionBundle, MinionKind, MinionState, MinionStorage, MinionTarget, WalkTargetBundle
};

use super::PlayerTag;

#[derive(Clone, Copy, Debug, Component, Reflect)]
pub struct PlayerCollector;

#[derive(Clone, Copy, Debug, Resource, Reflect)]
pub struct MinionStorageInput {
    pub chosen_ty: MinionKind,
    pub want_to_throw: bool,
    pub to_where: Vec3,
    pub do_pickup: bool,
}

pub fn minion_storage_throw(
    mut min_inp: ResMut<MinionStorageInput>,
    mut player_q: Query<(&GlobalTransform, &mut MinionStorage)>,
    mut commands: Commands,
) {
    let Ok((tf, mut mins)) = player_q.get_single_mut() else {
        return;
    };

    if !min_inp.want_to_throw {
        return;
    }

    min_inp.want_to_throw = false;

    let ty = min_inp.chosen_ty;
    if !mins.extract_minion(ty) {
        return;
    }

    let target_id = commands
        .spawn(WalkTargetBundle {
            spatial: SpatialBundle {
                transform: Transform::from_translation(min_inp.to_where),
                ..default()
            },
            target_tag: MinionTarget,
        })
        .id();

    let minion_pos = tf.translation()
        + 2.0 * (min_inp.to_where - tf.translation()).normalize_or_zero()
        + 3.0 * Vec3::Y;

    commands.spawn(MinionBundle {
        spatial: SpatialBundle {
            transform: Transform::from_translation(minion_pos),
            ..default()
        },
        collider: Collider::cuboid(0.3, 0.3, 0.3),
        kind: min_inp.chosen_ty,
        state: MinionState::GoingTo(target_id),
        ..default()
    });
}

pub fn minion_storage_pickup(
    mut min_inp: ResMut<MinionStorageInput>,
    rap_ctx: ResMut<RapierContext>,
    dropped_mins: Query<(Entity, &MinionKind)>,
    mut collector: Query<(&mut Transform, &Children), With<PlayerCollector>>,
    mut player_q: Query<(&mut MinionStorage, &CharacterWalkControl), With<PlayerTag>>,
    mut commands: Commands,
) {
    let Ok((mut mins, walk)) = player_q.get_single_mut() else {
        return;
    };
    let Ok((mut coll_tf, children)) = collector.get_single_mut() else {
        return;
    };
    let Some(&collider) = children.first() else {
        return;
    };
    let angle = walk.direction.xz().to_angle();

    if angle.is_nan() {
        return;
    }

    *coll_tf = Transform::from_rotation(Quat::from_rotation_y(-angle));

    if !min_inp.do_pickup {
        return;
    }

    min_inp.do_pickup = false;

    for (min, ty) in dropped_mins.iter() {
        let result = rap_ctx.intersection_pair(min, collider);
        info!("{min:?} {collider:?}: {:?}", result);
        let Some(coll) = result else {
            continue;
        };

        if !coll {
            continue;
        }

        mins.add_minion(*ty);

        commands.entity(min).despawn_recursive();
    }
}

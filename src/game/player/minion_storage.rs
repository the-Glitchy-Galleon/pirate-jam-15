use crate::game::{
    minion::{
        collector::MinionStorage,
        minion_builder::{MinionAssets, MinionBuilder},
        walk_target::WalkTargetBundle,
    },
    CharacterWalkControl, MinionKind, MinionState,
};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use super::PlayerTag;

#[derive(Clone, Copy, Debug, Reflect)]
pub enum MinionThrowTarget {
    Location(Vec3),
    Ent(Entity),
}

#[derive(Clone, Copy, Debug, Component, Reflect)]
pub struct PlayerCollector;

#[derive(Clone, Copy, Debug, Resource, Reflect)]
pub struct MinionStorageInput {
    pub chosen_ty: MinionKind,
    pub want_to_throw: bool,
    pub to_where: MinionThrowTarget,
    pub do_pickup: bool,
}

pub fn minion_storage_throw(
    mut min_inp: ResMut<MinionStorageInput>,
    mut player_q: Query<(&GlobalTransform, &mut MinionStorage)>,
    mut commands: Commands,
    assets: Res<MinionAssets>,
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

    let target_id = match min_inp.to_where {
        MinionThrowTarget::Ent(e) => e,
        MinionThrowTarget::Location(pos) => commands
            .spawn(WalkTargetBundle {
                spatial: SpatialBundle {
                    transform: Transform::from_translation(pos),
                    ..default()
                },
                ..default()
            })
            .id(),
    };

    let minion_pos = tf.translation()
        - 2.0 * Vec3::X // TODO compute proper pos
        + 3.0 * Vec3::Y;

    MinionBuilder::new(
        min_inp.chosen_ty,
        minion_pos,
        MinionState::GoingTo(target_id),
    )
    .build(&mut commands, &assets);
}

#[derive(Component)]
pub struct MinionToWhereDebugUi;

pub fn debug_minion_to_where_ui(
    mut cmd: Commands,
    mut text: Query<&mut Text, With<MinionToWhereDebugUi>>,
    input: Res<MinionStorageInput>,
) {
    let Some(mut text) = text.iter_mut().last() else {
        cmd.spawn((
            TextBundle::from_sections([TextSection::new("", TextStyle::default())]).with_style(
                Style {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(5.0),
                    left: Val::Px(15.0),
                    ..default()
                },
            ),
            MinionToWhereDebugUi,
        ));
        return;
    };
    text.sections[0].value = format!("{:?}", input.to_where);
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

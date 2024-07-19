use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_rapier3d::prelude::*;
use bevy_rapier3d::rapier::pipeline::DebugColor;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[derive(Component, Reflect)]
pub enum MinionKind {
    Spoink,
    Doink,
    Woink,
}

#[derive(Component, Reflect)]
pub struct MinionStorage {
    storage: HashMap<MinionKind, u32>,
}

#[derive(Clone, Copy, Debug)]
#[derive(Resource, Reflect)]
pub struct MinionInput {
    pub chosen_ty: MinionKind,
    pub want_to_throw: bool,
    pub to_where: Vec3,
}

impl MinionStorage {
    pub fn new() -> Self {
        Self {
            storage: HashMap::new(),
        }
    }

    pub fn add_minion(&mut self, ty: MinionKind) {
        *self.storage.entry(ty)
            .or_default() += 1;
    }

    pub fn extract_minion(&mut self, ty: MinionKind) -> bool {
        let cnt = self.storage.entry(ty).or_default();

        if *cnt == 0 { return false; }

        *cnt -= 1;

        true
    }
}

pub fn player_minion(
    mut min_inp: ResMut<MinionInput>,
    mut player_q: Query<&mut MinionStorage>,
    mut commands: Commands,
) {
    let Ok(mut mins) = player_q.get_single_mut()
        else { return; };

    if !min_inp.want_to_throw {
        return;
    }

    min_inp.want_to_throw = false;

    let ty = min_inp.chosen_ty;
    if !mins.extract_minion(ty) {
        return;
    }

    commands.spawn((
        SpatialBundle {
            transform: Transform::from_translation(min_inp.to_where + 3.0 * Vec3::Y),
            ..default()
        },
        RigidBody::Dynamic,
        Collider::cuboid(0.3, 0.3, 0.3),
        ColliderDebugColor(Color::linear_rgb(0.0, 1.0, 0.0).into()),
        min_inp.chosen_ty,
    ));
}

pub fn player_minion_pickup(

) {

}
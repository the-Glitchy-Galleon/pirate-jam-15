use crate::game::objects::definitions::ColorDef;
use bevy::{color::palettes::tailwind, prelude::*};

#[derive(Component)]
pub struct ShowForwardGizmo;

pub fn show_forward_gizmo(
    forwarder: Query<(&Transform, &GlobalTransform), With<ShowForwardGizmo>>,
    mut gizmos: Gizmos,
) {
    for (tx, gx) in forwarder.iter() {
        let offset = tx.translation - gx.translation();
        gizmos.arrow(
            tx.translation + offset,
            tx.translation + offset + *tx.forward(),
            tailwind::BLUE_700,
        );
        gizmos.arrow(
            gx.translation(),
            gx.translation() + *gx.forward(),
            tailwind::CYAN_500,
        );
    }
}

#[derive(Component, Reflect)]
pub struct RootParent {
    parent: Entity,
}
impl RootParent {
    pub fn parent(&self) -> Entity {
        self.parent.clone()
    }
}

// Todo: this does not consider changes in hierarchy while the game is running
pub fn link_root_parents(
    mut cmd: Commands,
    entity: Query<Entity, Without<RootParent>>,
    hierarchy: Query<&Parent>,
) {
    for entity in entity.iter() {
        let mut current_root = entity;
        while let Ok(parent) = hierarchy.get(current_root) {
            current_root = parent.get();
        }
        cmd.entity(entity).insert(RootParent {
            parent: current_root,
        });
    }
}

#[derive(Component)]
pub struct Colored {
    color: ColorDef,
}

impl Colored {
    pub fn new(color: ColorDef) -> Self {
        Self { color }
    }
    pub fn color(&self) -> ColorDef {
        self.color
    }
}

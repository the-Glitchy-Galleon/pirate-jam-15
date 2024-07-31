use std::f32::EPSILON;

use bevy::prelude::*;

#[cfg(feature = "debug_visuals")]
use bevy::color::palettes::tailwind;

#[derive(Component)]
pub struct PrimaryCamera; // make sure there's only one in the scene

#[derive(Component)]
pub struct ShowForwardGizmo;

#[cfg(feature = "debug_visuals")]
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

pub fn approach_f32(origin: f32, target: f32, step: f32) -> f32 {
    let distance = (target - origin).abs();
    if step + f32::EPSILON >= distance - EPSILON {
        target
    } else {
        let direction = (target - origin).signum();
        origin + step * direction
    }
}

pub fn shoot_for_f32(origin: f32, target: f32, step: f32) -> f32 {
    let direction = (target - origin).signum();
    origin + step * direction
}

pub fn approach_vec3(origin: Vec3, target: Vec3, step: f32) -> Vec3 {
    let distance = origin.distance(target);
    if step + f32::EPSILON >= distance - f32::EPSILON {
        target
    } else {
        let direction = (target - origin).signum();
        origin + step * direction
    }
}

pub fn shoot_for_vec3(origin: Vec3, target: Vec3, step: f32) -> Vec3 {
    let direction = (target - origin).signum();
    origin + step * direction
}

pub fn approach_angle(origin: f32, target: f32, step: f32) -> f32 {
    let diff = normalized_angle(target - origin);
    let direction = diff.signum();
    let distance = diff.abs();

    if step + f32::EPSILON >= distance - f32::EPSILON {
        target
    } else {
        origin + step * direction
    }
}

pub fn angle_distance(a: f32, b: f32) -> f32 {
    normalized_angle(a - b).abs()
}

pub fn normalized_angle(mut angle: f32) -> f32 {
    use std::f32::consts::{PI, TAU};
    angle %= TAU;
    if angle > PI {
        angle -= TAU;
    } else if angle < -PI {
        angle += TAU;
    }
    angle
}

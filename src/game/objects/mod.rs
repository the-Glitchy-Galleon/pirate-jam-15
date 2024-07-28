use crate::game::objects::{
    assets::GameObjectAssets,
    camera::CameraObjBuilder,
    cauldron::CauldronBuilder,
    definitions::{ObjectDef, ObjectDefKind},
    destructible_target_test::DestructibleTargetTestBuilder,
    physics_cubes_test::PhysicsCubeTestBuilder,
};
use bevy::prelude::*;

pub mod assets;
pub mod camera;
pub mod cauldron;
pub mod definitions;
pub mod destructible_target_test;
pub mod physics_cubes_test;

pub fn spawn_object(
    mut cmd: &mut Commands,
    object: &ObjectDef,
    assets: &GameObjectAssets,
) -> Entity {
    match object.kind {
        ObjectDefKind::Cauldron => {
            let builder = CauldronBuilder(object);
            builder.build(&mut cmd, &assets)
        }
        ObjectDefKind::Camera => {
            let builder = CameraObjBuilder(object);
            builder.build(&mut cmd, &assets)
        }
        ObjectDefKind::DestructibleTargetTest => {
            let builder = DestructibleTargetTestBuilder(object);
            builder.build(&mut cmd, &assets)
        }
        ObjectDefKind::PhysicsCubesTest => {
            let builder = PhysicsCubeTestBuilder(object);
            builder.build(&mut cmd, &assets)
        }
        _ => cmd
            .spawn(PbrBundle {
                mesh: assets.dummy_cube_mesh.clone(),
                material: assets.dummy_cube_material(object.color),
                transform: Transform::IDENTITY
                    .with_translation(object.position + Vec3::Y * 0.5)
                    .with_rotation(Quat::from_rotation_y(object.rotation))
                    .with_scale(Vec3::splat(0.4)),
                ..Default::default()
            })
            .id(),
    }
}

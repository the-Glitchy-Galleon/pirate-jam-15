//! Example to demonstrate simple usage of minion requirement

use crate::game::objects::{assets::GameObjectAssets, definitions::ObjectDef};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub struct PhysicsCubeTestBuilder(pub ObjectDef);

impl PhysicsCubeTestBuilder {
    pub fn build(self, cmd: &mut Commands, _assets: &GameObjectAssets) -> Entity {
        /*
         * Create the cubes
         */
        let num = 2;
        let rad = 1.0;

        let shift = rad * 2.0 + rad;
        let centerx = shift * (num / 2) as f32;
        let centery = shift / 2.0;
        let centerz = shift * (num / 2) as f32;

        let mut offset = -(num as f32) * (rad * 2.0 + rad) * 0.5;
        let mut color = 0;
        let colors = [
            Hsla::hsl(220.0, 1.0, 0.3),
            Hsla::hsl(180.0, 1.0, 0.3),
            Hsla::hsl(260.0, 1.0, 0.7),
        ];
        let mut root = cmd.spawn(SpatialBundle::default());

        root.with_children(|cmd| {
            for j in 0usize..2 {
                for i in 0..num {
                    for k in 0usize..num {
                        let pos = Vec3::new(
                            i as f32 * shift - centerx + offset,
                            j as f32 * shift + centery + 3.0,
                            k as f32 * shift - centerz + offset,
                        );
                        color += 1;

                        cmd.spawn((
                            TransformBundle::from(Transform::from_translation(
                                pos + self.0.position,
                            )),
                            RigidBody::Dynamic,
                            Collider::cuboid(rad, rad, rad),
                            ColliderDebugColor(colors[color % 3]),
                        ));
                    }
                }

                offset -= 0.05 * rad * (num as f32 - 1.0);
            }
        });
        root.id()
    }
}

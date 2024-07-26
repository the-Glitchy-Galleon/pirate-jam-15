use crate::framework::easing::TweenBackAndForth;

use super::{assets::GameObjectAssets, definitions::*};
use bevy::{color::palettes::tailwind, prelude::*, time::Real};
use helpers::*;
use std::{f32::consts::FRAC_PI_2, ops::RangeInclusive};

const NUM_SPOTLIGHT_RAYS: usize = 16;
const ANGLE_SIZE: f32 = 0.4;
const ANGLE_RANGE: RangeInclusive<f32> = 0.3..=0.6; // range that kinda works out

pub struct CameraObjBuilder(pub ObjectDef);

impl CameraObjBuilder {
    pub fn build(self, cmd: &mut Commands, assets: &GameObjectAssets) -> Entity {
        let position = self.0.position + Vec3::Y * 3.0;
        let spotlight_position = position + Vec3::new(0.0, 0.1, -0.5);

        let mut max_range = 0.0;
        for pos in &self.0.pos_refs {
            max_range = f32::max(max_range, pos.distance(spotlight_position));
        }

        let angle = FRAC_PI_2 * f32::clamp(ANGLE_SIZE, *ANGLE_RANGE.start(), *ANGLE_RANGE.end());

        let root = (
            Name::new(format!(
                "{} Camera at {{{}}}",
                self.0.color.as_str(),
                self.0.position
            )),
            SpatialBundle {
                transform: Transform::IDENTITY
                    .with_translation(position)
                    .with_rotation(Quat::from_rotation_y(self.0.rotation)),
                ..Default::default()
            },
            CameraPathState::new(self.0.pos_refs.clone(), position),
            ShowForwardGizmo,
        );
        let spotlight2 = (SpotLightBundle {
            transform: Transform::from_translation(spotlight_position - position),
            spot_light: SpotLight {
                intensity: 5_000_000.0, // lumens? but it doesn't do much with reasonable values
                color: self.spotlight_color(),
                shadows_enabled: true,
                range: 100.0, // ignore calculated range because it doesn't really reach
                inner_angle: angle,
                outer_angle: angle * 0.9,
                ..default()
            },
            ..Default::default()
        },);
        let wall_mount = (MaterialMeshBundle::<StandardMaterial> {
            mesh: assets.camera_wall_mount.clone(),
            material: assets.camera_material.clone(),
            ..Default::default()
        },);
        let rotating_mesh = (
            MaterialMeshBundle::<StandardMaterial> {
                mesh: assets.camera_rotating_mesh.clone(),
                material: assets.camera_material.clone(),
                ..Default::default()
            },
            LookAtPathState,
        );

        cmd.spawn(root)
            .with_children(|cmd| {
                cmd.spawn(wall_mount).with_children(|cmd| {
                    cmd.spawn(rotating_mesh).with_children(|cmd| {
                        cmd.spawn(spotlight2);
                    });
                });
            })
            .id()
    }

    #[rustfmt::skip]
    fn spotlight_color(&self) -> Color {
        match self.0.color {
            ColorDef::Void    => tailwind::GRAY_500,
            ColorDef::Red     => tailwind::RED_500,
            ColorDef::Green   => tailwind::GREEN_500,
            ColorDef::Blue    => tailwind::BLUE_500,
            ColorDef::Yellow  => tailwind::YELLOW_500,
            ColorDef::Magenta => tailwind::PURPLE_500,
            ColorDef::Cyan    => tailwind::CYAN_500,
            ColorDef::White   => tailwind::GREEN_100,
        }.into()
    }
}

pub fn add_systems_and_resources(app: &mut App) {
    app.add_event::<SpotlightHitEvent>();
    app.add_systems(PreUpdate, link_root_parents);
    app.add_systems(
        Update,
        (
            show_forward_gizmo,
            update_path_state,
            draw_path_state_gizmo.after(update_path_state),
            follow_path_state.after(update_path_state),
            look_at_path_state.after(update_path_state),
            cast_spotlight_rays.after(look_at_path_state),
        ),
    );
}

#[derive(Component, Reflect)]
pub struct RootParent {
    entity: Entity,
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
            entity: current_root,
        });
    }
}

#[derive(Component, Reflect)]
pub struct CameraPathState {
    path: TweenBackAndForth,
    root_position: Vec3,
    position: Vec3,
}

impl CameraPathState {
    pub fn new(paths: Vec<Vec3>, root_position: Vec3) -> Self {
        Self {
            path: TweenBackAndForth::new(paths),
            root_position,
            position: Vec3::ZERO,
        }
    }
}

pub fn update_path_state(mut state: Query<&mut CameraPathState>, time: Res<Time<Real>>) {
    for mut state in state.iter_mut() {
        state.position = state.path.tick(time.delta_seconds());
    }
}

pub fn draw_path_state_gizmo(state: Query<&CameraPathState>, mut gizmos: Gizmos) {
    for state in state.iter() {
        gizmos.cuboid(
            Transform::from_translation(state.position).with_scale(Vec3::splat(0.1)),
            tailwind::BLUE_700,
        );
    }
}

#[derive(Component, Reflect)]
pub struct FollowPathState;

pub fn follow_path_state(
    state: Query<&CameraPathState>,
    mut follower: Query<(&RootParent, &mut Transform, &GlobalTransform), With<FollowPathState>>,
) {
    for (root, mut tx, gx) in follower.iter_mut() {
        if let Ok(state) = state.get(root.entity) {
            let offset = tx.translation - gx.translation();
            tx.translation = state.position + offset;
        }
    }
}

#[derive(Component, Reflect)]
pub struct LookAtPathState;

pub fn look_at_path_state(
    state: Query<&CameraPathState>,
    mut looker: Query<(&RootParent, &mut Transform, &GlobalTransform), With<LookAtPathState>>,
) {
    for (root, mut tx, gx) in looker.iter_mut() {
        if let Ok(state) = state.get(root.entity) {
            let transform = compute_parent_transform(gx, &tx);
            let local_point = transform.transform_point(state.position);
            tx.look_at(local_point, Vec3::Y);
        }
    }
}

fn compute_parent_transform(
    global_transform: &GlobalTransform,
    transform: &Transform,
) -> Transform {
    Transform::from_matrix(transform.compute_matrix() * global_transform.compute_matrix().inverse())
}

#[derive(Event)]
pub struct SpotlightHitEvent;

// A cone collider would be better, but i gave up setting it up correctly for now...
pub fn cast_spotlight_rays(
    spotlight: Query<(&Transform, &GlobalTransform, &SpotLight)>,
    mut _evs: EventWriter<SpotlightHitEvent>,
    mut gizmos: Gizmos,
) {
    for (_tx, gx, spotlight) in spotlight.iter() {
        let rays = generate_rays::<NUM_SPOTLIGHT_RAYS>(
            gx.translation(),
            *gx.forward(),
            spotlight.outer_angle,
        );
        for ray in rays {
            let dir = Vec3::new(ray.dir.x, ray.dir.y, ray.dir.z);

            gizmos.ray(
                ray.origin.into(),
                dir * 10.0,
                tailwind::AMBER_200.with_alpha(0.2),
            );
        }
    }
}

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
            tx.translation + offset + tx.forward() * 2.0,
            tailwind::BLUE_700,
        );
        gizmos.arrow(
            gx.translation(),
            gx.translation() + gx.forward() * 2.0,
            tailwind::CYAN_500,
        );
    }
}

mod helpers {
    use bevy::prelude::*;
    use bevy_rapier3d::rapier::prelude::Ray;
    use std::f32::consts::PI;

    pub const fn gen_range_usize<const N: usize>() -> [usize; N] {
        let mut res = [0; N];
        let mut i = 0;
        while i < N {
            res[i] = i;
            i += 1;
        }
        res
    }

    pub fn generate_rays<const N: usize>(
        position: Vec3,
        direction: Vec3,
        half_angle: f32,
    ) -> [Ray; N] {
        let normal = direction.normalize();
        let angle_step = (2.0 * PI) / N as f32;

        let ortho_vector1 = if normal.x.abs() > normal.z.abs() {
            Vec3::new(-normal.y, normal.x, 0.0).normalize()
        } else {
            Vec3::new(0.0, -normal.z, normal.y).normalize()
        };

        let ortho_vector2 = normal.cross(ortho_vector1).normalize();

        gen_range_usize().map(|i| {
            let theta = angle_step * i as f32;
            let sin_half_angle = half_angle.sin();
            let cone_direction = normal * half_angle.cos()
                + ortho_vector1 * sin_half_angle * theta.cos()
                + ortho_vector2 * sin_half_angle * theta.sin();
            Ray::new(position.into(), cone_direction.normalize().into())
        })
    }
}

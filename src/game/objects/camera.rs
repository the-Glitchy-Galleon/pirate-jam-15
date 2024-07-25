use crate::framework::easing::TweenBackAndForth;

use super::{assets::GameObjectAssets, definitions::*};
use bevy::{color::palettes::tailwind, prelude::*, time::Real};
use helpers::*;
use std::f32::consts::FRAC_PI_2;

const NUM_SPOTLIGHT_RAYS: usize = 16;

pub struct CameraObjBuilder(pub ObjectDef);

impl CameraObjBuilder {
    pub fn build(self, cmd: &mut Commands, assets: &GameObjectAssets) -> Entity {
        let position = self.0.position + Vec3::Y * 3.0;
        let spotlight_position = position + Vec3::new(0.25, 0.25, 0.0);

        let mut max_range = 0.0;
        for pos in &self.0.pos_refs {
            max_range = f32::max(max_range, pos.distance(spotlight_position));
        }

        let angle = FRAC_PI_2 * 0.4;

        // range that kinda works out
        let angle = f32::clamp(angle, FRAC_PI_2 * 0.3, FRAC_PI_2 * 0.6);

        let root = (
            Name::new(format!(
                "{} Camera at {{{}}}",
                self.0.color.as_str(),
                self.0.position
            )),
            SpatialBundle {
                transform: Transform::IDENTITY.with_translation(position),
                ..Default::default()
            },
            CameraPathState::new(self.0.pos_refs.clone(), position),
            ShowForwardGizmo,
        );
        let spotlight2 = (
            SpotLightBundle {
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
            },
            LookAtPathState,
        );
        let mesh = (
            MaterialMeshBundle::<StandardMaterial> {
                mesh: assets.camera_mesh.clone(),
                material: assets.camera_material.clone(),
                transform: Transform::IDENTITY
                    .with_rotation(Quat::from_rotation_y(self.0.rotation))
                    .with_translation(Vec3::new(0.0, 0.25, 0.3)),
                ..Default::default()
            },
            LookAtPathState,
        );

        cmd.spawn(root)
            .with_children(|cmd| {
                cmd.spawn(mesh);
                // cmd.spawn(capsule);
                // cmd.spawn(spotlight);
                cmd.spawn(spotlight2);
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
    parent: Query<&CameraPathState>,
    mut follower: Query<(&Parent, &mut Transform, &GlobalTransform), With<FollowPathState>>,
) {
    for (pid, mut tx, gx) in follower.iter_mut() {
        if let Ok(state) = parent.get(pid.get()) {
            let offset = tx.translation - gx.translation();
            tx.translation = state.position + offset;
        }
    }
}

#[derive(Component, Reflect)]
pub struct LookAtPathState;

pub fn look_at_path_state(
    parent: Query<&CameraPathState>,
    mut looker: Query<(&Parent, &mut Transform, &GlobalTransform), With<LookAtPathState>>,
) {
    for (pid, mut tx, gx) in looker.iter_mut() {
        if let Ok(state) = parent.get(pid.get()) {
            let offset = tx.translation - gx.translation();
            tx.look_at(state.position + offset, Vec3::Y);
        }
    }
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

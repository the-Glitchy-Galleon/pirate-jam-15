use super::{assets::GameObjectAssets, definitions::*};
use crate::{framework::easing::*, game::collision_groups::*};
use bevy::{color::palettes::tailwind, prelude::*, time::Real};
use bevy_rapier3d::prelude::*;
use std::{f32::consts::FRAC_PI_2, ops::RangeInclusive};

const _NUM_SPOTLIGHT_RAYS: usize = 16;
const SPOTLIGHT_ANGLE: f32 = 0.4;
const SPOTLIGHT_ANGLE_RANGE: RangeInclusive<f32> = 0.3..=0.6; // range that kinda works out for the angle
const CONE_RADIUS_FACTOR: f32 = 0.8;
const CONE_RANGE_FACTOR: f32 = 1.2;
const RAY_RANGE_FACTOR: f32 = 1.2;

pub const CHARGE_DURATION_SECS: f32 = 1.0;
pub const BEAM_DURATION_SECS: f32 = 1.0;

// const COLLISION_GROUP_WALL: Group = Group::GROUP_15;

pub struct CameraObjBuilder(pub ObjectDef);

impl CameraObjBuilder {
    pub fn build(self, cmd: &mut Commands, assets: &GameObjectAssets) -> Entity {
        let position = self.0.position + Vec3::Y * 3.0;
        let spotlight_position = position + Vec3::new(0.0, 0.1, -0.5);

        let mut max_range = 0.0;
        for pos in &self.0.pos_refs {
            max_range = f32::max(max_range, pos.distance(spotlight_position));
        }

        let angle = FRAC_PI_2
            * f32::clamp(
                SPOTLIGHT_ANGLE,
                *SPOTLIGHT_ANGLE_RANGE.start(),
                *SPOTLIGHT_ANGLE_RANGE.end(),
            );
        let cone_radius = (max_range * angle.tan()) * CONE_RADIUS_FACTOR;
        let cone_range = max_range * CONE_RANGE_FACTOR;

        info!("Radius: {cone_radius}");

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
            CameraPhase::Pathing,
            CameraPathState::new(self.0.pos_refs.clone(), position),
            ShinedEntityList::default(),
            ShowForwardGizmo,
        );
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
        let spotlight = (SpotLightBundle {
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
        let cone = (
            SpatialBundle {
                transform: Transform::from_rotation(Quat::from_rotation_x(FRAC_PI_2))
                    .with_translation(Vec3::NEG_Z * cone_range * 0.5),
                ..Default::default()
            },
            Sensor,
            Collider::cone(cone_range * 0.5, cone_radius),
            CollisionGroups {
                memberships: G_SENSOR,
                filters: G_ALL,
            },
            ActiveCollisionTypes::STATIC_STATIC,
            ShineCone {
                max_range: cone_range * RAY_RANGE_FACTOR,
            },
        );

        cmd.spawn(root)
            .with_children(|cmd| {
                cmd.spawn(wall_mount).with_children(|cmd| {
                    cmd.spawn(rotating_mesh).with_children(|cmd| {
                        cmd.spawn(spotlight).with_children(|cmd| {
                            cmd.spawn(cone);
                        });
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
            ColorDef::White   => tailwind::GRAY_100,
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
            update_shined_entities,
            update_phase.after(update_shined_entities),
            process_spotlight_hit.after(update_phase),
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
pub struct Shineable;

#[derive(Component, Reflect)]
pub struct ShineCone {
    max_range: f32,
}

#[derive(Component, Default, Reflect)]
pub struct ShinedEntityList {
    entities: Vec<Entity>,
}

pub fn update_shined_entities(
    rapier: Res<RapierContext>,
    mut state: Query<&mut ShinedEntityList>,
    // mut reader: EventReader<CollisionEvent>,
    cone: Query<
        (
            Entity,
            &ShineCone,
            &RootParent,
            &Transform,
            &GlobalTransform,
        ),
        Without<Shineable>,
    >,
    shineable: Query<(Entity, &GlobalTransform), With<Shineable>>,
    mut gizmos: Gizmos,
) {
    for (cone_ent, cone, root, tx, gx) in cone.iter() {
        let Ok(mut state) = state.get_mut(root.entity) else {
            continue;
        };
        let origin =
            Transform::from_matrix(gx.compute_matrix() * tx.compute_matrix().inverse()).translation;

        let all_ents_in_cone = rapier
            .intersection_pairs_with(cone_ent)
            .filter_map(|(a, b, x)| {
                x.then_some(if cone_ent == a {
                    b
                } else if cone_ent == b {
                    a
                } else {
                    None?
                })
            })
            .collect::<Vec<_>>();

        state.entities.clear();
        for (shineable, sgx) in shineable.iter() {
            if !all_ents_in_cone.contains(&shineable) {
                continue;
            }

            let destination = sgx.translation();
            let direction = (destination - origin).normalize();
            let distance = origin.distance(destination);

            let raycast = rapier.cast_ray(
                origin.into(),
                direction.into(),
                cone.max_range.into(),
                true,
                QueryFilter {
                    groups: Some(CollisionGroups {
                        memberships: G_SENSOR,
                        filters: G_GROUND | G_WALL | G_PLAYER | G_MINION,
                    }),
                    ..Default::default()
                },
            );

            let hit = match raycast {
                Some((ent, _toi)) if ent == shineable => true,
                _ => false,
            };

            let color = match hit {
                true => tailwind::ORANGE_100,
                false => tailwind::ORANGE_900,
            };
            gizmos.ray(
                origin.into(),
                direction * f32::min(cone.max_range, distance),
                color,
            );
            if hit {
                state.entities.push(shineable);
            }
        }
    }
}

#[derive(Component, Reflect, Clone, PartialEq)]
pub enum CameraPhase {
    Pathing,
    Charging(f32),
    Cooldown(f32),
}

pub fn update_phase(
    mut phase: Query<(&mut CameraPhase, &ShinedEntityList)>,
    time: Res<Time<Real>>,
    mut hit: EventWriter<SpotlightHitEvent>,
) {
    for (mut phase, shined) in phase.iter_mut() {
        match phase.as_mut() {
            CameraPhase::Pathing => {
                if shined.entities.len() > 0 {
                    *phase = CameraPhase::Charging(CHARGE_DURATION_SECS)
                }
            }
            CameraPhase::Charging(t) => {
                *t -= time.delta_seconds();
                if *t <= 0.0 {
                    *phase = CameraPhase::Cooldown(BEAM_DURATION_SECS);
                    for entity in shined.entities.iter() {
                        hit.send(SpotlightHitEvent { target: *entity });
                    }
                }
            }
            CameraPhase::Cooldown(t) => {
                *t -= time.delta_seconds();
                if *t <= 0.0 {
                    *phase = CameraPhase::Pathing;
                }
            }
        }
    }
}

pub fn process_spotlight_hit(mut hit: EventReader<SpotlightHitEvent>, name: Query<&Name>) {
    for hit in hit.read() {
        let name = match name.get(hit.target) {
            Ok(name) => name.to_string(),
            _ => format!("{:?}", hit.target),
        };
        info!("Hit entity: {name}");
    }
}

#[derive(Component, Reflect)]
pub struct CameraPathState {
    path: TweenList,
    root_position: Vec3,
    position: Vec3,
}

impl CameraPathState {
    pub fn new(paths: Vec<Vec3>, root_position: Vec3) -> Self {
        Self {
            path: TweenList::new(paths, Easing::InPowf(2.0)),
            root_position,
            position: Vec3::ZERO,
        }
    }
}

pub fn update_path_state(
    mut state: Query<(&mut CameraPathState, &CameraPhase)>,
    time: Res<Time<Real>>,
) {
    for (mut state, phase) in state.iter_mut() {
        if *phase == CameraPhase::Pathing {
            state.position = state.path.tick(time.delta_seconds());
        }
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
pub struct SpotlightHitEvent {
    pub target: Entity,
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

mod helpers {
    use bevy::prelude::*;
    use bevy_rapier3d::rapier::prelude::Ray;
    use std::f32::consts::PI;

    pub const fn _gen_range_usize<const N: usize>() -> [usize; N] {
        let mut res = [0; N];
        let mut i = 0;
        while i < N {
            res[i] = i;
            i += 1;
        }
        res
    }

    ///
    pub fn _generate_conical_rays<const N: usize>(
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

        _gen_range_usize().map(|i| {
            let theta = angle_step * i as f32;
            let sin_half_angle = half_angle.sin();
            let cone_direction = normal * half_angle.cos()
                + ortho_vector1 * sin_half_angle * theta.cos()
                + ortho_vector2 * sin_half_angle * theta.sin();
            Ray::new(position.into(), cone_direction.normalize().into())
        })
    }
}

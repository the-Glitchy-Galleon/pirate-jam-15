use crate::{
    framework::easing::{Easing, TweenList},
    game::{
        collision_groups::{ACTOR_GROUP, GROUND_GROUP, WALL_GROUP},
        common::{Colored, RootParent, ShowForwardGizmo},
        objects::{
            assets::GameObjectAssets,
            definitions::{ColorDef, ObjectDef},
        },
        player::{AddPlayerRespawnEvent, PlayerTag},
        LevelResources,
    },
};
use bevy::{color::palettes::tailwind, prelude::*, time::Real};
use bevy_rapier3d::prelude::*;
use std::{
    f32::consts::{FRAC_PI_2, PI, TAU},
    ops::RangeInclusive,
};
use util::is_within_cone_shape;

pub const SPOTLIGHT_ANGLE: f32 = 0.4;
pub const SPOTLIGHT_ANGLE_RANGE: RangeInclusive<f32> = 0.3..=0.6; // range that kinda works out for the angle
pub const CONE_DETECTION_RADIUS_FACTOR: f32 = 0.9;

pub const CHARGE_DURATION_SECS: f32 = 1.5;
pub const BEAM_DURATION_SECS: f32 = 1.0;

pub struct CameraObjBuilder<'a>(pub &'a ObjectDef);

impl CameraObjBuilder<'_> {
    pub fn build(self, cmd: &mut Commands, assets: &GameObjectAssets) -> Entity {
        let position = self.0.position + Vec3::Y * 3.0;
        let spotlight_position = position + Vec3::new(0.0, 0.1, -0.5);

        let half_angle = FRAC_PI_2
            * f32::clamp(
                SPOTLIGHT_ANGLE,
                *SPOTLIGHT_ANGLE_RANGE.start(),
                *SPOTLIGHT_ANGLE_RANGE.end(),
            );

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
            Colored::new(self.0.color),
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
                color: spotlight_color(self.0.color).into(),
                shadows_enabled: true,
                range: 100.0, // ignore calculated range because it doesn't really reach
                inner_angle: half_angle,
                outer_angle: half_angle * 0.9,
                ..default()
            },
            ..Default::default()
        },);
        let cone = (
            SpatialBundle::default(),
            ShineCone {
                half_angle: half_angle * CONE_DETECTION_RADIUS_FACTOR,
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
}

#[rustfmt::skip]
const fn spotlight_color(color: ColorDef) -> Srgba {
    match color {
        ColorDef::Void    => tailwind::GRAY_100,
        ColorDef::Red     => tailwind::RED_500,
        ColorDef::Green   => tailwind::GREEN_500,
        ColorDef::Blue    => tailwind::BLUE_500,
        ColorDef::Yellow  => tailwind::YELLOW_500,
        ColorDef::Magenta => tailwind::PURPLE_500,
        ColorDef::Cyan    => tailwind::CYAN_500,
        ColorDef::White   => tailwind::GRAY_900,
    }
}

pub fn add_systems_and_resources(app: &mut App) {
    app.add_event::<SpotlightHitEvent>();
    app.add_systems(
        Update,
        (
            update_path_state,
            draw_path_state_gizmo.after(update_path_state),
            follow_path_state.after(update_path_state),
            look_at_path_state.after(update_path_state),
            update_shined_entities,
            update_phase.after(update_shined_entities),
            camera_charge_effect.after(update_phase),
            spotlight_hit_player.after(update_phase),
        ),
    );
}

#[derive(Component, Reflect)]
pub struct Shineable;

#[derive(Component, Reflect)]
pub struct ShineCone {
    half_angle: f32,
}

#[derive(Component, Default, Reflect)]
struct ShinedEntityList {
    entities: Vec<Entity>,
}

fn update_shined_entities(
    rapier: Res<RapierContext>,
    mut state: Query<&mut ShinedEntityList>,
    // mut reader: EventReader<CollisionEvent>,
    cone: Query<(&ShineCone, &RootParent, &Transform, &GlobalTransform), Without<Shineable>>,
    shineable: Query<(Entity, &GlobalTransform), With<Shineable>>,
    mut gizmos: Gizmos,
) {
    for (cone, root, tx, gx) in cone.iter() {
        let Ok(mut state) = state.get_mut(root.parent()) else {
            continue;
        };
        let origin =
            Transform::from_matrix(gx.compute_matrix() * tx.compute_matrix().inverse()).translation;

        state.entities.clear();
        for (shineable, sgx) in shineable.iter() {
            let destination = sgx.translation();
            let direction = (destination - origin).normalize();
            let distance = origin.distance(destination);

            if !is_within_cone_shape(direction, *gx.forward(), cone.half_angle) {
                continue;
            }

            let raycast = rapier.cast_ray(
                origin.into(),
                direction.into(),
                bevy_rapier3d::math::Real::INFINITY,
                true,
                QueryFilter {
                    groups: Some(CollisionGroups::new(
                        Group::all(),
                        GROUND_GROUP | WALL_GROUP | ACTOR_GROUP | ACTOR_GROUP,
                    )),
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
            gizmos.ray(origin.into(), direction * distance, color);
            if hit {
                state.entities.push(shineable);
            }
        }
    }
}

#[derive(Component, Reflect, Clone, PartialEq)]
enum CameraPhase {
    Pathing,
    Charging(f32),
    Cooldown(f32),
}

fn update_phase(
    mut cmd: Commands,
    mut phase: Query<(Entity, &mut CameraPhase, &ShinedEntityList, &Colored)>,
    cone: Query<(Entity, &RootParent), With<ShineCone>>,
    time: Res<Time<Real>>,
    mut hit: EventWriter<SpotlightHitEvent>,
) {
    for (ent, mut phase, shined, colored) in phase.iter_mut() {
        match phase.as_mut() {
            CameraPhase::Pathing => {
                if shined.entities.len() > 0 {
                    *phase = CameraPhase::Charging(CHARGE_DURATION_SECS);
                    for (cone, root) in cone.iter() {
                        if root.parent() == ent {
                            cmd.entity(cone).insert(CameraChargeEffect {
                                timer: Timer::from_seconds(CHARGE_DURATION_SECS, TimerMode::Once),
                                color: colored.color(),
                            });
                            break;
                        }
                    }
                }
            }
            CameraPhase::Charging(t) => {
                *t -= time.delta_seconds();
                if *t <= 0.0 {
                    *phase = CameraPhase::Cooldown(BEAM_DURATION_SECS);
                    for entity in shined.entities.iter() {
                        hit.send(SpotlightHitEvent {
                            source: ent,
                            target: *entity,
                            color: colored.color(),
                        });
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

#[derive(Component)]
struct CameraChargeEffect {
    timer: Timer,
    color: ColorDef,
}

fn camera_charge_effect(
    mut cmd: Commands,
    mut charge: Query<(
        Entity,
        &mut CameraChargeEffect,
        &ShineCone,
        &GlobalTransform,
    )>,
    mut gizmos: Gizmos,
    time: Res<Time<Real>>,
) {
    for (ent, mut charge, cone, gx) in charge.iter_mut() {
        charge.timer.tick(time.delta());
        let alpha = 0.6 + (0.4 * (PI / charge.timer.fraction()).sin());

        let rays = util::generate_conical_rays::<16>(
            gx.translation(),
            *gx.forward(),
            cone.half_angle,
            (charge.timer.fraction()) / TAU,
        );
        for ray in rays {
            gizmos.ray(
                ray.origin.into(),
                (ray.dir * 10.0).into(),
                spotlight_color(charge.color).with_alpha(alpha),
            );
        }
        if charge.timer.finished() {
            cmd.entity(ent).remove::<CameraChargeEffect>();
        }
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

fn update_path_state(
    mut state: Query<(&mut CameraPathState, &CameraPhase)>,
    time: Res<Time<Real>>,
) {
    for (mut state, phase) in state.iter_mut() {
        if *phase == CameraPhase::Pathing {
            state.position = state.path.tick(time.delta_seconds());
        }
    }
}

fn draw_path_state_gizmo(state: Query<&CameraPathState>, mut gizmos: Gizmos) {
    for state in state.iter() {
        gizmos.cuboid(
            Transform::from_translation(state.position).with_scale(Vec3::splat(0.1)),
            tailwind::BLUE_700,
        );
    }
}

#[derive(Component, Reflect)]
pub struct FollowPathState;

fn follow_path_state(
    state: Query<&CameraPathState>,
    mut follower: Query<(&RootParent, &mut Transform, &GlobalTransform), With<FollowPathState>>,
) {
    for (root, mut tx, gx) in follower.iter_mut() {
        if let Ok(state) = state.get(root.parent()) {
            let offset = tx.translation - gx.translation();
            tx.translation = state.position + offset;
        }
    }
}

#[derive(Component, Reflect)]
pub struct LookAtPathState;

fn look_at_path_state(
    state: Query<&CameraPathState>,
    mut looker: Query<(&RootParent, &mut Transform, &GlobalTransform), With<LookAtPathState>>,
) {
    for (root, mut tx, gx) in looker.iter_mut() {
        if let Ok(state) = state.get(root.parent()) {
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
    pub source: Entity,
    pub target: Entity,
    pub color: ColorDef,
}

fn spotlight_hit_player(
    mut hit: EventReader<SpotlightHitEvent>,
    player: Query<Entity, With<PlayerTag>>,
    level: Res<LevelResources>,
    mut respawn: EventWriter<AddPlayerRespawnEvent>,
) {
    for hit in hit.read() {
        let Ok(_) = player.get(hit.target) else {
            continue;
        };
        let Some(spawnpoints) = &level.spawnpoints else {
            continue;
        };

        let highest_respawn_pos = spawnpoints
            .iter()
            .filter(|o| o.2)
            .max_by(|a, b| a.1.cmp(&b.1))
            .map(|o| o.0)
            .unwrap_or(Vec3::ZERO + Vec3::Y * 7.0);

        respawn.send(AddPlayerRespawnEvent {
            position: highest_respawn_pos,
        });
    }
}

mod util {
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

    pub fn generate_conical_rays<const N: usize>(
        position: Vec3,
        direction: Vec3,
        half_angle: f32,
        rotation: f32,
    ) -> [Ray; N] {
        let normal = direction.normalize();
        let angle_step = (2.0 * PI) / N as f32 + rotation;

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

    pub fn is_within_cone_shape(direction: Vec3, cone_normal: Vec3, cone_half_angle: f32) -> bool {
        cone_normal.dot(direction) >= cone_half_angle.cos()
    }
}

use crate::game::{
    collision_groups::{ACTOR_GROUP, GROUND_GROUP, TARGET_GROUP},
    common::Colored,
    minion::{
        minion_builder::{MinionAssets, MinionBuilder},
        MinionKind, MinionPath, MinionStartedInteraction, MinionState, MinionTarget,
    },
    objects::{
        assets::GameObjectAssets,
        definitions::{ColorDef, ObjectDef},
    },
};
use bevy::{color::palettes::tailwind, prelude::*, time::Real};
use bevy_rapier3d::prelude::*;
use std::{collections::VecDeque, f32::consts::TAU};

pub struct CauldronBuilder<'a>(pub &'a ObjectDef);

impl CauldronBuilder<'_> {
    pub fn build(self, cmd: &mut Commands, assets: &GameObjectAssets) -> Entity {
        let root = (
            SpatialBundle {
                transform: Transform::IDENTITY
                    .with_translation(self.0.position)
                    .with_rotation(Quat::from_rotation_y(self.0.rotation))
                    .with_scale(Vec3::splat(0.7)), // Todo: increase minion interaction range
                ..Default::default()
            },
            MinionTarget,
            CauldronTag,
            CauldronQueue::default(),
            Colored::new(self.0.color),
            Collider::cylinder(1.0, 1.5),
            CollisionGroups::new(TARGET_GROUP, GROUND_GROUP | ACTOR_GROUP),
        );
        let base = (
            SpatialBundle::default(),
            assets.cauldron_mesh.clone(),
            assets.cauldron_material.clone(),
        );
        let fluid = (
            SpatialBundle::default(),
            assets.cauldron_fluid_mesh.clone(),
            assets.cauldron_fluid_material(self.0.color),
        );

        cmd.spawn(root)
            .with_children(|cmd| {
                cmd.spawn(base);
                cmd.spawn(fluid);
            })
            .id()
    }
}

#[rustfmt::skip]
pub const fn fluid_base_color(color: ColorDef) -> Srgba {
    match color {
        ColorDef::Void     => tailwind::GRAY_900,
        ColorDef::Red      => tailwind::RED_600,
        ColorDef::Green    => tailwind::GREEN_600,
        ColorDef::Blue     => tailwind::BLUE_600,
        ColorDef::Yellow   => tailwind::YELLOW_600,
        ColorDef::Magenta  => tailwind::PURPLE_600,
        ColorDef::Cyan     => tailwind::CYAN_600,
        ColorDef::White    => tailwind::GRAY_100,
    }
}

#[derive(Component, Default)]
pub struct CauldronQueue {
    minions: VecDeque<Entity>,
    state: CauldronQueueState,
}

enum CauldronQueueState {
    Cooldown(Timer),
    Pull((Entity, Vec3, Timer)), // old one in
    Pop((Entity, Vec3, Timer)),  // new one out
}

impl Default for CauldronQueueState {
    fn default() -> Self {
        Self::Cooldown(Timer::from_seconds(0.0, TimerMode::Once))
    }
}

#[derive(Component)]
pub struct CauldronTag;

pub fn queue_minion_for_cauldron(
    mut cauldron: Query<(Entity, &mut CauldronQueue), With<CauldronTag>>,
    mut started: EventReader<MinionStartedInteraction>,
) {
    for started in started.read() {
        for (cauldron, mut queue) in cauldron.iter_mut() {
            if started.target != cauldron {
                continue;
            }
            if !queue.minions.contains(&started.source) {
                info!("Queued a minion for cauldron");
                queue.minions.push_back(started.source);
            }
        }
    }
}

pub fn process_cauldron_queue(
    mut cmd: Commands,
    mut cauldron: Query<(Entity, &mut CauldronQueue, &Colored, &GlobalTransform)>,
    mut minion: Query<(
        Entity,
        &mut Transform,
        &GlobalTransform,
        &MinionKind,
        &mut MinionState,
    )>,
    assets: Res<MinionAssets>,
    time: Res<Time<Real>>,
) {
    for (cauldron, mut queue, colored, gx) in cauldron.iter_mut() {
        match &mut queue.state {
            CauldronQueueState::Cooldown(timer) => {
                timer.tick(time.delta());
                if timer.finished() {
                    if let Some(entity) = queue.minions.pop_front() {
                        let Ok((entity, _mtx, mgx, _kind, _)) = minion.get(entity) else {
                            warn!("Minion queueing for cauldron vanished");
                            queue.state = CauldronQueueState::default();
                            continue;
                        };

                        info!("Starting Pull");

                        cmd.entity(entity).remove::<MinionPath>();
                        queue.state = CauldronQueueState::Pull((
                            entity,
                            mgx.translation(),
                            Timer::from_seconds(1.2, TimerMode::Once),
                        ))
                    }
                }
            }
            CauldronQueueState::Pull((entity, src, timer)) => {
                let Ok((minion, mut mtx, mgx, kind, _)) = minion.get_mut(*entity) else {
                    warn!("Minion being pulled into cauldron vanished");
                    queue.state = CauldronQueueState::default();
                    continue;
                };

                timer.tick(time.delta());

                let to_minion = mtx.translation - mgx.translation();

                let t = timer.fraction();
                let y = if timer.fraction() < 0.5 {
                    f32::lerp(src.y, gx.translation().y + 3.5, t)
                } else {
                    f32::lerp(
                        gx.translation().y + 2.0,
                        gx.translation().y,
                        (t - 0.5) * 2.0,
                    )
                };

                let target = Vec3::lerp(*src, gx.translation(), t).with_y(y);
                mtx.translation = target + to_minion;

                mtx.rotation = Quat::from_axis_angle(Vec3::X, timer.fraction() * TAU * 2.0);

                if timer.finished() {
                    let new_color = ColorDef::from(*kind) + colored.color();
                    cmd.entity(minion).despawn_recursive();

                    let minion = MinionBuilder::new(
                        MinionKind::from(new_color),
                        gx.translation(),
                        MinionState::Interracting(cauldron),
                    )
                    .build(&mut cmd, &assets);

                    queue.state = CauldronQueueState::Pop((
                        minion,
                        gx.translation() + gx.forward() * 2.0 + Vec3::Y * 1.5,
                        Timer::from_seconds(1.2, TimerMode::Once),
                    ));
                }
            }
            CauldronQueueState::Pop((entity, dst, timer)) => {
                let Ok((_minion, mut mtx, mgx, _, mut state)) = minion.get_mut(*entity) else {
                    warn!("Minion popping out of cauldron vanished");
                    queue.state = CauldronQueueState::default();
                    continue;
                };
                timer.tick(time.delta());

                let to_minion = mtx.translation - mgx.translation();

                let t = timer.fraction();
                let y = if timer.fraction() < 0.5 {
                    f32::lerp(gx.translation().y, dst.y + 3.5, t)
                } else {
                    f32::lerp(dst.y + 2.0, dst.y, (t - 0.5) * 2.0)
                };
                let target = Vec3::lerp(gx.translation(), *dst, t).with_y(y);
                mtx.translation = target + to_minion;

                mtx.rotation = Quat::from_axis_angle(Vec3::X, timer.fraction() * TAU * 2.0);

                if timer.finished() {
                    *state = MinionState::GoingToPlayer;
                    queue.state =
                        CauldronQueueState::Cooldown(Timer::from_seconds(0.3, TimerMode::Once));
                }
            }
        }
    }
}

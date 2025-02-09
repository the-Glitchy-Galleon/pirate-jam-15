use crate::{
    framework::easing::Easing,
    game::{
        collision_groups::{ACTOR_GROUP, GROUND_GROUP, TARGET_GROUP, WALL_GROUP},
        common::RootParent,
        kinematic_char::KinematicCharacterBundle,
        minion::collector::MinionStorage,
        objects::camera::Shineable,
        player::{minion_storage::MinionStorageInput, PlayerTag},
        CharacterWalkControl, LevelResources,
    },
};
use bevy::{color::palettes::tailwind, prelude::*, time::Real};
use bevy_rapier3d::{
    plugin::RapierContext,
    prelude::{Collider, CollisionGroups, Group, QueryFilter},
};
use minion_builder::MinionMeshTag;
use std::f32::consts::{PI, TAU};
use vleue_navigator::{NavMesh, TransformedPath};

use super::common;

pub mod collector;
pub mod minion_builder;
pub mod walk_target;

pub const MINION_INTERRACTION_RANGE: f32 = 0.5;
pub const MINION_NODE_DIST: f32 = 0.1;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Component, Reflect, Default)]
pub enum MinionKind {
    #[default]
    Void,
    Red,
    Green,
    Blue,
    Yellow,
    Magenta,
    Cyan,
    White,
}

impl MinionKind {
    pub const VARIANTS: [MinionKind; 8] = [
        MinionKind::Void,
        MinionKind::Red,
        MinionKind::Green,
        MinionKind::Blue,
        MinionKind::Yellow,
        MinionKind::Magenta,
        MinionKind::Cyan,
        MinionKind::White,
    ];
    pub const COUNT: usize = Self::VARIANTS.len();

    #[rustfmt::skip]
    pub fn as_str(self) -> &'static str {
        match self {
            MinionKind::Void    => "Void",
            MinionKind::Red     => "Red",
            MinionKind::Green   => "Green",
            MinionKind::Blue    => "Blue",
            MinionKind::Yellow  => "Yellow",
            MinionKind::Magenta => "Magenta",
            MinionKind::Cyan    => "Cyan",
            MinionKind::White   => "White",
        }
    }
}

impl AsRef<str> for MinionKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

/// A component to mark an eligible target for the minions. The presence of that
/// component does not mean that it is currently being targetted.
#[derive(Clone, Copy, Default, Debug, Component, Reflect)]
#[reflect(Component)]
pub struct MinionTarget;

#[derive(Clone, Copy, Default, Debug, Component, Reflect, PartialEq, Eq)]
#[reflect(Component)]
pub enum MinionState {
    #[default]
    Idling,
    GoingToPlayer,
    GoingTo(Entity),
    Interracting(Entity),
}

#[derive(Bundle)]
pub struct MinionBundle {
    pub spatial: SpatialBundle,
    pub collider: Collider,
    pub collision_groups: CollisionGroups,
    pub character: KinematicCharacterBundle,
    pub kind: MinionKind,
    pub state: MinionState,
    pub shineable: Shineable,
}

impl Default for MinionBundle {
    fn default() -> Self {
        Self {
            spatial: Default::default(),
            collider: Default::default(),
            character: Default::default(),
            kind: Default::default(),
            state: Default::default(),
            collision_groups: CollisionGroups::new(ACTOR_GROUP, GROUND_GROUP | WALL_GROUP),
            shineable: Shineable,
        }
    }
}

#[derive(Component)]
pub struct MinionPath(TransformedPath);

// TODO: render it more aligned to the level
#[cfg(feature = "debug_visuals")]
pub fn debug_navmesh(
    level_reses: Res<LevelResources>,
    navmeshes: Res<Assets<NavMesh>>,
    mut gizmos: Gizmos,
) {
    let Some(navmesh) = &level_reses.navmesh else {
        return;
    };
    let Some(navmesh) = navmeshes.get(navmesh.id()) else {
        return;
    };
    let red = LinearRgba {
        red: 1.0,
        green: 0.0,
        blue: 0.0,
        alpha: 1.0,
    };
    let verts = &navmesh.get().vertices;

    for poly in &navmesh.get().polygons {
        let fst = poly
            .vertices
            .iter()
            .map(|x| *x)
            .map(|x| verts[x as usize].coords)
            .map(|v| Vec3::new(v.x, 0.0, v.y));
        let snd = poly
            .vertices
            .iter()
            .map(|x| *x)
            .skip(1)
            .chain(std::iter::once(poly.vertices[0]))
            .map(|x| verts[x as usize].coords)
            .map(|v| Vec3::new(v.x, 0.0, v.y));
        for (start, end) in fst.zip(snd) {
            gizmos.line(start, end, red);

            let center = (start + end) / 2.0;
            let dir = end - start;
            let ort = Vec3::new(-dir.z, 0.0, dir.x);
            gizmos.line(center, center + 0.3 * ort.normalize_or_zero(), red);
        }
    }
}

pub fn minion_update_path(
    mut minion_q: Query<(Entity, &MinionState, &MinionPath)>,
    target_q: Query<&GlobalTransform, With<MinionTarget>>,
    player_q: Query<&GlobalTransform, With<PlayerTag>>,
    mut commands: Commands,
) {
    let Ok(player_tf) = player_q.get_single() else {
        return;
    };

    for (ent, state, path) in minion_q.iter_mut() {
        let target_pos = match &state {
            MinionState::GoingToPlayer => player_tf.translation(),
            MinionState::GoingTo(target) => match target_q.get(*target) {
                Ok(tf) => tf.translation(),
                Err(e) => {
                    warn!("Failed to get target pos: {e}");
                    continue;
                }
            },
            _ => continue,
        };
        let target_navmesh_pos = Vec3::new(target_pos.x, 0.0, target_pos.z);
        let Some(last) = path.0.path.last() else {
            // info!("Removing last path");
            commands.entity(ent).remove::<MinionPath>();
            continue;
        };

        if target_navmesh_pos.distance(*last) < MINION_NODE_DIST {
            continue;
        }
        // info!("Removing path at the end of update");
        commands.entity(ent).remove::<MinionPath>();
    }
}

pub fn minion_build_path(
    level_reses: Res<LevelResources>,
    navmeshes: Res<Assets<NavMesh>>,
    mut minion_q: Query<(Entity, &GlobalTransform, &mut MinionState), Without<MinionPath>>,
    target_q: Query<&GlobalTransform, With<MinionTarget>>,
    player_q: Query<&GlobalTransform, With<PlayerTag>>,
    mut commands: Commands,
) {
    let Ok(player_tf) = player_q.get_single() else {
        return;
    };
    let Some(navmesh) = &level_reses.navmesh else {
        return;
    };
    let Some(navmesh) = navmeshes.get(navmesh) else {
        return;
    };

    for (ent, tf, mut state) in minion_q.iter_mut() {
        let target_pos = match state.as_ref() {
            MinionState::GoingToPlayer => player_tf.translation(),
            MinionState::GoingTo(target) => match target_q.get(*target) {
                Ok(tf) => tf.translation(),
                Err(e) => {
                    warn!("Failed to get target pos: {e}");
                    continue;
                }
            },
            _ => continue,
        };

        if !navmesh.transformed_is_in_mesh(tf.translation()) {
            error!("Minion is not in the navigation: {:?}", tf.translation());
            *state = MinionState::Idling;
            continue;
        }
        if !navmesh.transformed_is_in_mesh(target_pos) {
            warn!("Minion target is not in the navigation");
            *state = MinionState::Idling;
            continue;
        }

        let Some(path) = navmesh.transformed_path(
            Vec3::new(tf.translation().x, 0.0, tf.translation().z),
            Vec3::new(target_pos.x, 0.0, target_pos.z),
        ) else {
            warn!("Failed to find the path");
            *state = MinionState::Idling;
            continue;
        };

        commands.entity(ent).insert(MinionPath(path));
    }
}

pub fn minion_walk(
    level_reses: Res<LevelResources>,
    navmeshes: Res<Assets<NavMesh>>,
    mut minion_q: Query<(&GlobalTransform, &mut CharacterWalkControl, &mut MinionPath)>,
) {
    let Some(navmesh) = &level_reses.navmesh else {
        return;
    };
    let Some(navmesh) = navmeshes.get(navmesh) else {
        return;
    };

    for (tf, mut walk, mut path) in minion_q.iter_mut() {
        let path = &mut path.0.path;

        if let Some(p) = path.first().map(|x| *x) {
            let minion_pos = navmesh.transform().transform_point(tf.translation()).xy();
            let p = navmesh.transform().transform_point(p).xy();
            if p.distance(minion_pos) <= MINION_NODE_DIST {
                path.pop();
                // info!("Popped path, remaining: {}", path.len());
            }
        }

        if let Some(next) = path.first().map(|x| *x) {
            walk.do_move = true;
            walk.direction = next - Vec3::new(tf.translation().x, 0.0, tf.translation().z);
        }
    }
}

pub fn update_minion_state(
    mut minion_q: Query<(Entity, &GlobalTransform, &mut MinionState)>,
    target_q: Query<&GlobalTransform, With<MinionTarget>>,
    player_q: Query<(Entity, &GlobalTransform), With<PlayerTag>>,
    rap_ctx: ResMut<RapierContext>,
    mut started: EventWriter<MinionStartedInteraction>,
) {
    let Ok((player_ent, player_tf)) = player_q.get_single() else {
        return;
    };

    for (minion, tf, mut state) in minion_q.iter_mut() {
        let (target_pos, target_ent) = match state.as_ref() {
            MinionState::GoingToPlayer => (player_tf.translation(), player_ent),
            MinionState::GoingTo(target) => match target_q.get(*target) {
                Ok(tf) => (tf.translation(), *target),
                Err(e) => {
                    warn!("Failed to get target pos: {e}");
                    continue;
                }
            },
            _ => continue,
        };

        let is_target_reachable = rap_ctx
            .cast_ray(
                tf.translation(),
                (target_pos - tf.translation()).normalize(),
                MINION_INTERRACTION_RANGE,
                true,
                QueryFilter {
                    groups: Some(CollisionGroups::new(
                        Group::all(),
                        GROUND_GROUP | TARGET_GROUP,
                    )),
                    ..Default::default()
                },
            )
            .map(|(e, _)| e == target_ent)
            .unwrap_or_default();

        match *state {
            MinionState::GoingToPlayer if is_target_reachable => *state = MinionState::Idling,
            MinionState::GoingTo(target) if is_target_reachable => {
                *state = MinionState::Interracting(target);
                started.send(MinionStartedInteraction {
                    source: minion,
                    target,
                });
            }
            _ => (),
        }
    }
}

#[derive(Event)]
pub struct MinionStartedInteraction {
    pub source: Entity,
    pub target: Entity,
}

pub fn cleanup_minion_state(
    mut minion_q: Query<&mut MinionState>,
    target_q: Query<(), With<MinionTarget>>,
) {
    for mut st in minion_q.iter_mut() {
        match *st {
            MinionState::GoingTo(target) if !target_q.contains(target) => {
                *st = MinionState::Idling;
            }
            MinionState::Interracting(target) if !target_q.contains(target) => {
                *st = MinionState::Idling;
            }
            _ => continue,
        }
    }
}
#[cfg(feature = "debug_visuals")]
pub fn display_navigator_path(
    navigator: Query<(&Transform, &MinionPath, &GlobalTransform)>,
    mut gizmos: Gizmos,
) {
    for (_tx, path, gx) in &navigator {
        let y = gx.translation().y + 0.5;
        let mut to_display = path.0.path.clone();
        // to_display.push(tx.translation);
        // to_display.push(path.current.clone());
        to_display.reverse();
        if to_display.len() >= 1 {
            gizmos.linestrip(
                to_display.iter().map(|xz| Vec3::new(xz.x, y, xz.z)),
                tailwind::AMBER_200,
            );
        }
    }
}

#[derive(Component)]
pub struct ChosenMinionUi;

pub fn setup_chosen_minion_ui(mut cmd: Commands) {
    let sections =
        MinionKind::VARIANTS.map(|kind| TextSection::new(kind.as_str(), TextStyle::default()));
    cmd.spawn((
        TextBundle::from_sections(sections).with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(5.0),
            right: Val::Px(15.0),
            ..default()
        }),
        ChosenMinionUi,
    ));
}

pub fn update_chosen_minion_debug_ui(
    mut text: Query<&mut Text, With<ChosenMinionUi>>,
    minion: ResMut<MinionStorageInput>,
    mut storage: Query<&MinionStorage>,
) {
    let storage = storage.single_mut();
    let mut text = text.single_mut();

    for (i, kind) in MinionKind::VARIANTS.into_iter().enumerate() {
        let num_minions = storage.num_minions(kind);
        text.sections[i].value = format!("{}: {}\n", kind.as_str(), num_minions);

        text.sections[i].style.color = if kind == minion.chosen_ty {
            if num_minions == 0 {
                tailwind::AMBER_300.into()
            } else {
                tailwind::AMBER_50.into()
            }
        } else {
            if num_minions == 0 {
                tailwind::AMBER_900.into()
            } else {
                tailwind::AMBER_700.into()
            }
        };
    }
}

#[derive(Component, Default)]
pub struct MinionAnimation {
    walk_speed: f32, // 0.0..=1.0
    hop_t: f32,      // 0.0..=1.0
}

pub fn update_animation(
    mut walk: Query<(&CharacterWalkControl, &mut MinionAnimation)>,
    mut mesh: Query<(&RootParent, &mut Transform), With<MinionMeshTag>>,
    time: Res<Time<Real>>,
) {
    for (root, mut tx) in mesh.iter_mut() {
        if let Ok((walk, mut anim)) = walk.get_mut(root.parent()) {
            anim.walk_speed = common::approach_f32(
                anim.walk_speed,
                if walk.do_move { 1.0 } else { 0.0 },
                time.delta_seconds() * 3.0,
            );
            anim.hop_t = (anim.hop_t + time.delta_seconds()) % 1.0;
            let x = anim.walk_speed
                * 0.1
                * f32::sin(Easing::InOutPowf(3.0).apply(anim.hop_t as f64) as f32 * TAU);
            let z = anim.walk_speed
                * 0.2
                * f32::sin(Easing::InOutPowf(2.0).apply(anim.hop_t as f64) as f32 * TAU);
            let offset = walk.direction;
            let y = f32::atan2(offset.x, offset.z) - PI;
            tx.rotation = Quat::from_euler(EulerRot::XYZ, x, y, z);

            let y = (0.5 + 0.5 * f32::sin(anim.hop_t * TAU)) * anim.walk_speed * 0.4;
            tx.translation.y = -minion_builder::COLLIDER_HALF_HEIGHT + y - 0.05;
        }
    }
}

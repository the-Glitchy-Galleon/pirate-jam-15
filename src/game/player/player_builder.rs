use crate::game::{
    collision_groups::{ACTOR_GROUP, DETECTION_GROUP, GROUND_GROUP, TARGET_GROUP, WALL_GROUP},
    common::{self, ShowForwardGizmo},
    kinematic_char::{CharacterWalkControl, KinematicCharacterBundle},
    minion::{collector::MinionStorage, MinionKind},
    objects::{camera::Shineable, definitions::ColorDef},
    player::{minion_storage::PlayerCollector, PlayerTag},
};
use bevy::{asset::LoadState, color::palettes::tailwind, prelude::*, time::Real};
use bevy_rapier3d::prelude::*;
use std::f32::consts::{PI, TAU};

pub const COLLIDER_HALF_HEIGHT: f32 = 0.6;

pub struct PlayerBuilder {
    position: Vec3,
}

#[derive(Component)]
pub struct PlayerMeshTag;

impl PlayerBuilder {
    pub fn new(position: Vec3) -> Self {
        Self { position }
    }

    pub fn build(self, cmd: &mut Commands, assets: &PlayerAssets) -> Entity {
        let mut minion_storage = MinionStorage::new();

        for color in ColorDef::VARIANTS {
            let kind = MinionKind::from(color);
            for _ in 0..5 {
                minion_storage.add_minion(kind);
            }
        }
        let root = (
            Name::new("Player"),
            ShowForwardGizmo,
            PlayerTag,
            PlayerAnimation::default(),
            minion_storage,
            Collider::round_cylinder(COLLIDER_HALF_HEIGHT, 0.15, 0.2),
            CollisionGroups::new(ACTOR_GROUP | TARGET_GROUP, GROUND_GROUP | WALL_GROUP),
            SpatialBundle {
                transform: Transform::from_translation(
                    self.position + Vec3::Y * (COLLIDER_HALF_HEIGHT + 0.5),
                ),
                visibility: Visibility::Hidden,
                ..default()
            },
            KinematicCharacterBundle::default(),
            Shineable,
        );
        let collector = (SpatialBundle::default(), PlayerCollector);
        let collector_sensor = (
            SpatialBundle {
                transform: Transform::from_rotation(Quat::from_rotation_x(
                    std::f32::consts::FRAC_PI_2,
                ))
                .with_translation(Vec3::new(0.0, -1.0, -3.0)),
                ..default()
            },
            ActiveCollisionTypes::KINEMATIC_STATIC,
            Collider::cone(3.0, 4.5),
            CollisionGroups::new(DETECTION_GROUP, ACTOR_GROUP),
            RigidBody::Fixed,
            Sensor,
        );
        let mesh = (
            PlayerMeshTag,
            PbrBundle {
                transform: Transform::from_scale(Vec3::splat(0.8)),
                ..Default::default()
            },
        );
        let clothing = PbrBundle {
            mesh: assets.clothing_mesh.clone(),
            material: assets.clothing_material.clone(),
            ..Default::default()
        };
        let head = PbrBundle {
            mesh: assets.head_mesh.clone(),
            material: assets.head_material.clone(),
            ..Default::default()
        };
        let eyes = PbrBundle {
            mesh: assets.eyes_mesh.clone(),
            material: assets.eyes_material.clone(),
            ..Default::default()
        };
        let staff = PbrBundle {
            mesh: assets.staff_mesh.clone(),
            material: assets.staff_material.clone(),
            ..Default::default()
        };
        let orb = PbrBundle {
            mesh: assets.orb_mesh.clone(),
            material: assets.orb_material.clone(),
            ..Default::default()
        };

        // spawn mesh separately
        cmd.spawn(mesh).with_children(|cmd| {
            cmd.spawn(clothing);
            cmd.spawn(head);
            cmd.spawn(eyes);
            cmd.spawn(staff);
            cmd.spawn(orb);
        });

        cmd.spawn(root)
            .with_children(|cmd| {
                cmd.spawn(collector).with_children(|cmd| {
                    cmd.spawn(collector_sensor);
                });
            })
            .id()
    }
}

#[derive(Resource)]
pub struct PlayerAssets {
    clothing_mesh: Handle<Mesh>,
    head_mesh: Handle<Mesh>,
    eyes_mesh: Handle<Mesh>,
    staff_mesh: Handle<Mesh>,
    orb_mesh: Handle<Mesh>,
    clothing_material: Handle<StandardMaterial>,
    head_material: Handle<StandardMaterial>,
    eyes_material: Handle<StandardMaterial>,
    staff_material: Handle<StandardMaterial>,
    orb_material: Handle<StandardMaterial>,
}

pub fn load_player_assets(
    mut cmd: Commands,
    ass: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let clothing_mesh = ass.load("player.glb#Mesh0/Primitive0");
    let head_mesh = ass.load("player.glb#Mesh0/Primitive1");
    let eyes_mesh = ass.load("player.glb#Mesh0/Primitive2");
    let staff_mesh = ass.load("player.glb#Mesh0/Primitive3");
    let orb_mesh = ass.load("player.glb#Mesh0/Primitive4");

    let clothing_material = materials.add(StandardMaterial {
        base_color: tailwind::GRAY_100.into(),
        emissive: (tailwind::GRAY_100 * 0.1).into(),
        ..Default::default()
    });
    let head_material = materials.add(StandardMaterial {
        base_color: tailwind::GRAY_950.into(),
        emissive: (tailwind::GRAY_950 * 0.1).into(),
        ..Default::default()
    });
    let eyes_material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        emissive: (Srgba::WHITE * 0.1).into(),
        ..Default::default()
    });
    let staff_material = materials.add(StandardMaterial {
        base_color: tailwind::RED_800.into(),
        emissive: (tailwind::RED_800 * 0.1).into(),
        ..Default::default()
    });
    let orb_material = materials.add(StandardMaterial {
        base_color: tailwind::PURPLE_500.into(),
        emissive: (tailwind::PURPLE_500 * 0.1).into(),
        ..Default::default()
    });

    cmd.insert_resource(PlayerAssets {
        clothing_mesh,
        head_mesh,
        eyes_mesh,
        staff_mesh,
        orb_mesh,
        clothing_material,
        head_material,
        eyes_material,
        staff_material,
        orb_material,
    });
}

#[rustfmt::skip]
pub fn are_player_assets_ready(
    ass: &AssetServer,
    assets: &PlayerAssets,
) -> bool {
    ass.load_state(&assets.clothing_mesh)     != LoadState::Loading && 
    ass.load_state(&assets.head_mesh)         != LoadState::Loading && 
    ass.load_state(&assets.eyes_mesh)         != LoadState::Loading && 
    ass.load_state(&assets.staff_mesh)        != LoadState::Loading && 
    ass.load_state(&assets.orb_mesh)          != LoadState::Loading && 
    ass.load_state(&assets.clothing_material) != LoadState::Loading && 
    ass.load_state(&assets.head_material)     != LoadState::Loading && 
    ass.load_state(&assets.eyes_material)     != LoadState::Loading && 
    ass.load_state(&assets.staff_material)    != LoadState::Loading && 
    ass.load_state(&assets.orb_material)      != LoadState::Loading
}

#[derive(Component, Default)]
pub struct PlayerAnimation {
    walk_speed: f32, // 0.0..=1.0
}

pub fn update_animation(
    mut mesh: Query<&mut Transform, With<PlayerMeshTag>>,
    mut player: Query<
        (
            &GlobalTransform,
            &CharacterWalkControl,
            &mut PlayerAnimation,
        ),
        With<PlayerTag>,
    >,
    time: Res<Time<Real>>,
) {
    let mut mesh = mesh.single_mut();
    let (player_gx, walk, mut anim) = player.single_mut();

    anim.walk_speed = common::approach_f32(
        anim.walk_speed,
        if walk.do_move { 1.0 } else { 0.0 },
        time.delta_seconds() * 3.0,
    );

    // the kinematic root component jitters around a little,
    // so try to smooth out the mesh following it as best as possible
    let (_scale, rotation, translation) = player_gx.to_scale_rotation_translation();

    let hover = 0.5 + 0.5 * (time.elapsed_seconds() * 2.0 % TAU).sin();
    let y = -COLLIDER_HALF_HEIGHT + hover * 0.3 - 0.2;
    let translation = translation + Vec3::Y * y;
    mesh.translation = Vec3::lerp(
        mesh.translation,
        translation,
        f32::clamp(time.delta_seconds() * 5.0, 0.0, 0.5),
    );

    let lean = Quat::from_axis_angle(Vec3::NEG_X, anim.walk_speed * TAU * 0.05);
    let rotation = rotation * lean;

    mesh.rotation = Quat::slerp(
        mesh.rotation,
        rotation,
        f32::clamp(time.delta_seconds() * PI * 1.2, 0.0, 0.5),
    );
}

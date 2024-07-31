use crate::game::{
    collision_groups::{ACTOR_GROUP, DETECTION_GROUP, GROUND_GROUP},
    common::ShowForwardGizmo,
    kinematic_char::KinematicCharacterBundle,
    minion::{MinionAnimation, MinionKind, MinionState},
    objects::{camera::Shineable, definitions::ColorDef},
};
use bevy::{asset::LoadState, color::palettes::tailwind, prelude::*};
use bevy_rapier3d::prelude::*;

pub const COLLIDER_HALF_HEIGHT: f32 = 0.3;

pub struct MinionBuilder {
    kind: MinionKind,
    position: Vec3,
    state: MinionState,
}

#[derive(Component)]
pub struct MinionMeshTag;

impl MinionBuilder {
    pub fn new(kind: MinionKind, position: Vec3, state: MinionState) -> Self {
        Self {
            kind,
            position,
            state,
        }
    }
    pub fn build(self, cmd: &mut Commands, assets: &MinionAssets) -> Entity {
        let root = (
            Name::new(format!("{} Minion", ColorDef::from(self.kind).as_str())),
            MinionAnimation::default(),
            SpatialBundle {
                transform: Transform::from_translation(self.position),
                ..default()
            },
            self.kind,
            self.state,
            Shineable,
            Collider::cuboid(0.3, COLLIDER_HALF_HEIGHT, 0.3),
            // good idea? add detection group so they get matched with the pickup cone
            CollisionGroups::new(ACTOR_GROUP, GROUND_GROUP | DETECTION_GROUP),
            KinematicCharacterBundle::default(),
        );
        let body = PbrBundle {
            mesh: assets.body_mesh(self.kind),
            material: assets.body_material(self.kind),
            ..Default::default()
        };
        let eyes = PbrBundle {
            mesh: assets.eye_mesh(self.kind),
            material: assets.eye_material(self.kind),
            ..Default::default()
        };
        let mesh = (
            ShowForwardGizmo,
            MinionMeshTag,
            SpatialBundle {
                transform: Transform::IDENTITY
                    .with_translation(Vec3::NEG_Y * COLLIDER_HALF_HEIGHT)
                    .with_scale(Vec3::splat(minion_scene_scale(self.kind))),
                ..Default::default()
            },
        );

        cmd.spawn(root)
            .with_children(|cmd| {
                cmd.spawn(mesh).with_children(|cmd| {
                    cmd.spawn(body);
                    cmd.spawn(eyes);
                });
            })
            .id()
    }
}

#[derive(Resource)]
pub struct MinionAssets {
    body_meshes: [Handle<Mesh>; ColorDef::COUNT],
    eye_meshes: [Handle<Mesh>; ColorDef::COUNT],
    body_materials: [Handle<StandardMaterial>; ColorDef::COUNT],
    eye_materials: [Handle<StandardMaterial>; ColorDef::COUNT],
}
impl MinionAssets {
    #[rustfmt::skip]
    pub fn body_mesh(&self, kind: MinionKind) -> Handle<Mesh> {
        match kind {
            MinionKind::Void    => self.body_meshes[0].clone(),
            MinionKind::Red     => self.body_meshes[1].clone(),
            MinionKind::Green   => self.body_meshes[2].clone(),
            MinionKind::Blue    => self.body_meshes[3].clone(),
            MinionKind::Yellow  => self.body_meshes[4].clone(),
            MinionKind::Magenta => self.body_meshes[5].clone(),
            MinionKind::Cyan    => self.body_meshes[6].clone(),
            MinionKind::White   => self.body_meshes[7].clone(),
        }
    }
    #[rustfmt::skip]
    pub fn eye_mesh(&self, kind: MinionKind) -> Handle<Mesh> {
        match kind {
            MinionKind::Void    => self.eye_meshes[0].clone(),
            MinionKind::Red     => self.eye_meshes[1].clone(),
            MinionKind::Green   => self.eye_meshes[2].clone(),
            MinionKind::Blue    => self.eye_meshes[3].clone(),
            MinionKind::Yellow  => self.eye_meshes[4].clone(),
            MinionKind::Magenta => self.eye_meshes[5].clone(),
            MinionKind::Cyan    => self.eye_meshes[6].clone(),
            MinionKind::White   => self.eye_meshes[7].clone(),
        }
    }
    #[rustfmt::skip]
    pub fn body_material(&self, kind: MinionKind) -> Handle<StandardMaterial> {
        match kind {
            MinionKind::Void    => self.body_materials[0].clone(),
            MinionKind::Red     => self.body_materials[1].clone(),
            MinionKind::Green   => self.body_materials[2].clone(),
            MinionKind::Blue    => self.body_materials[3].clone(),
            MinionKind::Yellow  => self.body_materials[4].clone(),
            MinionKind::Magenta => self.body_materials[5].clone(),
            MinionKind::Cyan    => self.body_materials[6].clone(),
            MinionKind::White   => self.body_materials[7].clone(),
        }
    }
    #[rustfmt::skip]
    pub fn eye_material(&self, kind: MinionKind) -> Handle<StandardMaterial> {
        match kind {
            MinionKind::Void    => self.eye_materials[0].clone(),
            MinionKind::Red     => self.eye_materials[1].clone(),
            MinionKind::Green   => self.eye_materials[2].clone(),
            MinionKind::Blue    => self.eye_materials[3].clone(),
            MinionKind::Yellow  => self.eye_materials[4].clone(),
            MinionKind::Magenta => self.eye_materials[5].clone(),
            MinionKind::Cyan    => self.eye_materials[6].clone(),
            MinionKind::White   => self.eye_materials[7].clone(),
        }
    }
}

pub fn load_minion_assets(
    mut cmd: Commands,
    ass: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    #[rustfmt::skip]
    let body_meshes = ColorDef::VARIANTS.map(|color| {
        match color {
            ColorDef::Void    => ass.load("minions.glb#Mesh4/Primitive1"),
            ColorDef::Red     => ass.load("minions.glb#Mesh1/Primitive1"),
            ColorDef::Green   => ass.load("minions.glb#Mesh2/Primitive1"),
            ColorDef::Blue    => ass.load("minions.glb#Mesh3/Primitive1"),
            ColorDef::Yellow  => ass.load("minions.glb#Mesh7/Primitive0"),
            ColorDef::Magenta => ass.load("minions.glb#Mesh6/Primitive0"),
            ColorDef::Cyan    => ass.load("minions.glb#Mesh5/Primitive0"),
            ColorDef::White   => ass.load("minions.glb#Mesh0/Primitive0"),
        }
    });

    #[rustfmt::skip]
    let eye_meshes = ColorDef::VARIANTS.map(|color| {
        match color {
            ColorDef::Void    => ass.load("minions.glb#Mesh4/Primitive0"),
            ColorDef::Red     => ass.load("minions.glb#Mesh1/Primitive0"),
            ColorDef::Green   => ass.load("minions.glb#Mesh2/Primitive0"),
            ColorDef::Blue    => ass.load("minions.glb#Mesh3/Primitive0"),
            ColorDef::Yellow  => ass.load("minions.glb#Mesh7/Primitive1"),
            ColorDef::Magenta => ass.load("minions.glb#Mesh6/Primitive1"),
            ColorDef::Cyan    => ass.load("minions.glb#Mesh5/Primitive1"),
            ColorDef::White   => ass.load("minions.glb#Mesh0/Primitive1"),
        }
    });

    #[rustfmt::skip]
    let body_materials = ColorDef::VARIANTS.map(|color| {
        let color = minion_base_body_color(MinionKind::from(color));
        materials.add(StandardMaterial {
            base_color: color.into(),
            emissive: (color * 0.2).into(),
            perceptual_roughness: 0.3,
            ..Default::default()
        })
    });

    #[rustfmt::skip]
    let eye_materials = ColorDef::VARIANTS.map(|color| {
        let color = minion_base_eye_color(MinionKind::from(color));
        materials.add(StandardMaterial {
            base_color: color.into(),
            emissive: (color * 2.0).into(),
            ..Default::default()
        })
    });

    cmd.insert_resource(MinionAssets {
        body_meshes,
        eye_meshes,
        body_materials,
        eye_materials,
    });
}

#[rustfmt::skip]
pub fn are_minion_assets_ready(
    ass: &AssetServer,
    assets: &MinionAssets,
) -> bool {
    assets.body_meshes   .iter().map(|u| (ass.load_state(u) == LoadState::Loading) as u32).sum::<u32>() == 0 && 
    assets.eye_meshes    .iter().map(|u| (ass.load_state(u) == LoadState::Loading) as u32).sum::<u32>() == 0 && 
    assets.body_materials.iter().map(|u| (ass.load_state(u) == LoadState::Loading) as u32).sum::<u32>() == 0 && 
    assets.eye_materials .iter().map(|u| (ass.load_state(u) == LoadState::Loading) as u32).sum::<u32>() == 0
}

#[rustfmt::skip]
pub fn minion_scene_scale(kind: MinionKind) -> f32 {
    0.9 * match kind {
        MinionKind::Void    => 0.500,
        MinionKind::Red     => 0.275,
        MinionKind::Green   => 0.250,
        MinionKind::Blue    => 0.250,
        MinionKind::Yellow  => 0.275,
        MinionKind::Magenta => 0.275,
        MinionKind::Cyan    => 0.250,
        MinionKind::White   => 0.250,
    }
}

#[rustfmt::skip]
pub const fn minion_base_body_color(kind: MinionKind) -> Srgba {
    match kind {
        MinionKind::Void     => tailwind::GRAY_800,
        MinionKind::Red      => tailwind::RED_600,
        MinionKind::Green    => tailwind::GREEN_600,
        MinionKind::Blue     => tailwind::BLUE_600,
        MinionKind::Yellow   => tailwind::YELLOW_600,
        MinionKind::Magenta  => tailwind::PURPLE_600,
        MinionKind::Cyan     => tailwind::CYAN_600,
        MinionKind::White    => tailwind::GRAY_100,
    }
}

#[rustfmt::skip]
pub const fn minion_base_eye_color(kind: MinionKind) -> Srgba {
    match kind {
        MinionKind::Void     => tailwind::GRAY_900,
        MinionKind::Red      => tailwind::RED_100,
        MinionKind::Green    => tailwind::GREEN_100,
        MinionKind::Blue     => tailwind::BLUE_100,
        MinionKind::Yellow   => tailwind::YELLOW_100,
        MinionKind::Magenta  => tailwind::PURPLE_100,
        MinionKind::Cyan     => tailwind::CYAN_100,
        MinionKind::White    => Srgba::WHITE,
    }
}

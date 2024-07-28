use crate::game::{
    collision_groups::{ACTOR_GROUP, DETECTION_GROUP, GROUND_GROUP},
    kinematic_char::KinematicCharacterBundle,
    minion::{MinionKind, MinionState},
    objects::{camera::Shineable, definitions::ColorDef},
};
use bevy::{color::palettes::tailwind, prelude::*};
use bevy_rapier3d::prelude::*;

pub struct MinionBuilder {
    kind: MinionKind,
    position: Vec3,
    state: MinionState,
}

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
            SpatialBundle {
                transform: Transform::from_translation(self.position),
                ..default()
            },
            self.kind,
            self.state,
            Shineable,
            Collider::cuboid(0.3, 0.3, 0.3),
            // good idea? add detection group so they get matched with the pickup cone
            CollisionGroups::new(ACTOR_GROUP, GROUND_GROUP | DETECTION_GROUP),
            KinematicCharacterBundle::default(),
        );
        let scene = SceneBundle {
            scene: assets.scene(self.kind),
            transform: Transform::IDENTITY
                .with_translation(Vec3::NEG_Y * 0.3) // move down collider half-size
                .with_scale(Vec3::splat(minion_scene_scale(self.kind))),
            ..Default::default()
        };

        cmd.spawn(root)
            .with_children(|cmd| {
                cmd.spawn(scene);
            })
            .id()
    }
}

#[derive(Resource)]
pub struct MinionAssets {
    scenes: [Handle<Scene>; ColorDef::COUNT],
}
impl MinionAssets {
    #[rustfmt::skip]
    pub fn scene(&self, kind: MinionKind) -> Handle<Scene> {
        match kind {
            MinionKind::Void    => self.scenes[0].clone(),
            MinionKind::Red     => self.scenes[1].clone(),
            MinionKind::Green   => self.scenes[2].clone(),
            MinionKind::Blue    => self.scenes[3].clone(),
            MinionKind::Yellow  => self.scenes[4].clone(),
            MinionKind::Magenta => self.scenes[5].clone(),
            MinionKind::Cyan    => self.scenes[6].clone(),
            MinionKind::White   => self.scenes[7].clone(),
        }
    }
}

impl FromWorld for MinionAssets {
    fn from_world(world: &mut World) -> Self {
        let ass = world.resource::<AssetServer>();

        #[rustfmt::skip]
        let scenes = ColorDef::VARIANTS.map(|color| {
            match color {
                ColorDef::Void    => ass.load("minion_void.glb#Scene0"),
                ColorDef::Red     => ass.load("minion_red.glb#Scene0"),
                ColorDef::Green   => ass.load("minion_green.glb#Scene0"),
                ColorDef::Blue    => ass.load("minion_blue.glb#Scene0"),
                ColorDef::Yellow  => ass.load("minion_yellow.glb#Scene0"),
                ColorDef::Magenta => ass.load("minion_purple.glb#Scene0"),
                ColorDef::Cyan    => ass.load("minion_cyan.glb#Scene0"),
                ColorDef::White   => ass.load("minion_white.glb#Scene0"),
            }
        });

        Self { scenes }
    }
}

#[rustfmt::skip]
pub const fn minion_scene_scale(kind: MinionKind) -> f32 {
    match kind {
        MinionKind::Void    => 0.500,
        MinionKind::Red     => 0.300,
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
        MinionKind::Void     => tailwind::GRAY_900,
        MinionKind::Red      => tailwind::RED_600,
        MinionKind::Green    => tailwind::GREEN_600,
        MinionKind::Blue     => tailwind::BLUE_600,
        MinionKind::Yellow   => tailwind::YELLOW_600,
        MinionKind::Magenta  => tailwind::PURPLE_600,
        MinionKind::Cyan     => tailwind::CYAN_600,
        MinionKind::White    => tailwind::GRAY_100,
    }
}

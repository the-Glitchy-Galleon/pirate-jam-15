use crate::{
    framework::audio::AudioReceiver,
    game::{
        common::{self, PrimaryCamera},
        player::PlayerTag,
    },
    AppState,
};
use bevy::{prelude::*, time::Real};
use std::f32::consts::{PI, TAU};

pub struct TopDownCameraBuilder {
    height: f32,
    distance: f32,
    initial_focus: Vec3,
}

impl TopDownCameraBuilder {
    pub fn new(height: f32, distance: f32) -> Self {
        Self {
            height,
            distance,
            initial_focus: Vec3::new(0.0, 10.0, 0.0),
        }
    }
    pub fn build(self, cmd: &mut Commands) -> Entity {
        let camera = (
            Name::new("Top-Down Camera"),
            AudioReceiver,
            PrimaryCamera,
            TopDownCamera::new(self.height, self.distance),
            Camera3dBundle {
                transform: Transform::IDENTITY.looking_at(self.initial_focus, Vec3::Y),
                ..Default::default()
            },
        );
        cmd.spawn(camera).id()
    }
}

pub struct TopDownCameraPlugin;

impl Plugin for TopDownCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (update_camera_root).run_if(in_state(AppState::Ingame)),
        );
    }
}

#[derive(Component, Default)]
pub struct TopDownCamera {
    distance: f32,
    height: f32,
    target_angle_y: f32,
    angle_y: f32,
}
impl TopDownCamera {
    pub fn new(height: f32, distance: f32) -> Self {
        Self {
            height,
            distance,
            ..Default::default()
        }
    }

    pub fn set_target_angle_y(&mut self, angle: f32) {
        self.target_angle_y = common::normalized_angle(angle + PI);
    }
    pub fn set_target_angle_from_direction(&mut self, direction: Vec3) {
        self.set_target_angle_y(f32::atan2(direction.x, direction.z));
    }
}

pub fn update_camera_root(
    mut camera: Query<(&mut TopDownCamera, &mut Transform), With<TopDownCamera>>,
    player_gx: Query<&GlobalTransform, With<PlayerTag>>,
    time: Res<Time<Real>>,
) {
    let (mut camera, mut camera_tx) = camera.single_mut();
    let player_gx = player_gx.single();

    let dist = common::angle_distance(camera.target_angle_y, camera.angle_y);
    camera.angle_y = common::approach_angle(
        camera.angle_y,
        camera.target_angle_y,
        1.5 * time.delta_seconds() * TAU * f32::clamp(dist / PI, 0.1, 1.0),
    );
    let player_pos = player_gx.translation();
    let target_camera_pos = Vec3::new(
        player_pos.x + camera.distance * camera.angle_y.sin(),
        player_pos.y + camera.height,
        player_pos.z + camera.distance * camera.angle_y.cos(),
    );
    camera_tx.translation = Vec3::lerp(
        camera_tx.translation,
        target_camera_pos,
        f32::clamp(10.0 * time.delta_seconds(), 0.0, 0.5),
    );
    camera_tx.translation = target_camera_pos;

    let mut rot_copy = camera_tx.clone();
    rot_copy.look_at(player_pos, Vec3::Y);
    let rot_copy = rot_copy.rotation;

    camera_tx.rotation = camera_tx.rotation.slerp(
        rot_copy,
        f32::clamp(time.delta_seconds() * PI * 10.0, 0.0, 0.5),
    );
}

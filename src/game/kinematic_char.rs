//! Module to provide some simple API for kinematic character controller
//! provided by `bevy_rapier3d`.
//! The API supports jumping.

use crate::game::collision_groups::{ACTOR_GROUP, GROUND_GROUP, WALL_GROUP};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub const GROUND_TIMER: f32 = 0.5;
pub const MOVEMENT_SPEED: f32 = 4.0;
// pub const JUMP_SPEED: f32 = 20.0;
pub const GRAVITY: f32 = -9.81;

/// Controls how the character shall move.
#[derive(Clone, Copy, Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct CharacterWalkControl {
    /// Move direction. Doesn't need to be normalised.
    /// Can be a zero vec. The game will only acknowledge
    /// the part that is within the Oxz plane
    pub direction: Vec3,
    /// Makes the character move in the specified direction.
    /// Resets on next frame.
    pub do_move: bool,
}

#[derive(Clone, Copy, Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct CharacterWalkState {
    pub grounded_timer: f32,
    pub vertical_movement: f32,
}

#[derive(Clone, Bundle, Debug)]
pub struct KinematicCharacterBundle {
    pub controller: KinematicCharacterController,
    pub control: CharacterWalkControl,
    pub state: CharacterWalkState,
    pub rigid_body: RigidBody,
}

impl Default for KinematicCharacterBundle {
    fn default() -> Self {
        Self {
            controller: KinematicCharacterController {
                custom_mass: Some(5.0),
                up: Vec3::Y,
                offset: CharacterLength::Absolute(0.01),
                slide: true,
                autostep: Some(CharacterAutostep {
                    max_height: CharacterLength::Relative(0.3),
                    min_width: CharacterLength::Relative(0.5),
                    include_dynamic_bodies: false,
                }),
                // Don’t allow climbing slopes larger than 45 degrees.
                max_slope_climb_angle: 45.0_f32.to_radians(),
                // Automatically slide down on slopes smaller than 30 degrees.
                min_slope_slide_angle: 30.0_f32.to_radians(),
                apply_impulse_to_dynamic_bodies: true,
                snap_to_ground: None,
                filter_groups: Some(CollisionGroups::new(ACTOR_GROUP, GROUND_GROUP | WALL_GROUP)),
                ..default()
            },
            rigid_body: RigidBody::KinematicPositionBased,
            control: CharacterWalkControl::default(),
            state: CharacterWalkState::default(),
        }
    }
}

pub fn update_kinematic_character(
    time: Res<Time>,
    mut player: Query<(
        &mut KinematicCharacterController,
        Option<&KinematicCharacterControllerOutput>,
        &mut CharacterWalkControl,
        &mut CharacterWalkState,
    )>,
) {
    let delta_time = time.delta_seconds();

    for (mut controller, output, mut walk, mut state) in player.iter_mut() {
        /* Retrieve input */
        let mut movement = Vec3::ZERO;
        if walk.do_move {
            movement = Vec3::new(walk.direction.x, 0.0, walk.direction.z).normalize_or_zero()
                * MOVEMENT_SPEED;
        }
        walk.do_move = false;
        // no jumping
        // let jump_speed = walk.direction.y * JUMP_SPEED;
        let jump_speed = 0.0;
        /* Check physics ground check */
        let grounded = output.map(|o| o.grounded).unwrap_or_default();
        if grounded {
            state.grounded_timer = GROUND_TIMER;
            state.vertical_movement = 0.0;
        }
        if state.grounded_timer > 0.0 {
            state.grounded_timer -= delta_time;
            if jump_speed > 0.0 {
                state.vertical_movement = jump_speed;
                state.grounded_timer = 0.0;
            }
        }
        movement.y = state.vertical_movement;
        state.vertical_movement += GRAVITY * delta_time * controller.custom_mass.unwrap_or(1.0);
        controller.translation = Some(movement * delta_time);
    }
}

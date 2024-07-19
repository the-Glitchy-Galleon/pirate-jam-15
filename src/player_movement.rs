use bevy::{
    input::InputSystem,
    prelude::*,
    render::camera::RenderTarget,
    window::{PrimaryWindow, WindowRef},
};
use bevy_rapier3d::prelude::*;

const GROUND_TIMER: f32 = 0.5;
const MOVEMENT_SPEED: f32 = 8.0;
const JUMP_SPEED: f32 = 20.0;
const GRAVITY: f32 = -9.81;

/// Keyboard input vector
#[derive(Default, Resource, Deref, DerefMut)]
pub struct MovementInput(pub Vec3);

#[derive(Resource, Deref, DerefMut)]
pub struct PlayerDirection(pub Dir3);

pub fn player_movement(
    time: Res<Time>,
    mut input: ResMut<MovementInput>,
    mut player: Query<(
        &mut Transform,
        &mut KinematicCharacterController,
        Option<&KinematicCharacterControllerOutput>,
    )>,
    mut vertical_movement: Local<f32>,
    mut grounded_timer: Local<f32>,
) {
    let Ok((transform, mut controller, output)) = player.get_single_mut() else {
        return;
    };
    let delta_time = time.delta_seconds();
    // Retrieve input
    let mut movement = Vec3::new(input.x, 0.0, input.z) * MOVEMENT_SPEED;
    let jump_speed = input.y * JUMP_SPEED;
    // Clear input
    **input = Vec3::ZERO;
    // Check physics ground check
    if output.map(|o| o.grounded).unwrap_or(false) {
        *grounded_timer = GROUND_TIMER;
        *vertical_movement = 0.0;
    }
    // If we are grounded we can jump
    if *grounded_timer > 0.0 {
        *grounded_timer -= delta_time;
        // If we jump we clear the grounded tolerance
        if jump_speed > 0.0 {
            *vertical_movement = jump_speed;
            *grounded_timer = 0.0;
        }
    }
    movement.y = *vertical_movement;
    *vertical_movement += GRAVITY * delta_time * controller.custom_mass.unwrap_or(1.0);
    controller.translation = Some(transform.rotation * (movement * delta_time));
}

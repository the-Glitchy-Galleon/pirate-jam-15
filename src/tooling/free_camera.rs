use bevy::{input::mouse::MouseMotion, prelude::*};
use std::f32::consts::PI;

#[derive(Component)]
pub struct FreeCameraTag;

#[derive(Default)]
pub struct FreeCameraPlugin {
    pub transform: Transform,
}

impl Plugin for FreeCameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(FreeCameraSetup {
            transform: self.transform,
        })
        .add_systems(PreStartup, setup)
        .add_systems(Update, camera_controller);
    }
}

fn setup(mut commands: Commands, setup: Res<FreeCameraSetup>) {
    commands.spawn((
        FreeCameraTag,
        Camera3dBundle {
            transform: setup.transform,
            ..default()
        },
        CameraController::default(),
    ));
}

#[derive(Resource)]
struct FreeCameraSetup {
    transform: Transform,
}

#[derive(Component)]
pub struct CameraController {
    pub enabled: bool,
    pub initialized: bool,
    pub sensitivity: f32,
    pub key_forward: KeyCode,
    pub key_back: KeyCode,
    pub key_left: KeyCode,
    pub key_right: KeyCode,
    pub key_up: KeyCode,
    pub key_down: KeyCode,
    pub key_run: KeyCode,
    pub mouse_key_enable_mouse: Option<MouseButton>,
    pub walk_speed: f32,
    pub run_speed: f32,
    pub friction: f32,
    pub pitch: f32,
    pub yaw: f32,
    pub velocity: Vec3,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            enabled: true,
            initialized: false,
            sensitivity: 0.2,
            key_forward: KeyCode::KeyW,
            key_back: KeyCode::KeyS,
            key_left: KeyCode::KeyA,
            key_right: KeyCode::KeyD,
            key_up: KeyCode::KeyE,
            key_down: KeyCode::KeyQ,
            key_run: KeyCode::ShiftLeft,
            mouse_key_enable_mouse: Some(MouseButton::Right),
            walk_speed: 6.0,
            run_speed: 24.0,
            friction: 0.5,
            pitch: 0.0,
            yaw: 0.0,
            velocity: Vec3::ZERO,
        }
    }
}

pub fn camera_controller(
    time: Res<Time>,
    mut mouse_evs: EventReader<MouseMotion>,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut camera: Query<(&mut Transform, &mut CameraController), With<Camera>>,
) {
    let Ok((mut transform, mut controller)) = camera.get_single_mut() else {
        return;
    };

    let dt = time.delta_seconds();

    if !controller.initialized {
        let (yaw, pitch, _roll) = transform.rotation.to_euler(EulerRot::YXZ);
        controller.yaw = yaw;
        controller.pitch = pitch;
        controller.initialized = true;
    }
    if !controller.enabled {
        return;
    }

    #[rustfmt::skip]
    let axis_input = {
        let mut axis_input = Vec3::ZERO;
        if keys.pressed(controller.key_forward) { axis_input.z += 1.0; }
        if keys.pressed(controller.key_back)    { axis_input.z -= 1.0; }
        if keys.pressed(controller.key_right)   { axis_input.x += 1.0; }
        if keys.pressed(controller.key_left)    { axis_input.x -= 1.0; }
        if keys.pressed(controller.key_up)      { axis_input.y += 1.0; }
        if keys.pressed(controller.key_down)    { axis_input.y -= 1.0; }
        axis_input
    };

    if axis_input != Vec3::ZERO {
        let max_speed = keys
            .pressed(controller.key_run)
            .then_some(controller.run_speed)
            .unwrap_or(controller.walk_speed);

        controller.velocity = axis_input.normalize() * max_speed;
    } else {
        let friction = controller.friction.clamp(0.0, 1.0);
        controller.velocity *= 1.0 - friction;
        if controller.velocity.length_squared() < 1e-6 {
            controller.velocity = Vec3::ZERO;
        }
    }

    let forward = transform.forward();
    let right = transform.right();
    transform.translation += controller.velocity.x * dt * right
        + controller.velocity.y * dt * Vec3::Y
        + controller.velocity.z * dt * forward;

    // Handle mouse input
    let mouse_input = {
        let mut mouse_delta = Vec2::ZERO;
        let enabled = match controller.mouse_key_enable_mouse {
            Some(button) => mouse.pressed(button),
            None => true,
        };

        if enabled {
            for mouse_event in mouse_evs.read() {
                mouse_delta += mouse_event.delta;
            }
        }
        mouse_delta
    };

    if mouse_input != Vec2::ZERO {
        controller.pitch = (controller.pitch - mouse_input.y * 0.5 * controller.sensitivity * dt)
            .clamp(-PI / 2., PI / 2.);
        controller.yaw -= mouse_input.x * controller.sensitivity * dt;
        transform.rotation = Quat::from_euler(EulerRot::ZYX, 0.0, controller.yaw, controller.pitch);
    }
}

use bevy::window::{CursorGrabMode, PrimaryWindow, WindowFocused};
use bevy::{input::mouse::MouseMotion, prelude::*};
use std::f32::consts::PI;

#[derive(Component)]
pub struct FreeCameraTag;

#[derive(Event)]
pub struct CursorGrabEvent(pub Entity, pub bool);

#[derive(Default)]
pub struct FreeCameraPlugin;

impl Plugin for FreeCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CursorGrabEvent>()
            .add_systems(PreStartup, (setup, ui::setup))
            .add_systems(
                Update,
                (
                    camera_controller,
                    check_cursor_grab,
                    apply_cursor_grab.after(check_cursor_grab),
                    ui::toggle_crosshair_focus,
                    ui::cursor_recenter,
                ),
            );
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((
        FreeCameraTag,
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 0.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        CameraController::default(),
    ));
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
            mouse_key_enable_mouse: None,
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
    mut mouse_events: EventReader<MouseMotion>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    key_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &mut CameraController), With<Camera>>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok(window) = primary_window.get_single() else {
        return;
    };
    let Ok((mut transform, mut controller)) = query.get_single_mut() else {
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
        if key_input.pressed(controller.key_forward) { axis_input.z += 1.0; }
        if key_input.pressed(controller.key_back)    { axis_input.z -= 1.0; }
        if key_input.pressed(controller.key_right)   { axis_input.x += 1.0; }
        if key_input.pressed(controller.key_left)    { axis_input.x -= 1.0; }
        if key_input.pressed(controller.key_up)      { axis_input.y += 1.0; }
        if key_input.pressed(controller.key_down)    { axis_input.y -= 1.0; }
        axis_input
    };

    if axis_input != Vec3::ZERO {
        let max_speed = key_input
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
        let enabled = window.cursor.grab_mode == CursorGrabMode::Locked
            && match controller.mouse_key_enable_mouse {
                Some(button) => mouse_button_input.pressed(button),
                None => true,
            };

        if enabled {
            for mouse_event in mouse_events.read() {
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

fn check_cursor_grab(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut primary_window: Query<Entity, With<PrimaryWindow>>,
    mut evs: EventReader<WindowFocused>,
    mut grab_evs: EventWriter<CursorGrabEvent>,
    mut ever_left_clicked: Local<bool>,
) {
    let Ok(ent) = primary_window.get_single_mut() else {
        return;
    };
    // Tt appears the browser sends a window focused event after we change the grab?
    // that's pretty counter-productive. This tries to fix it doesn't really work.
    let force_unfocus = keys.just_pressed(KeyCode::Escape);

    if !force_unfocus {
        if mouse.just_pressed(MouseButton::Left) {
            grab_evs.send(CursorGrabEvent(ent, true));
            *ever_left_clicked = true;
        }

        for ev in evs.read() {
            if ev.window != ent {
                warn!("focus event on another window");
                continue;
            }
            // Tries to fix a weird behaviour in the browser that doesn't want us to
            // snatch the focus on the very first left click. But doesn't really work.
            if !*ever_left_clicked {
                continue;
            }
            grab_evs.send(CursorGrabEvent(ent, ev.focused));
        }
    } else {
        grab_evs.send(CursorGrabEvent(ent, false));
    }
}

fn apply_cursor_grab(mut grab_evs: EventReader<CursorGrabEvent>, mut window: Query<&mut Window>) {
    for CursorGrabEvent(ent, on) in grab_evs.read() {
        let Ok(mut window) = window.get_mut(*ent) else {
            continue;
        };
        match on {
            true => {
                window.cursor.grab_mode = CursorGrabMode::Locked;
                window.cursor.visible = false;
            }
            false => {
                window.cursor.grab_mode = CursorGrabMode::None;
                window.cursor.visible = true;
            }
        }
    }
}

pub mod ui {
    use bevy::{
        prelude::*,
        ui::Val,
        window::{CursorGrabMode, PrimaryWindow},
    };

    use super::CursorGrabEvent;

    #[derive(Component)]
    pub struct Crosshair;

    pub fn setup(mut cmd: Commands, ass: Res<AssetServer>) {
        let crosshair = ass.load("tooling/scene_preview/crosshair.png");

        cmd.spawn((
            Crosshair,
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..Default::default()
                },
                ..Default::default()
            },
        ))
        .with_children(|p| {
            p.spawn(ImageBundle {
                image: UiImage::new(crosshair.clone()),
                style: Style {
                    width: Val::Px(64.0),
                    height: Val::Px(64.0),
                    ..Default::default()
                },
                ..Default::default()
            });
        });
    }

    pub fn toggle_crosshair_focus(
        mut evs: EventReader<CursorGrabEvent>,
        mut crosshair: Query<&mut Visibility, With<Crosshair>>,
    ) {
        let Ok(mut visibility) = crosshair.get_single_mut() else {
            warn!("Couldn't find crosshair.");
            return;
        };
        for CursorGrabEvent(_, on) in evs.read() {
            *visibility = if *on {
                Visibility::Visible
            } else {
                Visibility::Hidden
            }
        }
    }
    // https://bevy-cheatbook.github.io/window/mouse-grab.html
    // according to the book, this is only necessary for windows, but web also seems to have problems centering...
    pub fn cursor_recenter(mut q_windows: Query<&mut Window, With<PrimaryWindow>>) {
        let mut primary_window = q_windows.single_mut();
        if primary_window.cursor.grab_mode == CursorGrabMode::Locked {
            let center = Vec2::new(primary_window.width() / 2.0, primary_window.height() / 2.0);
            primary_window.set_cursor_position(Some(center));
        }
    }
}

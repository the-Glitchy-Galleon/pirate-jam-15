use bevy::{
    input::{mouse::MouseMotion, InputSystem},
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow, WindowFocused},
};

#[cfg(target_family = "wasm")]
use bevy_pointerlockchange_hook::{PointerLockChangePlugin, PointerLockChangedByBrowser};

use super::global_ui_state::GlobalUiState;

#[derive(Default)]
pub struct LogicalCursorPlugin {
    pub target_grab_mode: Option<(CursorGrabMode, bool)>,
}

impl Plugin for LogicalCursorPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LogicalCursor::new(self.target_grab_mode))
            .init_resource::<CurrentPrimaryWindowFocus>()
            .add_event::<CursorModeChanged>()
            .add_systems(
                PreUpdate,
                (
                    update_primary_window_focus.after(InputSystem),
                    apply_grab.after(update_primary_window_focus),
                    apply_ungrab.after(apply_grab),
                    update_position.after(apply_ungrab),
                ),
            );

        #[cfg(target_family = "wasm")]
        app.add_plugins(PointerLockChangePlugin);
    }
}

/// cursor position for the primary window
/// will be centered if the window focus was locked.
#[derive(Resource)]
pub struct LogicalCursor {
    pub position: Option<Vec2>,
    pub delta: Vec2,
    last_physical_position: Option<Vec2>,
    last_grab_mode: CursorGrabMode,
    pub target_grab_mode: Option<(CursorGrabMode, bool)>,
}

impl LogicalCursor {
    pub fn new(target_grab_mode: Option<(CursorGrabMode, bool)>) -> Self {
        Self {
            position: None,
            delta: Vec2::ZERO,
            last_physical_position: None,
            last_grab_mode: CursorGrabMode::None,
            target_grab_mode,
        }
    }
}

pub fn update_position(
    window: Query<&Window, With<PrimaryWindow>>,
    mut cursor: ResMut<LogicalCursor>,
    mut cursor_ev: EventReader<MouseMotion>,
    mut changed: EventWriter<CursorModeChanged>,
) {
    let window = window.single();

    let re_entered_window = cursor.last_physical_position.is_none() && cursor.position.is_some();
    let realign = cursor.last_grab_mode != window.cursor.grab_mode || re_entered_window;

    if realign {
        changed.send(CursorModeChanged {
            mode: window.cursor.grab_mode,
            position: match window.cursor.grab_mode {
                CursorGrabMode::None => window.cursor_position(),
                _ => cursor.position,
            },
        });
        cursor.last_grab_mode = window.cursor.grab_mode;
    }

    match window.cursor.grab_mode {
        // FPS mode. Todo: test before use
        CursorGrabMode::Locked => {
            let center = Vec2::new(window.width() / 2.0, window.height() / 2.0);
            cursor.delta = Vec2::ZERO;
            for ev in cursor_ev.read() {
                cursor.delta += ev.delta;
            }
            cursor.position = Some(center);
        }
        // Custom Cursor Mode
        CursorGrabMode::Confined => {
            cursor.delta = Vec2::ZERO;
            for ev in cursor_ev.read() {
                cursor.delta += ev.delta;
            }

            if let Some(pos) = cursor.position {
                cursor.position = Some(Vec2::new(
                    f32::clamp(pos.x + cursor.delta.x, 0.0, window.width()),
                    f32::clamp(pos.y + cursor.delta.y, 0.0, window.height()),
                ));
            }
        }
        // Physical Cursor Mode
        CursorGrabMode::None => {
            let current_position = window.cursor_position();
            cursor.delta = Vec2::ZERO;
            if !realign {
                if let Some(old_pos) = cursor.last_physical_position {
                    if let Some(new_pos) = current_position {
                        cursor.delta = new_pos - old_pos;
                    }
                }
            }
            cursor.position = current_position;
        }
    };

    cursor.last_physical_position = cursor.position;
}

#[derive(Event)]
pub struct CursorModeChanged {
    pub mode: CursorGrabMode,
    pub position: Option<Vec2>,
}

#[cfg(target_family = "wasm")]
pub fn apply_ungrab(
    mut window: Query<&mut Window, With<PrimaryWindow>>,
    mut changed: EventReader<PointerLockChangedByBrowser>,
    cursor: Res<LogicalCursor>,
) {
    let mut window = window.single_mut();

    // don't update the state ourselves, pray for the plugin to come through.
    if let Some(changed) = changed.read().last() {
        if changed.is_pointer_locked {
            if let Some((mode, vis)) = cursor.target_grab_mode {
                window.cursor.grab_mode = mode;
                window.cursor.visible = vis;
            } else {
                // something else locked it somehow
                warn!("Something caused the pointer to lock");
                window.cursor.grab_mode = CursorGrabMode::Confined;
                window.cursor.visible = false;
            }
        } else {
            window.cursor.grab_mode = CursorGrabMode::None;
            window.cursor.visible = true;
        }
    }
}

#[cfg(not(target_family = "wasm"))]
pub fn apply_ungrab(
    mut window: Query<&mut Window, With<PrimaryWindow>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let mut window = window.single_mut();
    if keys.just_pressed(KeyCode::Escape) {
        window.cursor.grab_mode = CursorGrabMode::None;
        window.cursor.visible = true;
    }
}

pub fn apply_grab(
    mut window: Query<&mut Window, With<PrimaryWindow>>,
    cursor: Res<LogicalCursor>,
    mouse: Res<ButtonInput<MouseButton>>,
    focus: Res<CurrentPrimaryWindowFocus>,
    ui_state: Res<GlobalUiState>,
) {
    if !ui_state.is_pointer_over_ui {
        let mut window = window.single_mut();

        if let Some((mode, vis)) = cursor.target_grab_mode {
            if focus.is_focused && mouse.just_pressed(MouseButton::Left) {
                window.cursor.grab_mode = mode;
                window.cursor.visible = vis;
            }
        }
    }
}

#[derive(Resource, Default)]
pub struct CurrentPrimaryWindowFocus {
    is_focused: bool,
}

pub fn update_primary_window_focus(
    mut focus: ResMut<CurrentPrimaryWindowFocus>,
    window: Query<Entity, With<PrimaryWindow>>,
    mut last_focused: EventReader<WindowFocused>,
) {
    let window = window.single();
    let last_focused = last_focused
        .read()
        .filter(|e| e.window == window)
        .last()
        .map(|e| e.focused);

    match last_focused {
        None => {}
        Some(true) => focus.is_focused = true,
        Some(false) => focus.is_focused = false,
    }
}

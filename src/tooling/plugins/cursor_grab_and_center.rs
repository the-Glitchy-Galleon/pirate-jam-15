use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow, WindowFocused};

use super::pointer_capture_check::IsPointerOverUi;

#[derive(Event)]
pub struct CursorGrabEvent(pub Entity, pub bool);

/// Depends on `PointerCaptureCheckPlugin`
#[derive(Default)]
pub struct CursorGrabAndCenterPlugin;

impl Plugin for CursorGrabAndCenterPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CursorGrabEvent>().add_systems(
            Update,
            (
                check_cursor_grab,
                apply_cursor_grab.after(check_cursor_grab),
                cursor_recenter,
            ),
        );
    }
}

fn check_cursor_grab(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut primary_window: Query<Entity, With<PrimaryWindow>>,
    mut evs: EventReader<WindowFocused>,
    mut grab_evs: EventWriter<CursorGrabEvent>,
    mut ever_left_clicked: Local<bool>,
    is_pointer_over_ui: Res<IsPointerOverUi>,
) {
    let Ok(ent) = primary_window.get_single_mut() else {
        return;
    };
    if is_pointer_over_ui.0 {
        return;
    }

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

// https://bevy-cheatbook.github.io/window/mouse-grab.html
// according to the book, this is only necessary for windows, but web also seems to have problems centering...
pub fn cursor_recenter(mut q_windows: Query<&mut Window, With<PrimaryWindow>>) {
    let mut primary_window = q_windows.single_mut();
    if primary_window.cursor.grab_mode == CursorGrabMode::Locked {
        let center = Vec2::new(primary_window.width() / 2.0, primary_window.height() / 2.0);
        primary_window.set_cursor_position(Some(center));
    }
}

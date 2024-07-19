use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow, WindowFocused};

use super::pointer_capture_check::IsPointerOverUi;

/// Depends on `PointerCaptureCheckPlugin`
#[derive(Default)]
pub struct CursorGrabAndCenterPlugin;

impl Plugin for CursorGrabAndCenterPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentPrimaryWindowFocus>()
            .add_systems(
                PreUpdate,
                (
                    update_current_window_focus,
                    apply_cursor_grab.after(update_current_window_focus),
                ),
            );
    }
}

#[derive(Resource, Default)]
struct CurrentPrimaryWindowFocus(bool);

fn update_current_window_focus(
    mut focus: ResMut<CurrentPrimaryWindowFocus>,
    mut primary_window: Query<Entity, With<PrimaryWindow>>,
    mut evs: EventReader<WindowFocused>,
) {
    let Ok(ent) = primary_window.get_single_mut() else {
        return;
    };

    let last_focused = evs
        .read()
        .filter(|e| e.window == ent)
        .last()
        .map(|e| e.focused);

    match last_focused {
        None => {}
        Some(true) => focus.0 = true,
        Some(false) => focus.0 = false,
    }
}

fn apply_cursor_grab(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut window: Query<&mut Window, With<PrimaryWindow>>,
    is_pointer_over_ui: Res<IsPointerOverUi>,
    focus: Res<CurrentPrimaryWindowFocus>,
) {
    let Ok(mut window) = window.get_single_mut() else {
        return;
    };

    // Todo: there's still a bug that causes the first escape press not to work.
    // the second press seems consistent
    if keys.just_pressed(KeyCode::Escape) {
        window.cursor.grab_mode = CursorGrabMode::None;
        window.cursor.visible = true;
    } else {
        if !is_pointer_over_ui.0 && mouse.just_pressed(MouseButton::Left) && focus.0 {
            window.cursor.grab_mode = CursorGrabMode::Locked;
            window.cursor.visible = false;
        }
    }
}

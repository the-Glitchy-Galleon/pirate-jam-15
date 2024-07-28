use bevy::{
    input::InputSystem,
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};

#[derive(Default)]
pub struct LogicalCursorPlugin;

impl Plugin for LogicalCursorPlugin {
    fn build(&self, app: &mut App) {
        // todo: this should ideally happen after `apply_cursor_grab`,
        // but i didn't want to introduce a plugin dependency
        app.init_resource::<LogicalCursor>()
            .add_systems(PreUpdate, update.after(InputSystem));
    }
}

/// cursor position for the primary window
/// will be centered if the window focus was grabbed.
/// might be able to get extended if we want to grab the cursor,
/// and confine our own logical cursor position inside the game
#[derive(Resource, Default)]
pub struct LogicalCursor {
    pub position: Option<Vec2>,
}

pub fn update(window: Query<&Window, With<PrimaryWindow>>, mut cursor: ResMut<LogicalCursor>) {
    let Ok(window) = window.get_single() else {
        return;
    };
    let center = Vec2::new(window.width() / 2.0, window.height() / 2.0);
    cursor.position = match window.cursor_position() {
        None => None,
        Some(pos) => match window.cursor.grab_mode {
            CursorGrabMode::Locked => Some(center),
            CursorGrabMode::Confined => Some(center),
            CursorGrabMode::None => Some(pos),
        },
    }
}

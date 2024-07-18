use bevy::{
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};

#[derive(Default)]
pub struct LogicalCursorPlugin;

impl Plugin for LogicalCursorPlugin {
    fn build(&self, app: &mut App) {
        // todo: this should ideally happen after `apply_cursor_grab`,
        // but i didn't want to introduce a plugin dependency
        app.init_resource::<LogicalCursorPosition>()
            .add_systems(PreUpdate, update_logical_cursor_position);
    }
}

/// cursor position for the primary window
/// will be centered if the window focus was grabbed.
/// might be able to get extended if we want to grab the cursor,
/// and confine our own logical cursor position inside the game
#[derive(Resource, Default)]
pub struct LogicalCursorPosition(pub Option<Vec2>);

fn update_logical_cursor_position(
    window: Query<&Window, With<PrimaryWindow>>,
    mut position: ResMut<LogicalCursorPosition>,
) {
    let Ok(window) = window.get_single() else {
        return;
    };
    let center = Vec2::new(window.width() / 2.0, window.height() / 2.0);
    position.0 = match window.cursor_position() {
        None => None,
        Some(pos) => match window.cursor.grab_mode {
            CursorGrabMode::Locked => Some(center),
            CursorGrabMode::Confined => Some(center),
            CursorGrabMode::None => Some(pos),
        },
    }
}

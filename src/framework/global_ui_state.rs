use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::EguiContexts;

#[derive(Default)]
pub struct GlobalUiStatePlugin;

impl Plugin for GlobalUiStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GlobalUiState>()
            .add_systems(PreUpdate, update_pointer_capture_var);
    }
}

#[derive(Resource, Default)]
pub struct GlobalUiState {
    pub is_pointer_captured: bool,
    pub is_any_focused: bool,
}

/// Marks a bevy UI node as click-through
#[derive(Component)]
pub struct NoPointerCapture;

/// Checks if the pointer is over either a bevy UI node, or egui element
/// Note that this doesn't do anything on its own.
/// Game events need to consider `IsPointerOverUi` before taking actions
pub fn update_pointer_capture_var(
    windows: Query<(Entity, &mut Window), With<PrimaryWindow>>,
    mut global_ui_state: ResMut<GlobalUiState>,
    mut ctxs: EguiContexts,
    nodes: Query<(&Node, &GlobalTransform, &ViewVisibility), Without<NoPointerCapture>>,
) {
    let Ok((ent, window)) = windows.get_single() else {
        return;
    };

    let Some(cursor_pos) = window.cursor_position() else {
        global_ui_state.is_pointer_captured = false;
        return;
    };

    let mut is_over_ui = false;

    let ctx = ctxs.ctx_for_window_mut(ent);

    // See: https://github.com/jakobhellermann/bevy-inspector-egui/issues/108
    //
    // Todo: this doesn't work if the cursor is barely outside of an egui element, but would still trigger a resize for example
    if ctx.is_pointer_over_area() {
        is_over_ui = true;
    }
    let is_ui_focused = ctx.memory(|i| i.focused().is_some());

    // if let Some(ctx) = ctxs.try_ctx_for_window_mut(ent) {
    //     if ctx.is_pointer_over_area() {
    //         is_over_ui = true;
    //     }
    // }

    if !is_over_ui {
        // See: https://www.reddit.com/r/bevy/comments/vbll6b/comment/ic94hgt
        // See: https://github.com/bevyengine/bevy/issues/3570#issuecomment-1548929099
        for (node, transform, vis) in nodes.iter() {
            if vis.get() {
                let size = node.size();
                let translation = transform.translation();
                let node_position = translation.xy();
                let half_size = 0.5 * size;
                let min = node_position - half_size;
                let max = node_position + half_size;
                if (min.x..max.x).contains(&cursor_pos.x) && (min.y..max.y).contains(&cursor_pos.y)
                {
                    is_over_ui = true;
                    break;
                }
            }
        }
    }

    if global_ui_state.is_pointer_captured != is_over_ui {
        // maybe fire an event here if needed later
    }
    global_ui_state.is_pointer_captured = is_over_ui;
    global_ui_state.is_any_focused = is_ui_focused;
}

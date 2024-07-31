use bevy::{prelude::*, window::PrimaryWindow};

#[cfg(feature = "debug_visuals")]
use bevy_egui::EguiContexts;

#[derive(Default)]
pub struct GlobalUiStatePlugin;

impl Plugin for GlobalUiStatePlugin {
    #[cfg(not(feature = "debug_visuals"))]
    fn build(&self, app: &mut App) {
        app.init_resource::<GlobalUiState>()
            .add_systems(PreUpdate, update_pointer_capture);
    }

    #[cfg(feature = "debug_visuals")]
    fn build(&self, app: &mut App) {
        app.init_resource::<GlobalUiState>().add_systems(
            PreUpdate,
            (
                update_egui_capture,
                update_pointer_capture.after(update_egui_capture),
            ),
        );
    }
}

#[derive(Resource, Default)]
pub struct GlobalUiState {
    pub is_pointer_over_ui: bool,

    pub is_pointer_over_egui: bool,
    pub is_egui_input_focused: bool,
    pub is_pointer_over_nodes: bool,
}

/// Marks a bevy UI node as click-through
#[derive(Component)]
pub struct NoPointerCapture;

/// Checks if the pointer is over either a bevy UI node, or egui element
/// Note that this doesn't do anything on its own.
/// Game events need to consider `IsPointerOverUi` before taking actions
pub fn update_pointer_capture(
    windows: Query<&mut Window, With<PrimaryWindow>>,
    mut state: ResMut<GlobalUiState>,
    nodes: Query<(&Node, &GlobalTransform, &ViewVisibility), Without<NoPointerCapture>>,
) {
    let Ok(window) = windows.get_single() else {
        return;
    };

    let Some(cursor_pos) = window.cursor_position() else {
        state.is_pointer_over_ui = false;
        return;
    };

    state.is_pointer_over_nodes = false;

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
            if (min.x..max.x).contains(&cursor_pos.x) && (min.y..max.y).contains(&cursor_pos.y) {
                state.is_pointer_over_nodes = true;
                break;
            }
        }
    }

    state.is_pointer_over_ui = state.is_pointer_over_nodes || state.is_pointer_over_egui;
}

// separated because the context kept making a fuzz
#[cfg(feature = "debug_visuals")]
pub fn update_egui_capture(
    windows: Query<Entity, With<PrimaryWindow>>,
    mut state: ResMut<GlobalUiState>,
    mut ctxs: EguiContexts,
) {
    let Ok(ent) = windows.get_single() else {
        return;
    };

    state.is_pointer_over_egui = false;
    state.is_egui_input_focused = false;

    if let Some(ctx) = ctxs.try_ctx_for_window_mut(ent) {
        // See: https://github.com/jakobhellermann/bevy-inspector-egui/issues/108
        //
        // Todo: this doesn't work if the cursor is barely outside of an egui element, but would still trigger a resize for example
        if ctx.is_pointer_over_area() {
            state.is_pointer_over_egui = true;
        }
        state.is_egui_input_focused = ctx.memory(|i| i.focused().is_some());
    }
}

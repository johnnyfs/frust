//! Client-specific terminal UI composition.

use ratatui::layout::Rect;

use crate::{
    app::{AppMessage, AppState},
    tui::{self, ViewTree},
};

mod area_name_box;
mod inspector;
mod palette;
mod party_status_box;
mod viewport;

/// Composes the full client UI for the current frame.
pub fn compose(state: &AppState, area: Rect) -> ViewTree<AppState, AppMessage> {
    let mut root = tui::root(area).child(viewport::view(state, area));

    if state.edit_mode() {
        // Edit mode shows the terrain palette and hides the explore/battle
        // overlays (party status, tile inspector).
        root = root.child(area_name_box::view(state, area));
        root.push_child(palette::view(state, area));
    } else {
        root = root
            .child(party_status_box::view(state, area))
            .child(area_name_box::view(state, area));
        if let Some(node) = inspector::view(state, area) {
            root = root.child(node);
        }
    }

    ViewTree::new(root)
}

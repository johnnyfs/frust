//! Client-specific terminal UI composition.

use ratatui::layout::Rect;

use crate::{
    app::{AppMessage, AppState},
    tui::{self, ViewTree},
};

mod area_name_box;
mod party_status_box;
mod viewport;

/// Composes the full client UI for the current frame.
pub fn compose(state: &AppState, area: Rect) -> ViewTree<AppState, AppMessage> {
    ViewTree::new(
        tui::root(area)
            .child(viewport::view(state, area))
            .child(party_status_box::view(state, area))
            .child(area_name_box::view(state, area)),
    )
}

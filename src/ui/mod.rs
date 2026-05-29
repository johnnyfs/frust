//! Client-specific terminal UI composition.

use ratatui::layout::Rect;

use crate::{
    app::{AppMessage, AppState},
    tui::{self, ViewTree},
};

mod area_name_box;
mod printable_background;

/// Composes the full client UI for the current frame.
pub fn compose(state: &AppState, area: Rect) -> ViewTree<AppState, AppMessage> {
    ViewTree::new(
        tui::root(area)
            .child(printable_background::view(area))
            .child(area_name_box::view(state, area)),
    )
}

//! Render traversal.

use ratatui::Frame;

use crate::tui::ViewTree;

/// Renders a composed tree in deterministic layer/z/insertion order.
pub fn render_tree<S: 'static, M: 'static>(
    tree: &ViewTree<S, M>,
    frame: &mut Frame<'_>,
    state: &S,
) {
    for node in tree.render_order() {
        node.view.render(frame, node.rect, state);
    }
}

//! Small composition helpers.

use ratatui::layout::Rect;

use crate::tui::{EventResult, InputPolicy, Layer, UiEvent, View, ViewId, ViewNode};

/// Creates a root node for a composed frame.
pub fn root<S: 'static, M: 'static>(area: Rect) -> ViewNode<S, M> {
    ViewNode::new(RootView::new("root"), area)
}

/// Minimal root/default view.
pub struct RootView {
    id: ViewId,
}

impl RootView {
    /// Creates a root view with an id.
    pub fn new(id: impl Into<ViewId>) -> Self {
        Self { id: id.into() }
    }
}

impl<S, M> View<S, M> for RootView {
    fn id(&self) -> ViewId {
        self.id.clone()
    }

    fn input_policy(&self) -> InputPolicy {
        InputPolicy::HitTest
    }

    fn layer(&self) -> Layer {
        Layer::Base
    }

    fn render(&self, _frame: &mut ratatui::Frame<'_>, _area: Rect, _state: &S) {}

    fn handle_event(
        &self,
        _event: &UiEvent,
        _area: Rect,
        _state: &S,
        _focus: &crate::tui::FocusState,
    ) -> EventResult<M> {
        EventResult::Ignored
    }
}

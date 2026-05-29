//! View trait and input result types.

use ratatui::{Frame, layout::Rect};

use crate::tui::{FocusState, Layer, UiEvent, ViewId};

/// How a view participates in input routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputPolicy {
    /// Rendered only. Never receives input.
    None,
    /// Receives mouse events when hit-tested.
    HitTest,
    /// Can receive keyboard focus and focused keyboard events.
    Focusable,
    /// Receives keyboard events while it is the active keyboard target.
    CaptureKeyboard,
    /// Captures mouse drag/up events after mouse down.
    CaptureMouse,
    /// Modal-style capture. Gets first chance at all events while active.
    CaptureAll,
}

impl InputPolicy {
    /// Returns true if this policy can receive point-based mouse events.
    pub fn can_hit_test(self) -> bool {
        matches!(
            self,
            Self::HitTest
                | Self::Focusable
                | Self::CaptureKeyboard
                | Self::CaptureMouse
                | Self::CaptureAll
        )
    }

    /// Returns true if this policy can receive keyboard focus.
    pub fn can_focus(self) -> bool {
        matches!(
            self,
            Self::Focusable | Self::CaptureKeyboard | Self::CaptureAll
        )
    }

    /// Returns true if mouse down should establish mouse capture.
    pub fn captures_mouse(self) -> bool {
        matches!(self, Self::CaptureMouse | Self::CaptureAll)
    }

    /// Returns true if the view is a modal capture target.
    pub fn captures_all(self) -> bool {
        matches!(self, Self::CaptureAll)
    }
}

/// Result returned by a view after an input event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventResult<M> {
    /// Event consumed with emitted application messages.
    Handled(Vec<M>),
    /// This view did not handle the event.
    Ignored,
    /// Offer the event to this view's parent.
    Bubble,
}

impl<M> EventResult<M> {
    /// Creates a handled result with one message.
    pub fn message(message: M) -> Self {
        Self::Handled(vec![message])
    }

    /// Returns true if the event was handled.
    pub fn is_handled(&self) -> bool {
        matches!(self, Self::Handled(_))
    }
}

/// Stateless renderer/event hook over application state.
pub trait View<S, M>: 'static {
    /// Stable id for focus, hit testing, modal identity, and tests.
    fn id(&self) -> ViewId;

    /// Input participation policy.
    fn input_policy(&self) -> InputPolicy {
        InputPolicy::None
    }

    /// Semantic layer.
    fn layer(&self) -> Layer {
        Layer::Base
    }

    /// Numeric z offset within `layer`.
    fn z_offset(&self) -> i32 {
        0
    }

    /// Render this view into its owned rectangle.
    fn render(&self, frame: &mut Frame<'_>, area: Rect, state: &S);

    /// Handle an input event.
    fn handle_event(
        &self,
        _event: &UiEvent,
        _area: Rect,
        _state: &S,
        _focus: &FocusState,
    ) -> EventResult<M> {
        EventResult::Ignored
    }
}

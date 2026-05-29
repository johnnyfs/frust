//! Custom view hook.

use ratatui::{Frame, layout::Rect};

use crate::tui::{EventResult, FocusState, InputPolicy, Layer, UiEvent, View, ViewId};

type RenderHook<S> = dyn Fn(&mut Frame<'_>, Rect, &S);
type EventHook<S, M> = dyn Fn(&UiEvent, Rect, &S, &FocusState) -> EventResult<M>;

/// A custom render/event hook view.
pub struct CustomView<S, M> {
    id: ViewId,
    input_policy: InputPolicy,
    layer: Layer,
    z_offset: i32,
    render: Box<RenderHook<S>>,
    event: Option<Box<EventHook<S, M>>>,
}

impl<S, M> CustomView<S, M> {
    /// Creates a custom view from a render hook.
    pub fn new(id: impl Into<ViewId>, render: impl Fn(&mut Frame<'_>, Rect, &S) + 'static) -> Self {
        Self {
            id: id.into(),
            input_policy: InputPolicy::None,
            layer: Layer::Base,
            z_offset: 0,
            render: Box::new(render),
            event: None,
        }
    }

    /// Sets input policy.
    pub fn input_policy(mut self, policy: InputPolicy) -> Self {
        self.input_policy = policy;
        self
    }

    /// Sets layer.
    pub fn layer(mut self, layer: Layer) -> Self {
        self.layer = layer;
        self
    }

    /// Sets z offset.
    pub fn z_offset(mut self, z_offset: i32) -> Self {
        self.z_offset = z_offset;
        self
    }

    /// Sets event hook.
    pub fn on_event(
        mut self,
        event: impl Fn(&UiEvent, Rect, &S, &FocusState) -> EventResult<M> + 'static,
    ) -> Self {
        self.event = Some(Box::new(event));
        self
    }
}

impl<S: 'static, M: 'static> View<S, M> for CustomView<S, M> {
    fn id(&self) -> ViewId {
        self.id.clone()
    }

    fn input_policy(&self) -> InputPolicy {
        self.input_policy
    }

    fn layer(&self) -> Layer {
        self.layer
    }

    fn z_offset(&self) -> i32 {
        self.z_offset
    }

    fn render(&self, frame: &mut Frame<'_>, area: Rect, state: &S) {
        (self.render)(frame, area, state);
    }

    fn handle_event(
        &self,
        event: &UiEvent,
        area: Rect,
        state: &S,
        focus: &FocusState,
    ) -> EventResult<M> {
        self.event
            .as_ref()
            .map(|handler| handler(event, area, state, focus))
            .unwrap_or(EventResult::Ignored)
    }
}

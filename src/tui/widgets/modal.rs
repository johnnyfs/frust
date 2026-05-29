//! Modal primitive.

use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::Style,
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::tui::{EventResult, FocusState, InputPolicy, Layer, UiEvent, View, ViewId};

/// Modal overlay with optional close command.
pub struct Modal<M> {
    id: ViewId,
    title: Option<String>,
    body: String,
    close_message: Option<M>,
    clear: bool,
    style: Style,
    policy: InputPolicy,
    z_offset: i32,
}

impl<M> Modal<M> {
    /// Creates a modal with body text.
    pub fn new(id: impl Into<ViewId>, body: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: None,
            body: body.into(),
            close_message: None,
            clear: true,
            style: Style::default(),
            policy: InputPolicy::CaptureAll,
            z_offset: 0,
        }
    }

    /// Sets title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets close message emitted on Escape.
    pub fn close_message(mut self, message: M) -> Self {
        self.close_message = Some(message);
        self
    }

    /// Enables or disables background clearing.
    pub fn clear(mut self, clear: bool) -> Self {
        self.clear = clear;
        self
    }

    /// Sets style.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Sets input policy.
    pub fn input_policy(mut self, policy: InputPolicy) -> Self {
        self.policy = policy;
        self
    }

    /// Sets z offset.
    pub fn z_offset(mut self, z_offset: i32) -> Self {
        self.z_offset = z_offset;
        self
    }

    /// Computes a centered rectangle within an area.
    pub fn centered(area: Rect, width: u16, height: u16) -> Rect {
        let [vertical] = Layout::vertical([Constraint::Length(height)])
            .flex(ratatui::layout::Flex::Center)
            .areas(area);
        let [horizontal] = Layout::horizontal([Constraint::Length(width)])
            .flex(ratatui::layout::Flex::Center)
            .areas(vertical);
        horizontal
    }
}

impl<S, M: Clone + 'static> View<S, M> for Modal<M> {
    fn id(&self) -> ViewId {
        self.id.clone()
    }

    fn input_policy(&self) -> InputPolicy {
        self.policy
    }

    fn layer(&self) -> Layer {
        Layer::Modal
    }

    fn z_offset(&self) -> i32 {
        self.z_offset
    }

    fn render(&self, frame: &mut Frame<'_>, area: Rect, _state: &S) {
        if self.clear {
            frame.render_widget(Clear, area);
        }
        let mut block = Block::default().borders(Borders::ALL).style(self.style);
        if let Some(title) = &self.title {
            block = block.title(title.as_str());
        }
        frame.render_widget(Paragraph::new(self.body.clone()).block(block), area);
    }

    fn handle_event(
        &self,
        event: &UiEvent,
        _area: Rect,
        _state: &S,
        _focus: &FocusState,
    ) -> EventResult<M> {
        match event {
            UiEvent::Key(key) if key.kind == KeyEventKind::Press && key.code == KeyCode::Esc => {
                self.close_message
                    .clone()
                    .map(EventResult::message)
                    .unwrap_or(EventResult::Handled(Vec::new()))
            }
            _ if self.policy == InputPolicy::CaptureAll => EventResult::Handled(Vec::new()),
            _ => EventResult::Ignored,
        }
    }
}

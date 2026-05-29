//! Scroll view primitive.

use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    widgets::{Block, Borders, Paragraph},
};

use crate::{EventResult, FocusState, InputPolicy, MouseKind, UiEvent, View, ViewId};

/// Messages emitted by `ScrollView`.
#[derive(Debug, Clone)]
pub struct ScrollMessages<M> {
    /// One line up.
    pub line_up: Option<M>,
    /// One line down.
    pub line_down: Option<M>,
    /// One page up.
    pub page_up: Option<M>,
    /// One page down.
    pub page_down: Option<M>,
}

impl<M> Default for ScrollMessages<M> {
    fn default() -> Self {
        Self {
            line_up: None,
            line_down: None,
            page_up: None,
            page_down: None,
        }
    }
}

/// Stateless scrollable text viewport.
pub struct ScrollView<M> {
    id: ViewId,
    content: String,
    scroll_y: u16,
    scroll_x: u16,
    messages: ScrollMessages<M>,
    input_policy: InputPolicy,
    style: Style,
    borders: bool,
}

impl<M> ScrollView<M> {
    /// Creates a scroll view. Scroll offsets are supplied by app state at
    /// composition time.
    pub fn new(id: impl Into<ViewId>, content: impl Into<String>, scroll_y: u16) -> Self {
        Self {
            id: id.into(),
            content: content.into(),
            scroll_y,
            scroll_x: 0,
            messages: ScrollMessages::default(),
            input_policy: InputPolicy::HitTest,
            style: Style::default(),
            borders: false,
        }
    }

    /// Sets horizontal scroll.
    pub fn scroll_x(mut self, scroll_x: u16) -> Self {
        self.scroll_x = scroll_x;
        self
    }

    /// Sets emitted messages.
    pub fn messages(mut self, messages: ScrollMessages<M>) -> Self {
        self.messages = messages;
        self
    }

    /// Sets input policy.
    pub fn input_policy(mut self, policy: InputPolicy) -> Self {
        self.input_policy = policy;
        self
    }

    /// Sets style.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Enables or disables borders.
    pub fn borders(mut self, borders: bool) -> Self {
        self.borders = borders;
        self
    }
}

impl<S, M: Clone + 'static> View<S, M> for ScrollView<M> {
    fn id(&self) -> ViewId {
        self.id.clone()
    }

    fn input_policy(&self) -> InputPolicy {
        self.input_policy
    }

    fn render(&self, frame: &mut Frame<'_>, area: Rect, _state: &S) {
        let mut paragraph = Paragraph::new(self.content.clone())
            .scroll((self.scroll_y, self.scroll_x))
            .style(self.style);
        if self.borders {
            paragraph = paragraph.block(Block::default().borders(Borders::ALL));
        }
        frame.render_widget(paragraph, area);
    }

    fn handle_event(
        &self,
        event: &UiEvent,
        _area: Rect,
        _state: &S,
        _focus: &FocusState,
    ) -> EventResult<M> {
        let message = match event {
            UiEvent::Mouse(mouse) if mouse.kind == MouseKind::ScrollUp => {
                self.messages.line_up.clone()
            }
            UiEvent::Mouse(mouse) if mouse.kind == MouseKind::ScrollDown => {
                self.messages.line_down.clone()
            }
            UiEvent::Key(key) if key.kind == KeyEventKind::Press && key.code == KeyCode::Up => {
                self.messages.line_up.clone()
            }
            UiEvent::Key(key) if key.kind == KeyEventKind::Press && key.code == KeyCode::Down => {
                self.messages.line_down.clone()
            }
            UiEvent::Key(key) if key.kind == KeyEventKind::Press && key.code == KeyCode::PageUp => {
                self.messages.page_up.clone()
            }
            UiEvent::Key(key)
                if key.kind == KeyEventKind::Press && key.code == KeyCode::PageDown =>
            {
                self.messages.page_down.clone()
            }
            _ => None,
        };

        message
            .map(EventResult::message)
            .unwrap_or(EventResult::Ignored)
    }
}

//! Tabs primitive.

use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::Line,
    widgets::{Block, Borders, Tabs as RatatuiTabs},
};

use crate::tui::{EventResult, FocusState, InputPolicy, MouseKind, UiEvent, View, ViewId};

/// Tab labels with click/key selection messages.
pub struct Tabs<M> {
    id: ViewId,
    labels: Vec<String>,
    selected: usize,
    select_messages: Vec<Option<M>>,
    previous_message: Option<M>,
    next_message: Option<M>,
    input_policy: InputPolicy,
    style: Style,
    selected_style: Style,
    borders: bool,
}

impl<M> Tabs<M> {
    /// Creates tabs with selected index.
    pub fn new(id: impl Into<ViewId>, labels: Vec<String>, selected: usize) -> Self {
        let select_messages = (0..labels.len()).map(|_| None).collect();
        Self {
            id: id.into(),
            labels,
            selected,
            select_messages,
            previous_message: None,
            next_message: None,
            input_policy: InputPolicy::HitTest,
            style: Style::default(),
            selected_style: Style::default(),
            borders: false,
        }
    }

    /// Sets per-tab selection messages.
    pub fn select_messages(mut self, messages: Vec<Option<M>>) -> Self {
        self.select_messages = messages;
        self
    }

    /// Sets keyboard previous/next messages.
    pub fn key_messages(mut self, previous: Option<M>, next: Option<M>) -> Self {
        self.previous_message = previous;
        self.next_message = next;
        self
    }

    /// Sets input policy.
    pub fn input_policy(mut self, policy: InputPolicy) -> Self {
        self.input_policy = policy;
        self
    }

    /// Sets base style.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Sets selected style.
    pub fn selected_style(mut self, style: Style) -> Self {
        self.selected_style = style;
        self
    }

    /// Enables or disables borders.
    pub fn borders(mut self, borders: bool) -> Self {
        self.borders = borders;
        self
    }

    fn tab_at_x(&self, area: Rect, x: u16) -> Option<usize> {
        let mut cursor = area.x;
        let offset = if self.borders { 1 } else { 0 };
        cursor = cursor.saturating_add(offset);
        for (index, label) in self.labels.iter().enumerate() {
            let width = label.chars().count() as u16 + 2;
            if x >= cursor && x < cursor.saturating_add(width) {
                return Some(index);
            }
            cursor = cursor.saturating_add(width);
        }
        None
    }
}

impl<S, M: Clone + 'static> View<S, M> for Tabs<M> {
    fn id(&self) -> ViewId {
        self.id.clone()
    }

    fn input_policy(&self) -> InputPolicy {
        self.input_policy
    }

    fn render(&self, frame: &mut Frame<'_>, area: Rect, _state: &S) {
        let labels = self
            .labels
            .iter()
            .map(|label| Line::from(format!(" {label} ")))
            .collect::<Vec<_>>();
        let mut tabs = RatatuiTabs::new(labels)
            .select(self.selected)
            .style(self.style)
            .highlight_style(self.selected_style);
        if self.borders {
            tabs = tabs.block(Block::default().borders(Borders::ALL));
        }
        frame.render_widget(tabs, area);
    }

    fn handle_event(
        &self,
        event: &UiEvent,
        area: Rect,
        _state: &S,
        _focus: &FocusState,
    ) -> EventResult<M> {
        let message = match event {
            UiEvent::Mouse(mouse)
                if mouse.kind == MouseKind::Down && mouse.position.is_inside(area) =>
            {
                self.tab_at_x(area, mouse.position.x)
                    .and_then(|index| self.select_messages.get(index))
                    .cloned()
                    .flatten()
            }
            UiEvent::Key(key) if key.kind == KeyEventKind::Press && key.code == KeyCode::Left => {
                self.previous_message.clone()
            }
            UiEvent::Key(key) if key.kind == KeyEventKind::Press && key.code == KeyCode::Right => {
                self.next_message.clone()
            }
            _ => None,
        };

        message
            .map(EventResult::message)
            .unwrap_or(EventResult::Ignored)
    }
}

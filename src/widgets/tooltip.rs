//! Tooltip primitive.

use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::{InputPolicy, Layer, View, ViewId};

/// Small top-layer hover box.
pub struct Tooltip {
    id: ViewId,
    text: String,
    style: Style,
    clear: bool,
    z_offset: i32,
}

impl Tooltip {
    /// Creates a tooltip.
    pub fn new(id: impl Into<ViewId>, text: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            style: Style::default(),
            clear: true,
            z_offset: 0,
        }
    }

    /// Sets style.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Enables or disables clearing.
    pub fn clear(mut self, clear: bool) -> Self {
        self.clear = clear;
        self
    }

    /// Sets z offset.
    pub fn z_offset(mut self, z_offset: i32) -> Self {
        self.z_offset = z_offset;
        self
    }

    /// Places a tooltip near an anchor while keeping it in bounds when possible.
    pub fn near(anchor: crate::Point, size: (u16, u16), bounds: Rect) -> Rect {
        let mut x = anchor.x.saturating_add(1);
        let mut y = anchor.y.saturating_add(1);
        if x.saturating_add(size.0) > bounds.x.saturating_add(bounds.width) {
            x = bounds.x.saturating_add(bounds.width).saturating_sub(size.0);
        }
        if y.saturating_add(size.1) > bounds.y.saturating_add(bounds.height) {
            y = bounds
                .y
                .saturating_add(bounds.height)
                .saturating_sub(size.1);
        }
        Rect::new(x, y, size.0, size.1)
    }
}

impl<S, M> View<S, M> for Tooltip {
    fn id(&self) -> ViewId {
        self.id.clone()
    }

    fn input_policy(&self) -> InputPolicy {
        InputPolicy::None
    }

    fn layer(&self) -> Layer {
        Layer::Tooltip
    }

    fn z_offset(&self) -> i32 {
        self.z_offset
    }

    fn render(&self, frame: &mut Frame<'_>, area: Rect, _state: &S) {
        if self.clear {
            frame.render_widget(Clear, area);
        }
        let block = Block::default().borders(Borders::ALL).style(self.style);
        frame.render_widget(Paragraph::new(self.text.clone()).block(block), area);
    }
}

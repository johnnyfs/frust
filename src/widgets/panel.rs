//! Panel primitive.

use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    widgets::{Block, Borders, Clear, Widget},
};

use crate::{EventResult, FocusState, InputPolicy, Layer, UiEvent, View, ViewId};

/// Bordered or unbordered rectangular region.
pub struct Panel {
    id: ViewId,
    title: Option<String>,
    borders: bool,
    style: Style,
    clear: bool,
    layer: Layer,
    z_offset: i32,
    input_policy: InputPolicy,
}

impl Panel {
    /// Creates a panel.
    pub fn new(id: impl Into<ViewId>) -> Self {
        Self {
            id: id.into(),
            title: None,
            borders: true,
            style: Style::default(),
            clear: false,
            layer: Layer::Base,
            z_offset: 0,
            input_policy: InputPolicy::None,
        }
    }

    /// Sets the title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Enables or disables borders.
    pub fn borders(mut self, borders: bool) -> Self {
        self.borders = borders;
        self
    }

    /// Sets style.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Clears the background before rendering.
    pub fn clear(mut self, clear: bool) -> Self {
        self.clear = clear;
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

    /// Sets input policy.
    pub fn input_policy(mut self, policy: InputPolicy) -> Self {
        self.input_policy = policy;
        self
    }
}

impl<S, M> View<S, M> for Panel {
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

    fn render(&self, frame: &mut Frame<'_>, area: Rect, _state: &S) {
        if self.clear {
            frame.render_widget(Clear, area);
        }

        let borders = if self.borders {
            Borders::ALL
        } else {
            Borders::NONE
        };
        let mut block = Block::default().borders(borders).style(self.style);
        if let Some(title) = &self.title {
            block = block.title(title.as_str());
        }
        block.render(area, frame.buffer_mut());
    }

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

//! Canvas-like cell grid primitive.

use ratatui::{Frame, layout::Rect, style::Style};

use crate::tui::{InputPolicy, Layer, Point, View, ViewId};

/// A single drawable cell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridCell {
    /// Character/glyph.
    pub ch: char,
    /// Cell style.
    pub style: Style,
}

impl Default for GridCell {
    fn default() -> Self {
        Self {
            ch: ' ',
            style: Style::default(),
        }
    }
}

/// Rectangular direct-cell drawing surface.
pub struct CellGrid {
    id: ViewId,
    width: u16,
    height: u16,
    cells: Vec<GridCell>,
    input_policy: InputPolicy,
    layer: Layer,
    z_offset: i32,
}

impl CellGrid {
    /// Creates a blank grid.
    pub fn new(id: impl Into<ViewId>, width: u16, height: u16) -> Self {
        Self {
            id: id.into(),
            width,
            height,
            cells: vec![GridCell::default(); width as usize * height as usize],
            input_policy: InputPolicy::HitTest,
            layer: Layer::Base,
            z_offset: 0,
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

    /// Sets a cell when in bounds.
    pub fn set_cell(mut self, x: u16, y: u16, ch: char, style: Style) -> Self {
        if let Some(cell) = self.cell_mut(x, y) {
            *cell = GridCell { ch, style };
        }
        self
    }

    /// Converts a screen point to a local grid point.
    pub fn screen_to_local(area: Rect, point: Point) -> Option<Point> {
        point.is_inside(area).then(|| Point {
            x: point.x - area.x,
            y: point.y - area.y,
        })
    }

    /// Converts a local grid point to a screen point.
    pub fn local_to_screen(area: Rect, point: Point) -> Point {
        Point {
            x: area.x.saturating_add(point.x),
            y: area.y.saturating_add(point.y),
        }
    }

    fn cell_mut(&mut self, x: u16, y: u16) -> Option<&mut GridCell> {
        if x < self.width && y < self.height {
            self.cells
                .get_mut(y as usize * self.width as usize + x as usize)
        } else {
            None
        }
    }
}

impl<S, M> View<S, M> for CellGrid {
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
        let width = self.width.min(area.width);
        let height = self.height.min(area.height);
        let buffer = frame.buffer_mut();
        for y in 0..height {
            for x in 0..width {
                let cell = &self.cells[y as usize * self.width as usize + x as usize];
                if let Some(buffer_cell) = buffer.cell_mut((area.x + x, area.y + y)) {
                    buffer_cell.set_char(cell.ch).set_style(cell.style);
                }
            }
        }
    }
}

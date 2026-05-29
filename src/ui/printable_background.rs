use ratatui::{layout::Rect, style::Style};

use crate::{
    app::{AppMessage, AppState},
    tui::{InputPolicy, Layer, ViewNode, widgets::CellGrid},
};

const PRINTABLE_START: u8 = b' ';
const N_PRINTABLE_CHARS: u16 = 95;

pub fn view(area: Rect) -> ViewNode<AppState, AppMessage> {
    ViewNode::new(
        printable_grid(area.width, area.height)
            .input_policy(InputPolicy::None)
            .layer(Layer::Base)
            .z_offset(i32::MIN),
        area,
    )
}

fn printable_grid(width: u16, height: u16) -> CellGrid {
    let mut grid = CellGrid::new("printable-background", width, height);
    for y in 0..height {
        for x in 0..width {
            let index = (y * width + x) % N_PRINTABLE_CHARS;
            let ch = (PRINTABLE_START + index as u8) as char;
            grid = grid.set_cell(x, y, ch, Style::default());
        }
    }
    grid
}

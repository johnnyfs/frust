use ratatui::{
    layout::Rect,
    style::{Color, Style},
};

use crate::{
    app::{AppMessage, AppState},
    data::grid::Vector,
    tui::{InputPolicy, Layer, ViewNode, widgets::CellGrid},
    view::worldview::WorldViewTerrain,
};

pub fn view(state: &AppState, area: Rect) -> ViewNode<AppState, AppMessage> {
    let world_view = state.world_view(Vector {
        x: area.width as i32,
        y: area.height as i32,
    });

    let mut grid = CellGrid::new("viewport", area.width, area.height)
        .input_policy(InputPolicy::None)
        .layer(Layer::Base);
    let grass_style = Style::default().fg(Color::LightGreen);

    for y in 0..area.height {
        for x in 0..area.width {
            if world_view.terrain.get(x as usize, y as usize) == Some(&WorldViewTerrain::Grass) {
                grid = grid.set_cell(x, y, '.', grass_style);
            }
        }
    }

    ViewNode::new(grid, area)
}

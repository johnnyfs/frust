use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
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
    let shrubbery_style = Style::default().fg(Color::Rgb(0, 100, 0));
    let player_style = Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::BOLD);

    for y in 0..area.height {
        for x in 0..area.width {
            match world_view.terrain.get(x as usize, y as usize) {
                Some(WorldViewTerrain::Grass) => {
                    grid = grid.set_cell(x, y, '.', grass_style);
                }
                Some(WorldViewTerrain::Shrubbery) => {
                    grid = grid.set_cell(x, y, '*', shrubbery_style);
                }
                Some(WorldViewTerrain::Blank) | None => {}
            }
        }
    }

    if area.width > 0 && area.height > 0 {
        grid = grid.set_cell(area.width / 2, area.height / 2, '@', player_style);
    }

    ViewNode::new(grid, area)
}

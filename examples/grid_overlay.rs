use frust::{
    InputPolicy, Layer, Point, ViewNode, ViewTree,
    widgets::{CellGrid, Panel, Tooltip},
};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
};

#[derive(Default)]
struct AppState {
    hover: Option<Point>,
}

#[derive(Debug, Clone)]
enum Msg {}

fn compose(state: &AppState, area: Rect) -> ViewTree<AppState, Msg> {
    let grid_area = Rect::new(
        1,
        1,
        area.width.saturating_sub(22),
        area.height.saturating_sub(2),
    );
    let detail_area = Rect::new(
        area.width.saturating_sub(20),
        1,
        19,
        area.height.saturating_sub(2),
    );

    let mut root = frust::root(area)
        .child(ViewNode::new(
            CellGrid::new("grid", grid_area.width, grid_area.height)
                .input_policy(InputPolicy::CaptureMouse)
                .set_cell(2, 2, '*', Style::default().fg(Color::Cyan)),
            grid_area,
        ))
        .child(ViewNode::new(
            Panel::new("details").title("Details"),
            detail_area,
        ));

    if let Some(point) = state.hover {
        let tooltip_area = Tooltip::near(point, (18, 3), area);
        root = root.overlay(ViewNode::new(
            Tooltip::new("cell-tooltip", format!("cell {},{}", point.x, point.y)).z_offset(10),
            tooltip_area,
        ));
    }

    root = root.overlay(ViewNode::new(
        Panel::new("path-overlay")
            .borders(false)
            .layer(Layer::Overlay),
        Rect::new(grid_area.x + 1, grid_area.y + 1, 10, 1),
    ));

    ViewTree::new(root)
}

fn main() {
    let state = AppState {
        hover: Some(Point::new(5, 5)),
    };
    let tree = compose(&state, Rect::new(0, 0, 80, 24));
    assert!(tree.hit_test(Point::new(3, 3)).is_some());
}

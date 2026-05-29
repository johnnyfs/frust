use frust::{
    InputPolicy, Layer, ViewNode, ViewTree, render_tree,
    widgets::{Panel, Tooltip},
};
use ratatui::{
    Terminal,
    backend::TestBackend,
    layout::Rect,
    style::{Color, Style},
};

#[derive(Default)]
struct AppState {
    show_popover: bool,
}

#[derive(Debug, Clone)]
enum Msg {}

fn compose(state: &AppState, area: Rect) -> ViewTree<AppState, Msg> {
    let left = Rect::new(area.x, area.y, area.width / 3, area.height);
    let main = Rect::new(
        area.width / 3,
        area.y,
        area.width - area.width / 3,
        area.height,
    );
    let popover = Rect::new(10, 3, 24, 5);

    ViewTree::new(
        frust::root(area)
            .child(ViewNode::new(Panel::new("metrics").title("Metrics"), left))
            .child(ViewNode::new(
                Panel::new("activity").title("Activity"),
                main,
            ))
            .modal_if(
                state.show_popover,
                ViewNode::new(
                    Panel::new("popover")
                        .title("Popover")
                        .layer(Layer::Overlay)
                        .input_policy(InputPolicy::HitTest)
                        .clear(true),
                    popover,
                ),
            )
            .overlay(ViewNode::new(
                Tooltip::new("hint", "hover detail").style(Style::default().fg(Color::Yellow)),
                Rect::new(4, 1, 16, 3),
            )),
    )
}

fn main() -> std::io::Result<()> {
    let state = AppState { show_popover: true };
    let mut terminal = Terminal::new(TestBackend::new(60, 18))?;
    terminal.draw(|frame| {
        let tree = compose(&state, frame.area());
        render_tree(&tree, frame, &state);
    })?;
    Ok(())
}

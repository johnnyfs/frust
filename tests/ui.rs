use frust::{
    app::AppState,
    tui::{ViewId, render_tree},
    ui,
};
use ratatui::{Terminal, backend::TestBackend, layout::Rect, style::Color};

#[test]
fn composed_ui_uses_viewport_instead_of_printable_background() {
    let state = AppState::default();
    let tree = ui::compose(&state, Rect::new(0, 0, 20, 8));

    assert!(tree.find(&ViewId::new("viewport")).is_some());
    assert!(tree.find(&ViewId::new("printable-background")).is_none());
}

#[test]
fn viewport_renders_grass_as_light_green_dots() {
    let state = AppState::default();
    let mut terminal = Terminal::new(TestBackend::new(20, 8)).unwrap();

    terminal
        .draw(|frame| {
            let tree = ui::compose(&state, frame.area());
            render_tree(&tree, frame, &state);
        })
        .unwrap();

    let cell = terminal.backend().buffer().cell((0, 0)).unwrap();
    assert_eq!(cell.symbol(), ".");
    assert_eq!(cell.fg, Color::LightGreen);
}

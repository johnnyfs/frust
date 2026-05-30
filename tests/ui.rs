use frust::{
    app::AppState,
    tui::{ViewId, render_tree},
    ui,
};
use ratatui::{
    Terminal,
    backend::TestBackend,
    layout::Rect,
    style::{Color, Modifier},
};

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

#[test]
fn viewport_renders_shrubbery_as_dark_green_asterisks() {
    let state = AppState::default();
    let mut terminal = Terminal::new(TestBackend::new(40, 20)).unwrap();

    terminal
        .draw(|frame| {
            let tree = ui::compose(&state, frame.area());
            render_tree(&tree, frame, &state);
        })
        .unwrap();

    let buffer = terminal.backend().buffer();
    let shrubbery = buffer
        .content()
        .iter()
        .find(|cell| cell.symbol() == "*" && cell.fg == Color::Rgb(0, 100, 0));
    assert!(shrubbery.is_some());
}

#[test]
fn viewport_renders_player_at_center_as_bright_white_at_sign() {
    let state = AppState::default();
    let mut terminal = Terminal::new(TestBackend::new(20, 8)).unwrap();

    terminal
        .draw(|frame| {
            let tree = ui::compose(&state, frame.area());
            render_tree(&tree, frame, &state);
        })
        .unwrap();

    let cell = terminal.backend().buffer().cell((10, 4)).unwrap();
    assert_eq!(cell.symbol(), "@");
    assert_eq!(cell.fg, Color::White);
    assert!(cell.modifier.contains(Modifier::BOLD));
}

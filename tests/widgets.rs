use crossterm::event::KeyModifiers;
use frust::{
    InputPolicy, MouseEvent, MouseKind, Point, UiEvent, ViewNode, ViewTree, render_tree,
    widgets::{CellGrid, ScrollMessages, ScrollView, Tabs},
};
use ratatui::{
    Terminal,
    backend::TestBackend,
    layout::Rect,
    style::{Color, Style},
};

#[derive(Debug, Clone, PartialEq, Eq)]
enum Msg {
    Up,
    Down,
    Tab(usize),
}

fn wheel(kind: MouseKind, x: u16, y: u16) -> UiEvent {
    UiEvent::Mouse(MouseEvent {
        position: Point::new(x, y),
        kind,
        button: None,
        modifiers: KeyModifiers::NONE,
    })
}

#[test]
fn scroll_view_emits_wheel_messages_and_clips_content() {
    let messages = ScrollMessages {
        line_up: Some(Msg::Up),
        line_down: Some(Msg::Down),
        page_up: None,
        page_down: None,
    };
    let tree = ViewTree::new(frust::root(Rect::new(0, 0, 8, 2)).child(ViewNode::new(
        ScrollView::new("scroll", "one\ntwo\nthree", 1).messages(messages),
        Rect::new(0, 0, 8, 2),
    )));

    let outcome = frust::route_event(
        &wheel(MouseKind::ScrollDown, 1, 1),
        &tree,
        &(),
        &frust::FocusState::default(),
    );
    assert_eq!(outcome.messages, vec![Msg::Down]);

    let mut terminal = Terminal::new(TestBackend::new(8, 2)).unwrap();
    terminal
        .draw(|frame| render_tree(&tree, frame, &()))
        .unwrap();
    assert_eq!(
        terminal.backend().buffer().cell((0, 0)).unwrap().symbol(),
        "t"
    );
    assert_eq!(
        terminal.backend().buffer().cell((0, 1)).unwrap().symbol(),
        "t"
    );
}

#[test]
fn tabs_emit_click_message() {
    let tree = ViewTree::new(
        frust::root(Rect::new(0, 0, 20, 1)).child(ViewNode::new(
            Tabs::new("tabs", vec!["One".into(), "Two".into()], 0)
                .select_messages(vec![Some(Msg::Tab(0)), Some(Msg::Tab(1))]),
            Rect::new(0, 0, 20, 1),
        )),
    );

    let outcome = frust::route_event(
        &UiEvent::Mouse(MouseEvent {
            position: Point::new(6, 0),
            kind: MouseKind::Down,
            button: None,
            modifiers: KeyModifiers::NONE,
        }),
        &tree,
        &(),
        &frust::FocusState::default(),
    );
    assert_eq!(outcome.messages, vec![Msg::Tab(1)]);
}

#[test]
fn cell_grid_renders_custom_cells_and_converts_coordinates() {
    let grid = CellGrid::new("grid", 4, 2)
        .input_policy(InputPolicy::HitTest)
        .set_cell(1, 1, 'X', Style::default().fg(Color::Red));
    let area = Rect::new(2, 3, 4, 2);
    let tree = ViewTree::new(
        frust::root::<(), Msg>(Rect::new(0, 0, 10, 6)).child(ViewNode::new(grid, area)),
    );

    assert_eq!(
        CellGrid::screen_to_local(area, Point::new(3, 4)),
        Some(Point::new(1, 1))
    );
    assert_eq!(
        CellGrid::local_to_screen(area, Point::new(1, 1)),
        Point::new(3, 4)
    );

    let mut terminal = Terminal::new(TestBackend::new(10, 6)).unwrap();
    terminal
        .draw(|frame| render_tree(&tree, frame, &()))
        .unwrap();
    let cell = terminal.backend().buffer().cell((3, 4)).unwrap();
    assert_eq!(cell.symbol(), "X");
    assert_eq!(cell.fg, Color::Red);
}

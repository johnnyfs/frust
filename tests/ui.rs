use frust::{
    app::{AppMessage, AppState},
    tui::{
        FocusState, MouseButton, MouseEvent, MouseKind, Point, UiEvent, ViewId, render_tree,
        route_event,
    },
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
fn viewport_renders_party_leader_at_center_as_colored_at_sign() {
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
    assert_eq!(cell.fg, Color::Cyan);
    assert_eq!(cell.bg, Color::Reset);
    assert!(cell.modifier.contains(Modifier::BOLD));
}

#[test]
fn battle_mode_renders_focus_character_with_white_box() {
    let mut state = AppState::default();
    frust::ecs::start_encounter(&mut state.ecs_world, frust::ecs::SQUIRREL_ENCOUNTER_ID);
    let size = frust::data::grid::Vector { x: 80, y: 40 };
    let mut terminal = Terminal::new(TestBackend::new(size.x as u16, size.y as u16)).unwrap();

    terminal
        .draw(|frame| {
            let tree = ui::compose(&state, frame.area());
            render_tree(&tree, frame, &state);
        })
        .unwrap();

    let focus_cell = state.viewport_focus_cell(size).unwrap();
    let cell = terminal
        .backend()
        .buffer()
        .cell((focus_cell.x as u16, focus_cell.y as u16))
        .unwrap();
    assert_eq!(cell.bg, Color::White);
}

#[test]
fn viewport_renders_sign_as_brown_vertical_post() {
    let state = AppState::default();
    let mut terminal = Terminal::new(TestBackend::new(20, 8)).unwrap();

    terminal
        .draw(|frame| {
            let tree = ui::compose(&state, frame.area());
            render_tree(&tree, frame, &state);
        })
        .unwrap();

    let cell = terminal.backend().buffer().cell((14, 5)).unwrap();
    assert_eq!(cell.symbol(), "|");
    assert_eq!(cell.fg, Color::Rgb(139, 69, 19));
}

#[test]
fn viewport_left_click_emits_walk_destination() {
    let state = AppState::default();
    let area = Rect::new(0, 0, 20, 8);
    let tree = ui::compose(&state, area);

    let outcome = route_event(
        &UiEvent::Mouse(MouseEvent {
            position: Point::new(12, 5),
            kind: MouseKind::Down,
            button: Some(MouseButton::Left),
            modifiers: crossterm::event::KeyModifiers::NONE,
        }),
        &tree,
        &state,
        &FocusState::default(),
    );

    assert_eq!(
        outcome.messages,
        vec![
            AppMessage::SetViewportCursor(Some(frust::data::grid::Vector { x: 12, y: 5 })),
            AppMessage::ViewportClicked(frust::data::grid::Vector { x: 2, y: 1 })
        ]
    );
}

#[test]
fn viewport_renders_squirrels_as_gray_rodents() {
    let mut state = AppState::default();
    state
        .ecs_world
        .resource_mut::<frust::ecs::ViewFocus>()
        .center = frust::data::grid::Vector { x: 0, y: -24 };
    let mut terminal = Terminal::new(TestBackend::new(20, 8)).unwrap();

    terminal
        .draw(|frame| {
            let tree = ui::compose(&state, frame.area());
            render_tree(&tree, frame, &state);
        })
        .unwrap();

    let cell = terminal.backend().buffer().cell((8, 4)).unwrap();
    assert_eq!(cell.symbol(), "r");
    assert_eq!(cell.fg, Color::Gray);
}

#[test]
fn battle_mode_tints_area_label_light_red() {
    let mut state = AppState::default();
    frust::ecs::start_encounter(&mut state.ecs_world, frust::ecs::SQUIRREL_ENCOUNTER_ID);
    let mut terminal = Terminal::new(TestBackend::new(40, 20)).unwrap();

    terminal
        .draw(|frame| {
            let tree = ui::compose(&state, frame.area());
            render_tree(&tree, frame, &state);
        })
        .unwrap();

    let buffer = terminal.backend().buffer();
    let area_name_cell = buffer
        .content()
        .iter()
        .find(|cell| cell.symbol() == "B" && cell.fg == Color::LightRed);
    assert!(area_name_cell.is_some());
}

#[test]
fn viewport_mouse_move_sets_rollover_cursor_and_renders_highlight() {
    let mut state = AppState::default();
    let area = Rect::new(0, 0, 20, 8);
    let tree = ui::compose(&state, area);

    let outcome = route_event(
        &UiEvent::Mouse(MouseEvent {
            position: Point::new(2, 6),
            kind: MouseKind::Move,
            button: None,
            modifiers: crossterm::event::KeyModifiers::NONE,
        }),
        &tree,
        &state,
        &FocusState::default(),
    );

    for message in outcome.messages {
        frust::app::update(&mut state, message);
    }

    let mut terminal = Terminal::new(TestBackend::new(20, 8)).unwrap();
    terminal
        .draw(|frame| {
            let tree = ui::compose(&state, frame.area());
            render_tree(&tree, frame, &state);
        })
        .unwrap();

    let cell = terminal.backend().buffer().cell((2, 6)).unwrap();
    assert!(cell.modifier.contains(Modifier::REVERSED));
}

#[test]
fn viewport_renders_current_destination_as_white_map_cell() {
    let mut state = AppState::default();
    frust::app::update(
        &mut state,
        AppMessage::WalkFocusedEntityTo(frust::data::grid::Vector { x: 2, y: 1 }),
    );
    let mut terminal = Terminal::new(TestBackend::new(20, 8)).unwrap();

    terminal
        .draw(|frame| {
            let tree = ui::compose(&state, frame.area());
            render_tree(&tree, frame, &state);
        })
        .unwrap();

    let cell = terminal.backend().buffer().cell((12, 5)).unwrap();
    assert_eq!(cell.symbol(), ".");
    assert_eq!(cell.fg, Color::LightGreen);
    assert_eq!(cell.bg, Color::White);
}

#[test]
fn viewport_keeps_destination_white_under_mouse_cursor() {
    let mut state = AppState::default();
    frust::app::update(
        &mut state,
        AppMessage::WalkFocusedEntityTo(frust::data::grid::Vector { x: 2, y: 1 }),
    );
    frust::app::update(
        &mut state,
        AppMessage::SetViewportCursor(Some(frust::data::grid::Vector { x: 12, y: 5 })),
    );
    let mut terminal = Terminal::new(TestBackend::new(20, 8)).unwrap();

    terminal
        .draw(|frame| {
            let tree = ui::compose(&state, frame.area());
            render_tree(&tree, frame, &state);
        })
        .unwrap();

    let cell = terminal.backend().buffer().cell((12, 5)).unwrap();
    assert_eq!(cell.symbol(), ".");
    assert_eq!(cell.fg, Color::LightGreen);
    assert_eq!(cell.bg, Color::White);
    assert!(cell.modifier.contains(Modifier::REVERSED));
}

#[test]
fn viewport_keeps_destination_white_while_player_is_walking() {
    let mut state = AppState::default();
    frust::app::update(
        &mut state,
        AppMessage::WalkFocusedEntityTo(frust::data::grid::Vector { x: 5, y: 1 }),
    );
    frust::app::tick(&mut state);
    let mut terminal = Terminal::new(TestBackend::new(20, 8)).unwrap();

    terminal
        .draw(|frame| {
            let tree = ui::compose(&state, frame.area());
            render_tree(&tree, frame, &state);
        })
        .unwrap();

    let cell = terminal.backend().buffer().cell((14, 4)).unwrap();
    assert_eq!(cell.symbol(), ".");
    assert_eq!(cell.fg, Color::LightGreen);
    assert_eq!(cell.bg, Color::White);
}

#[test]
fn viewport_destination_box_preserves_entity_glyph_underneath() {
    let mut state = AppState::default();
    frust::app::update(
        &mut state,
        AppMessage::WalkFocusedEntityTo(frust::data::grid::Vector { x: 4, y: 1 }),
    );
    let mut terminal = Terminal::new(TestBackend::new(20, 8)).unwrap();

    terminal
        .draw(|frame| {
            let tree = ui::compose(&state, frame.area());
            render_tree(&tree, frame, &state);
        })
        .unwrap();

    let cell = terminal.backend().buffer().cell((14, 5)).unwrap();
    assert_eq!(cell.symbol(), "|");
    assert_eq!(cell.fg, Color::Rgb(139, 69, 19));
    assert_eq!(cell.bg, Color::White);
}

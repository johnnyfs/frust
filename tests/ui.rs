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
fn battle_mode_tints_area_label_border_light_red_not_text() {
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
    // The centered "Bridgeport Outskirts" label box sits near the top (y=1),
    // well above the party boxes (y>=8), so its border corner is unambiguous.
    let corner = buffer.cell((8, 1)).unwrap();
    assert_eq!(corner.symbol(), "┌");
    assert_eq!(corner.fg, Color::LightRed);

    // The label text itself is not tinted.
    let label = buffer.cell((10, 2)).unwrap();
    assert_eq!(label.symbol(), "B");
    assert_ne!(label.fg, Color::LightRed);
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

fn read_string(terminal: &Terminal<TestBackend>, x: u16, y: u16, len: u16) -> String {
    let buffer = terminal.backend().buffer();
    (x..x + len)
        .map(|column| buffer.cell((column, y)).unwrap().symbol().to_string())
        .collect()
}

#[test]
fn party_status_box_renders_at_left_edge_top() {
    let state = AppState::default();
    let mut terminal = Terminal::new(TestBackend::new(40, 40)).unwrap();

    terminal
        .draw(|frame| {
            let tree = ui::compose(&state, frame.area());
            render_tree(&tree, frame, &state);
        })
        .unwrap();

    let corner = terminal.backend().buffer().cell((2, 8)).unwrap();
    assert_eq!(corner.symbol(), "┌");
}

#[test]
fn second_party_box_starts_one_row_after_first_box_bottom() {
    let state = AppState::default();
    let mut terminal = Terminal::new(TestBackend::new(40, 40)).unwrap();

    terminal
        .draw(|frame| {
            let tree = ui::compose(&state, frame.area());
            render_tree(&tree, frame, &state);
        })
        .unwrap();

    let buffer = terminal.backend().buffer();
    // First box: top border at y=8, bottom border at y=13, one empty gap row at y=14.
    assert_eq!(buffer.cell((2, 13)).unwrap().symbol(), "└");
    // The gap row carries no box border at the border column.
    let gap = buffer.cell((2, 14)).unwrap().symbol().to_string();
    assert!(!["┌", "└", "│"].contains(&gap.as_str()));
    // Second box top border one row after the gap.
    assert_eq!(buffer.cell((2, 15)).unwrap().symbol(), "┌");
}

#[test]
fn first_party_member_name_renders_in_cyan() {
    let state = AppState::default();
    let mut terminal = Terminal::new(TestBackend::new(40, 40)).unwrap();

    terminal
        .draw(|frame| {
            let tree = ui::compose(&state, frame.area());
            render_tree(&tree, frame, &state);
        })
        .unwrap();

    let cell = terminal.backend().buffer().cell((4, 9)).unwrap();
    assert_eq!(cell.symbol(), "M");
    assert_eq!(cell.fg, Color::Cyan);
    assert_eq!(read_string(&terminal, 4, 9, 4), "Mara");
}

#[test]
fn explore_party_box_movement_row_is_blank() {
    let state = AppState::default();
    let mut terminal = Terminal::new(TestBackend::new(40, 40)).unwrap();

    terminal
        .draw(|frame| {
            let tree = ui::compose(&state, frame.area());
            render_tree(&tree, frame, &state);
        })
        .unwrap();

    // Movement row of the first box (box top y=8, movement row y=12).
    assert_eq!(terminal.backend().buffer().cell((4, 12)).unwrap().symbol(), " ");
}

#[test]
fn battle_party_box_borders_render_light_red() {
    let mut state = AppState::default();
    frust::ecs::start_encounter(&mut state.ecs_world, frust::ecs::SQUIRREL_ENCOUNTER_ID);
    let mut terminal = Terminal::new(TestBackend::new(40, 40)).unwrap();

    terminal
        .draw(|frame| {
            let tree = ui::compose(&state, frame.area());
            render_tree(&tree, frame, &state);
        })
        .unwrap();

    let buffer = terminal.backend().buffer();
    let corner = buffer.cell((2, 8)).unwrap();
    assert_eq!(corner.symbol(), "┌");
    assert_eq!(corner.fg, Color::LightRed);

    // Only the outline is tinted: the class-line text stays untinted.
    let class = buffer.cell((4, 10)).unwrap();
    assert_eq!(class.symbol(), "L");
    assert_ne!(class.fg, Color::LightRed);
}

#[test]
fn battle_active_member_movement_renders_spent_over_remaining() {
    let mut state = AppState::default();
    frust::ecs::start_encounter(&mut state.ecs_world, frust::ecs::SQUIRREL_ENCOUNTER_ID);
    let leader = state.ecs_world.resource::<frust::ecs::PartyRoster>().members[0];
    state.ecs_world.entity_mut(leader).insert(frust::ecs::ActionState {
        remaining_movement: 4,
        has_attacked: false,
    });
    let mut terminal = Terminal::new(TestBackend::new(40, 40)).unwrap();

    terminal
        .draw(|frame| {
            let tree = ui::compose(&state, frame.area());
            render_tree(&tree, frame, &state);
        })
        .unwrap();

    // Mara has speed 9; with 4m remaining she has spent 5m.
    assert_eq!(read_string(&terminal, 4, 12, 8), "Mv 5/4 m");
}

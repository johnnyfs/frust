use frust::{
    app::{self, AppMessage, AppState},
    data::{grid::Vector, world::TerrainType},
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

fn left_click(position: Point) -> UiEvent {
    UiEvent::Mouse(MouseEvent {
        position,
        kind: MouseKind::Down,
        button: Some(MouseButton::Left),
        modifiers: crossterm::event::KeyModifiers::NONE,
    })
}

fn enter_edit_mode(state: &mut AppState) {
    app::update(state, AppMessage::ToggleEditMode);
}

fn mouse_event(kind: MouseKind, button: Option<MouseButton>, position: Point) -> UiEvent {
    UiEvent::Mouse(MouseEvent {
        position,
        kind,
        button,
        modifiers: crossterm::event::KeyModifiers::NONE,
    })
}

/// Focus state with the viewport holding the mouse capture, as the router
/// establishes after a mouse-down in edit mode.
fn viewport_capture() -> FocusState {
    FocusState {
        mouse_capture: Some(ViewId::new("viewport")),
        ..FocusState::default()
    }
}

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

/// Collects each buffer row into a string for substring assertions.
fn buffer_rows(terminal: &Terminal<TestBackend>) -> Vec<String> {
    let buffer = terminal.backend().buffer();
    let area = *buffer.area();
    (0..area.height)
        .map(|y| {
            (0..area.width)
                .map(|x| buffer.cell((x, y)).unwrap().symbol().to_string())
                .collect::<String>()
        })
        .collect()
}

fn read_string(terminal: &Terminal<TestBackend>, x: u16, y: u16, len: u16) -> String {
    let buffer = terminal.backend().buffer();
    (x..x + len)
        .map(|column| buffer.cell((column, y)).unwrap().symbol().to_string())
        .collect()
}

#[test]
fn palette_appears_only_in_edit_mode() {
    let mut state = AppState::default();
    let area = Rect::new(0, 0, 40, 20);

    let tree = ui::compose(&state, area);
    assert!(tree.find(&ViewId::new("palette")).is_none());

    enter_edit_mode(&mut state);
    let tree = ui::compose(&state, area);
    assert!(tree.find(&ViewId::new("palette")).is_some());
}

#[test]
fn expanded_palette_renders_toggle_and_selected_tile() {
    let mut state = AppState::default();
    enter_edit_mode(&mut state);
    let mut terminal = Terminal::new(TestBackend::new(40, 20)).unwrap();

    terminal
        .draw(|frame| {
            let tree = ui::compose(&state, frame.area());
            render_tree(&tree, frame, &state);
        })
        .unwrap();

    let buffer = terminal.backend().buffer();
    // Toggle is in the upper-left at the palette origin (x=4, y=4).
    assert_eq!(buffer.cell((4, 4)).unwrap().symbol(), "V");
    // First terrain row is grass: a light-green dot with its name.
    let grass_glyph = buffer.cell((4, 5)).unwrap();
    assert_eq!(grass_glyph.symbol(), ".");
    assert_eq!(grass_glyph.fg, Color::LightGreen);
    assert_eq!(buffer.cell((6, 5)).unwrap().symbol(), "G");
    // The selected terrain row (grass by default) is highlighted.
    assert!(
        buffer
            .cell((6, 5))
            .unwrap()
            .modifier
            .contains(Modifier::REVERSED)
    );
}

#[test]
fn collapsed_palette_renders_arrow_and_selected_tile() {
    let mut state = AppState::default();
    enter_edit_mode(&mut state);
    app::update(&mut state, AppMessage::SelectTerrain(TerrainType::River));
    app::update(&mut state, AppMessage::TogglePaletteCollapse);
    let mut terminal = Terminal::new(TestBackend::new(40, 20)).unwrap();

    terminal
        .draw(|frame| {
            let tree = ui::compose(&state, frame.area());
            render_tree(&tree, frame, &state);
        })
        .unwrap();

    let buffer = terminal.backend().buffer();
    assert_eq!(buffer.cell((4, 4)).unwrap().symbol(), ">");
    let selected = buffer.cell((6, 4)).unwrap();
    assert_eq!(selected.symbol(), ":");
    assert_eq!(selected.fg, Color::LightCyan);
}

#[test]
fn clicking_a_palette_row_selects_that_terrain() {
    let mut state = AppState::default();
    enter_edit_mode(&mut state);
    let area = Rect::new(0, 0, 40, 20);
    let tree = ui::compose(&state, area);

    // Forest is the third terrain (id 2), rendered on the third row below the
    // toggle: palette y origin 4 + (2 + 1) = 7.
    let outcome = route_event(&left_click(Point::new(4, 7)), &tree, &state, &FocusState::default());
    assert_eq!(
        outcome.messages,
        vec![AppMessage::SelectTerrain(TerrainType::Forest)]
    );

    for message in outcome.messages {
        app::update(&mut state, message);
    }
    assert_eq!(state.selected_terrain(), TerrainType::Forest);
}

#[test]
fn clicking_the_palette_toggle_collapses_it() {
    let mut state = AppState::default();
    enter_edit_mode(&mut state);
    let area = Rect::new(0, 0, 40, 20);
    let tree = ui::compose(&state, area);

    let outcome = route_event(&left_click(Point::new(4, 4)), &tree, &state, &FocusState::default());
    assert_eq!(outcome.messages, vec![AppMessage::TogglePaletteCollapse]);
}

#[test]
fn palette_consumes_clicks_so_they_do_not_paint_through() {
    let mut state = AppState::default();
    enter_edit_mode(&mut state);
    let area = Rect::new(0, 0, 40, 20);
    let tree = ui::compose(&state, area);

    // A click on a palette row never emits a PaintTerrain message.
    let outcome = route_event(&left_click(Point::new(5, 8)), &tree, &state, &FocusState::default());
    assert!(
        outcome
            .messages
            .iter()
            .all(|message| !matches!(message, AppMessage::PaintTerrain(_)))
    );
}

#[test]
fn viewport_renders_all_terrain_glyphs_and_styles() {
    let mut state = AppState::default();
    let size = Vector { x: 40, y: 20 };
    let cells = [
        (0u16, 0u16, TerrainType::Grass, ".", Color::LightGreen),
        (1, 0, TerrainType::Shrubbery, "*", Color::Rgb(0, 100, 0)),
        (2, 0, TerrainType::Forest, "#", Color::Rgb(0, 100, 0)),
        (3, 0, TerrainType::Path, ":", Color::Rgb(139, 69, 19)),
        (0, 1, TerrainType::Road, ":", Color::DarkGray),
        (1, 1, TerrainType::River, ":", Color::LightCyan),
        (2, 1, TerrainType::Pond, "~", Color::LightCyan),
        (3, 1, TerrainType::Clearing, ":", Color::LightGreen),
    ];

    for (x, y, terrain, _, _) in cells {
        let coord = state.viewport_cell_to_world(
            size,
            Vector {
                x: x as i32,
                y: y as i32,
            },
        );
        assert!(
            state
                .ecs_world
                .resource_mut::<frust::data::world::World>()
                .set_terrain(coord, terrain)
        );
    }

    let mut terminal = Terminal::new(TestBackend::new(size.x as u16, size.y as u16)).unwrap();
    terminal
        .draw(|frame| {
            let tree = ui::compose(&state, frame.area());
            render_tree(&tree, frame, &state);
        })
        .unwrap();

    let buffer = terminal.backend().buffer();
    for (x, y, _, glyph, color) in cells {
        let cell = buffer.cell((x, y)).unwrap();
        assert_eq!(cell.symbol(), glyph, "glyph at ({x}, {y})");
        assert_eq!(cell.fg, color, "color at ({x}, {y})");
    }
}

#[test]
fn inspector_hidden_until_a_tile_is_hovered() {
    let state = AppState::default();
    let mut terminal = Terminal::new(TestBackend::new(80, 40)).unwrap();
    terminal
        .draw(|frame| {
            let tree = ui::compose(&state, frame.area());
            render_tree(&tree, frame, &state);
        })
        .unwrap();

    // No mouse hover yet: the inspector panel is not composed at all.
    assert!(buffer_rows(&terminal).iter().all(|row| !row.contains("Grass")));
}

#[test]
fn inspector_shows_terrain_and_signpost_on_hover() {
    let mut state = AppState::default();
    let size = frust::data::grid::Vector { x: 80, y: 40 };
    // Local viewport cell over the signpost at world (4, 1): origin is
    // center - size/2 = (-40, -20), so local = world - origin = (44, 21).
    frust::app::update(
        &mut state,
        AppMessage::SetViewportCursor(Some(frust::data::grid::Vector { x: 44, y: 21 })),
    );

    let mut terminal = Terminal::new(TestBackend::new(size.x as u16, size.y as u16)).unwrap();
    terminal
        .draw(|frame| {
            let tree = ui::compose(&state, frame.area());
            render_tree(&tree, frame, &state);
        })
        .unwrap();

    let rows = buffer_rows(&terminal);
    // Terrain entry followed by the signpost entry and its flavor text, with
    // each marker glyph enclosed in brackets and indented one space.
    assert!(rows.iter().any(|row| row.contains(" [.] Grass")));
    assert!(rows.iter().any(|row| row.contains(" [|] Signpost")));
    assert!(rows.iter().any(|row| row.contains("A wooden signpost")));

    // The content-sized panel opens its top border one row off the top edge.
    let buffer = terminal.backend().buffer();
    assert!(
        (0..size.x as u16).any(|x| buffer.cell((x, 1)).unwrap().symbol() == "┌"),
        "inspector top-left corner not found on the top border row"
    );
}

#[test]
fn inspector_panel_has_fixed_width_and_expands_vertically() {
    let mut state = AppState::default();
    let size = frust::data::grid::Vector { x: 80, y: 40 };
    frust::app::update(
        &mut state,
        AppMessage::SetViewportCursor(Some(frust::data::grid::Vector { x: 44, y: 21 })),
    );
    let tree = ui::compose(&state, Rect::new(0, 0, size.x as u16, size.y as u16));
    let panel = tree.find(&ViewId::new("tile-inspector")).unwrap();

    // Fixed 24-cell interior plus two borders => width 26, regardless of content.
    // Four content lines (grass heading, blank, signpost heading, signpost
    // detail) plus borders => height 6.
    assert_eq!(panel.rect.width, 26);
    assert_eq!(panel.rect.height, 6);
}

#[test]
fn inspector_border_turns_red_in_battle() {
    let mut state = AppState::default();
    frust::ecs::start_encounter(&mut state.ecs_world, frust::ecs::SQUIRREL_ENCOUNTER_ID);
    let size = frust::data::grid::Vector { x: 80, y: 40 };
    // Hover the focused combatant's tile so the inspector reports something.
    let focus = state.viewport_focus_cell(size).unwrap();
    frust::app::update(&mut state, AppMessage::SetViewportCursor(Some(focus)));

    let mut terminal = Terminal::new(TestBackend::new(size.x as u16, size.y as u16)).unwrap();
    terminal
        .draw(|frame| {
            let tree = ui::compose(&state, frame.area());
            render_tree(&tree, frame, &state);
        })
        .unwrap();

    let buffer = terminal.backend().buffer();
    let corner = (0..size.x as u16)
        .find_map(|x| {
            let cell = buffer.cell((x, 1)).unwrap();
            (cell.symbol() == "┌").then_some(cell)
        })
        .expect("inspector top-left corner present in battle");
    assert_eq!(corner.fg, Color::LightRed);
}

#[test]
fn inspector_hidden_in_edit_mode() {
    let mut state = AppState::default();
    // Hover a tile so the inspector would normally appear...
    frust::app::update(
        &mut state,
        AppMessage::SetViewportCursor(Some(frust::data::grid::Vector { x: 44, y: 21 })),
    );
    // ...then enter edit mode, which must suppress it.
    enter_edit_mode(&mut state);
    let area = Rect::new(0, 0, 80, 40);
    let tree = ui::compose(&state, area);
    assert!(tree.find(&ViewId::new("tile-inspector")).is_none());
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

#[test]
fn party_status_hidden_in_edit_mode() {
    let mut state = AppState::default();
    enter_edit_mode(&mut state);
    let area = Rect::new(0, 0, 40, 40);
    let tree = ui::compose(&state, area);
    assert!(tree.find(&ViewId::new("party-status")).is_none());
}

#[test]
fn edit_mode_left_down_paints_instead_of_walking() {
    let mut state = AppState::default();
    enter_edit_mode(&mut state);
    let area = Rect::new(0, 0, 40, 20);
    let tree = ui::compose(&state, area);

    let outcome = route_event(
        &mouse_event(MouseKind::Down, Some(MouseButton::Left), Point::new(20, 10)),
        &tree,
        &state,
        &FocusState::default(),
    );
    assert!(
        outcome
            .messages
            .iter()
            .any(|m| matches!(m, AppMessage::PaintTerrain(_)))
    );
    assert!(
        !outcome
            .messages
            .iter()
            .any(|m| matches!(m, AppMessage::ViewportClicked(_)))
    );
}

#[test]
fn edit_mode_left_drag_routes_to_paint_line() {
    let mut state = AppState::default();
    enter_edit_mode(&mut state);
    let area = Rect::new(0, 0, 40, 20);
    let tree = ui::compose(&state, area);

    let outcome = route_event(
        &mouse_event(MouseKind::Drag, Some(MouseButton::Left), Point::new(25, 12)),
        &tree,
        &state,
        &viewport_capture(),
    );
    assert!(
        outcome
            .messages
            .iter()
            .any(|m| matches!(m, AppMessage::PaintTerrainLine(_)))
    );
}

#[test]
fn edit_mode_right_drag_gesture_boxes_then_commits() {
    let mut state = AppState::default();
    enter_edit_mode(&mut state);
    let area = Rect::new(0, 0, 40, 20);
    let tree = ui::compose(&state, area);

    // Point clear of the palette overlay (x 4..15, y 4..12) and the area label.
    let down = route_event(
        &mouse_event(MouseKind::Down, Some(MouseButton::Right), Point::new(30, 15)),
        &tree,
        &state,
        &FocusState::default(),
    );
    assert!(
        down.messages
            .iter()
            .any(|m| matches!(m, AppMessage::BeginEditBox(_)))
    );

    let drag = route_event(
        &mouse_event(MouseKind::Drag, Some(MouseButton::Right), Point::new(14, 9)),
        &tree,
        &state,
        &viewport_capture(),
    );
    assert!(
        drag.messages
            .iter()
            .any(|m| matches!(m, AppMessage::ExtendEditBox(_)))
    );

    let up = route_event(
        &mouse_event(MouseKind::Up, Some(MouseButton::Right), Point::new(14, 9)),
        &tree,
        &state,
        &viewport_capture(),
    );
    assert!(up.messages.contains(&AppMessage::CommitEditBox));
}

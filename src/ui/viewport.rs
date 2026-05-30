use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
};

use crate::{
    app::{AppMessage, AppState},
    data::{grid::Vector, world::TerrainType},
    tui::{
        EventResult, FocusState, InputPolicy, Layer, MouseButton, MouseKind, UiEvent, View,
        ViewNode,
        widgets::{CellGrid, CustomView},
    },
    view::{entityview::EntityViewCell, worldview::WorldViewTerrain},
};

pub fn view(state: &AppState, area: Rect) -> ViewNode<AppState, AppMessage> {
    let input_policy = if state.edit_mode() {
        InputPolicy::CaptureMouse
    } else {
        InputPolicy::HitTest
    };
    ViewNode::new(
        CustomView::new("viewport", |frame, area, state: &AppState| {
            let grid = viewport_grid(state, area);
            <CellGrid as View<AppState, AppMessage>>::render(&grid, frame, area, state);
        })
        .input_policy(input_policy)
        .layer(Layer::Base)
        .on_event(handle_event),
        area,
    )
}

/// Glyph and style used to render a terrain tile.
pub(crate) fn terrain_cell(kind: TerrainType) -> (char, Style) {
    let (glyph, color) = crate::view::worldview::terrain_marker(kind);
    (glyph, Style::default().fg(color))
}

fn viewport_grid(state: &AppState, area: Rect) -> CellGrid {
    let size = Vector {
        x: area.width as i32,
        y: area.height as i32,
    };
    let world_view = state.world_view(size);
    let entity_view = state.entity_view(size);
    let destination = state.viewport_destination_cell(size);
    let focus = state.viewport_focus_cell(size);
    let mut grid = CellGrid::new("viewport-grid", area.width, area.height)
        .input_policy(InputPolicy::None)
        .layer(Layer::Base);

    for y in 0..area.height {
        for x in 0..area.width {
            if let Some(WorldViewTerrain::Filled(kind)) =
                world_view.terrain.get(x as usize, y as usize)
            {
                let (glyph, style) = terrain_cell(*kind);
                grid = grid.set_cell(x, y, glyph, style);
            }
        }
    }

    // Preview the pending right-drag box as the selected terrain, highlighted.
    if let Some((anchor, end)) = state.edit_box() {
        let (glyph, style) = terrain_cell(state.selected_terrain());
        let preview_style = style.add_modifier(Modifier::REVERSED);
        let (x0, x1) = (anchor.x.min(end.x), anchor.x.max(end.x));
        let (y0, y1) = (anchor.y.min(end.y), anchor.y.max(end.y));
        for world_y in y0..=y1 {
            for world_x in x0..=x1 {
                if let Some(local) =
                    state.viewport_world_to_cell(size, Vector { x: world_x, y: world_y })
                {
                    grid = grid.set_cell(local.x as u16, local.y as u16, glyph, preview_style);
                }
            }
        }
    }

    for y in 0..area.height {
        for x in 0..area.width {
            if let Some(cell) = entity_view.get(x as usize, y as usize) {
                grid = grid.set_cell(x, y, cell.glyph, entity_style(cell));
            }
        }
    }

    if let Some(focus) = focus
        && focus.x >= 0
        && focus.y >= 0
        && focus.x < area.width as i32
        && focus.y < area.height as i32
    {
        let x = focus.x as u16;
        let y = focus.y as u16;
        let (glyph, style) = rendered_cell(&world_view, &entity_view, x, y);
        grid = grid.set_cell(x, y, glyph, style);
    }

    if let Some(destination) = destination
        && destination.x >= 0
        && destination.y >= 0
        && destination.x < area.width as i32
        && destination.y < area.height as i32
    {
        let x = destination.x as u16;
        let y = destination.y as u16;
        let (glyph, style) = rendered_cell(&world_view, &entity_view, x, y);
        grid = grid.set_cell(x, y, glyph, style);
    }

    if let Some(cursor) = state.viewport_cursor()
        && cursor.x >= 0
        && cursor.y >= 0
        && cursor.x < area.width as i32
        && cursor.y < area.height as i32
    {
        let x = cursor.x as u16;
        let y = cursor.y as u16;
        let (glyph, style) = if Some(cursor) == destination || Some(cursor) == focus {
            rendered_cell(&world_view, &entity_view, x, y)
        } else {
            base_cell(&world_view, &entity_view, x, y)
        };
        grid = grid.set_cell(x, y, glyph, style.add_modifier(Modifier::REVERSED));
    }

    grid
}

fn entity_style(cell: &EntityViewCell) -> Style {
    let style = Style::default().fg(cell.color);
    if cell.bold {
        style.add_modifier(Modifier::BOLD)
    } else {
        style
    }
}

fn rendered_cell(
    world_view: &crate::view::worldview::WorldView,
    entity_view: &crate::view::entityview::EntityView,
    x: u16,
    y: u16,
) -> (char, Style) {
    let (glyph, style) = base_cell(world_view, entity_view, x, y);
    (glyph, style.bg(Color::White))
}

fn base_cell(
    world_view: &crate::view::worldview::WorldView,
    entity_view: &crate::view::entityview::EntityView,
    x: u16,
    y: u16,
) -> (char, Style) {
    entity_view
        .get(x as usize, y as usize)
        .map(|cell| (cell.glyph, entity_style(cell)))
        .or_else(|| match world_view.terrain.get(x as usize, y as usize) {
            Some(WorldViewTerrain::Filled(kind)) => Some(terrain_cell(*kind)),
            Some(WorldViewTerrain::Blank) | None => None,
        })
        .unwrap_or((' ', Style::default()))
}

fn handle_event(
    event: &UiEvent,
    area: Rect,
    state: &AppState,
    _focus: &FocusState,
) -> EventResult<AppMessage> {
    let UiEvent::Mouse(mouse) = event else {
        return EventResult::Ignored;
    };
    let size = Vector {
        x: area.width as i32,
        y: area.height as i32,
    };

    if state.edit_mode() {
        return handle_edit_event(mouse, area, size, state);
    }

    let Some(local) = CellGrid::screen_to_local(area, mouse.position) else {
        return EventResult::Ignored;
    };
    let local = Vector {
        x: local.x as i32,
        y: local.y as i32,
    };

    if mouse.kind == MouseKind::Move {
        return EventResult::message(AppMessage::SetViewportCursor(Some(local)));
    }

    if mouse.kind != MouseKind::Down || mouse.button != Some(MouseButton::Left) {
        return EventResult::Ignored;
    }

    let destination = state.viewport_cell_to_world(size, local);

    EventResult::Handled(vec![
        AppMessage::SetViewportCursor(Some(local)),
        AppMessage::ViewportClicked(destination),
    ])
}

fn handle_edit_event(
    mouse: &crate::tui::MouseEvent,
    area: Rect,
    size: Vector,
    state: &AppState,
) -> EventResult<AppMessage> {
    let screen = Vector {
        x: mouse.position.x as i32,
        y: mouse.position.y as i32,
    };

    // World coordinate under the pointer, when it is inside the viewport.
    let local_coord = CellGrid::screen_to_local(area, mouse.position).map(|local| {
        let local = Vector {
            x: local.x as i32,
            y: local.y as i32,
        };
        (local, state.viewport_cell_to_world(size, local))
    });

    match (mouse.kind, mouse.button) {
        (MouseKind::Down, Some(MouseButton::Middle)) => {
            EventResult::message(AppMessage::BeginEditPan(screen))
        }
        (MouseKind::Drag, Some(MouseButton::Middle)) => {
            EventResult::message(AppMessage::DragEditPan(screen))
        }
        (MouseKind::Down, Some(MouseButton::Left)) => {
            let Some((local, coord)) = local_coord else {
                return EventResult::Ignored;
            };
            EventResult::Handled(vec![
                AppMessage::SetViewportCursor(Some(local)),
                AppMessage::PaintTerrain(coord),
            ])
        }
        (MouseKind::Drag, Some(MouseButton::Left)) => {
            let Some((local, coord)) = local_coord else {
                return EventResult::Ignored;
            };
            EventResult::Handled(vec![
                AppMessage::SetViewportCursor(Some(local)),
                AppMessage::PaintTerrainLine(coord),
            ])
        }
        (MouseKind::Down, Some(MouseButton::Right)) => {
            let Some((local, coord)) = local_coord else {
                return EventResult::Ignored;
            };
            EventResult::Handled(vec![
                AppMessage::SetViewportCursor(Some(local)),
                AppMessage::BeginEditBox(coord),
            ])
        }
        (MouseKind::Drag, Some(MouseButton::Right)) => {
            let Some((local, coord)) = local_coord else {
                return EventResult::Ignored;
            };
            EventResult::Handled(vec![
                AppMessage::SetViewportCursor(Some(local)),
                AppMessage::ExtendEditBox(coord),
            ])
        }
        (MouseKind::Up, Some(MouseButton::Right)) => {
            EventResult::message(AppMessage::CommitEditBox)
        }
        (MouseKind::Up, _) => EventResult::message(AppMessage::EndEditStroke),
        (MouseKind::Move, _) => {
            let Some((local, _)) = local_coord else {
                return EventResult::Ignored;
            };
            EventResult::message(AppMessage::SetViewportCursor(Some(local)))
        }
        _ => EventResult::Ignored,
    }
}

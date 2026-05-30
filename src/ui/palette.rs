//! Terrain palette overlay shown only in edit mode.

use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    widgets::{Clear, Widget},
};

use crate::{
    app::{AppMessage, AppState},
    data::world::TERRAIN_TYPES,
    tui::{
        EventResult, FocusState, InputPolicy, Layer, MouseButton, MouseKind, UiEvent, ViewNode,
        widgets::CustomView,
    },
    ui::viewport::terrain_cell,
};

const PALETTE_X: u16 = 4;
const PALETTE_Y: u16 = 4;

/// Builds the palette overlay node, clamped to fit `screen`.
pub fn view(state: &AppState, screen: Rect) -> ViewNode<AppState, AppMessage> {
    let (want_width, want_height) = palette_size(state);
    let width = want_width.min(screen.width.max(1));
    let height = want_height.min(screen.height.max(1));
    let x = screen.x + PALETTE_X.min(screen.width.saturating_sub(width));
    let y = screen.y + PALETTE_Y.min(screen.height.saturating_sub(height));
    let rect = Rect::new(x, y, width, height);

    ViewNode::new(
        CustomView::new("palette", render)
            .input_policy(InputPolicy::HitTest)
            .layer(Layer::Overlay)
            .z_offset(10)
            .on_event(handle_event),
        rect,
    )
}

fn palette_size(state: &AppState) -> (u16, u16) {
    if state.palette_collapsed() {
        // `>` + space + selected glyph.
        (3, 1)
    } else {
        let longest_name = TERRAIN_TYPES
            .iter()
            .map(|terrain| terrain.name().len())
            .max()
            .unwrap_or(0);
        // glyph + space + name.
        let width = (longest_name as u16).saturating_add(2).max(1);
        let height = 1 + TERRAIN_TYPES.len() as u16;
        (width, height)
    }
}

fn render(frame: &mut ratatui::Frame<'_>, area: Rect, state: &AppState) {
    Clear.render(area, frame.buffer_mut());
    let buffer = frame.buffer_mut();

    if state.palette_collapsed() {
        write_str(buffer, area, 0, 0, ">", Style::default());
        let (glyph, style) = terrain_cell(state.selected_terrain());
        write_char(buffer, area, 2, 0, glyph, style);
        return;
    }

    write_str(buffer, area, 0, 0, "V", Style::default());

    for (index, terrain) in TERRAIN_TYPES.iter().enumerate() {
        let row = index as u16 + 1;
        if row >= area.height {
            break;
        }
        let (glyph, glyph_style) = terrain_cell(*terrain);
        let selected = *terrain == state.selected_terrain();
        let name_style = if selected {
            Style::default().add_modifier(Modifier::REVERSED)
        } else {
            Style::default()
        };
        write_char(buffer, area, 0, row, glyph, glyph_style);
        write_str(buffer, area, 2, row, terrain.name(), name_style);
    }
}

fn handle_event(
    event: &UiEvent,
    area: Rect,
    state: &AppState,
    _focus: &FocusState,
) -> EventResult<AppMessage> {
    // The palette consumes any mouse event inside its rect so clicks never
    // paint through to the viewport beneath it.
    let UiEvent::Mouse(mouse) = event else {
        return EventResult::Ignored;
    };

    if mouse.kind != MouseKind::Down || mouse.button != Some(MouseButton::Left) {
        return EventResult::Handled(Vec::new());
    }

    let row = mouse.position.y.saturating_sub(area.y);
    if row == 0 {
        return EventResult::message(AppMessage::TogglePaletteCollapse);
    }

    if !state.palette_collapsed()
        && let Some(terrain) = TERRAIN_TYPES.get((row - 1) as usize).copied()
    {
        return EventResult::message(AppMessage::SelectTerrain(terrain));
    }

    EventResult::Handled(Vec::new())
}

fn write_char(
    buffer: &mut ratatui::buffer::Buffer,
    area: Rect,
    dx: u16,
    dy: u16,
    glyph: char,
    style: Style,
) {
    let x = area.x + dx;
    let y = area.y + dy;
    if dx < area.width
        && dy < area.height
        && let Some(cell) = buffer.cell_mut((x, y))
    {
        cell.set_char(glyph).set_style(style);
    }
}

fn write_str(
    buffer: &mut ratatui::buffer::Buffer,
    area: Rect,
    dx: u16,
    dy: u16,
    text: &str,
    style: Style,
) {
    for (offset, ch) in text.chars().enumerate() {
        write_char(buffer, area, dx + offset as u16, dy, ch, style);
    }
}

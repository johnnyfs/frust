//! Clickable "end turn" prompt shown in the bottom-left during battle, on a
//! player-controlled character's turn.

use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};

use crate::{
    app::{AppMessage, AppState},
    tui::{
        EventResult, FocusState, InputPolicy, Layer, MouseButton, MouseKind, UiEvent, ViewNode,
        widgets::CustomView,
    },
};

const PROMPT: &str = "Click here or press SPACE to end turn";
const BOX_HEIGHT: u16 = 3;
/// Columns inset from the left edge.
const BOX_LEFT: u16 = 2;

/// Builds the end-turn prompt node, or `None` when it should be hidden (not a
/// player turn, or the screen is too small to place it).
pub fn view(state: &AppState, area: Rect) -> Option<ViewNode<AppState, AppMessage>> {
    if !state.is_player_turn() {
        return None;
    }

    let want_width = (PROMPT.len() as u16).saturating_add(4); // borders + one pad each side
    let box_width = want_width.min(area.width);
    let box_x = area.x.saturating_add(BOX_LEFT);

    // Leave one row between the box's bottom border and the screen's bottom edge.
    if area.height < BOX_HEIGHT + 1 || box_x.saturating_add(box_width) > area.x + area.width {
        return None;
    }
    let box_y = area.y + area.height.saturating_sub(BOX_HEIGHT + 1);
    let box_rect = Rect::new(box_x, box_y, box_width, BOX_HEIGHT);

    Some(ViewNode::new(
        CustomView::new("end-turn-box", render)
            .input_policy(InputPolicy::HitTest)
            .layer(Layer::Overlay)
            .z_offset(5)
            .on_event(handle_event),
        box_rect,
    ))
}

fn render(frame: &mut ratatui::Frame<'_>, area: Rect, _state: &AppState) {
    Clear.render(area, frame.buffer_mut());
    Block::default()
        .borders(Borders::ALL)
        .render(area, frame.buffer_mut());

    // Interior text rect: skip the border and one pad column on each side.
    let text_rect = Rect::new(
        area.x.saturating_add(2),
        area.y.saturating_add(1),
        area.width.saturating_sub(4),
        1.min(area.height.saturating_sub(2)),
    );
    if text_rect.width > 0 && text_rect.height > 0 {
        frame.render_widget(Paragraph::new(PROMPT), text_rect);
    }
}

fn handle_event(
    event: &UiEvent,
    _area: Rect,
    _state: &AppState,
    _focus: &FocusState,
) -> EventResult<AppMessage> {
    // Consume any mouse event inside the box so clicks never fall through to the
    // viewport beneath it; a left click ends the current turn.
    let UiEvent::Mouse(mouse) = event else {
        return EventResult::Ignored;
    };

    if mouse.kind == MouseKind::Down && mouse.button == Some(MouseButton::Left) {
        return EventResult::message(AppMessage::EndTurn);
    }

    EventResult::Handled(Vec::new())
}

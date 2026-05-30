use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::Paragraph,
};

use crate::{
    app::{AppMessage, AppState},
    data::grid::Vector,
    tui::{
        ViewNode,
        widgets::{CustomView, Panel},
    },
};

/// Gaps between the panel and the screen edges.
const RIGHT_GAP: u16 = 1;
const TOP_GAP: u16 = 1;
/// Fixed interior width, in cells; the panel only grows vertically.
const INTERIOR_W: u16 = 24;

/// Builds the inspector panel, or `None` when it should not be shown: when
/// nothing is hovered, the hovered tile has nothing to report, or the screen
/// is too small to hold the panel plus its edge gaps.
pub fn view(state: &AppState, area: Rect) -> Option<ViewNode<AppState, AppMessage>> {
    state.viewport_cursor()?;

    // The viewport fills the whole screen, so the cursor it stored is a local
    // cell within `area`; convert it back with the same size.
    let size = Vector {
        x: area.width as i32,
        y: area.height as i32,
    };

    let lines = inspector_lines(state, size);
    if lines.is_empty() {
        return None;
    }

    // Fixed width; the panel expands only vertically to fit its lines.
    let interior_w = INTERIOR_W;
    let interior_h = lines.len() as u16;
    let panel_w = interior_w + 2;
    let panel_h = interior_h + 2;
    if area.width < panel_w + RIGHT_GAP || area.height < panel_h + TOP_GAP {
        return None;
    }

    let panel_x = area.x + area.width - (panel_w + RIGHT_GAP);
    let panel_y = area.y + TOP_GAP;
    let panel_rect = Rect::new(panel_x, panel_y, panel_w, panel_h);
    let text_rect = Rect::new(panel_x + 1, panel_y + 1, interior_w, interior_h);

    let border_style = if state.is_turn_based() {
        Style::default().fg(Color::LightRed)
    } else {
        Style::default()
    };

    Some(
        ViewNode::new(
            Panel::new("tile-inspector")
                .borders(true)
                .clear(true)
                .border_style(border_style),
            panel_rect,
        )
        .child(ViewNode::new(
            CustomView::new(
                "tile-inspector-body",
                move |frame, area, state: &AppState| {
                    let lines = inspector_lines(state, size);
                    frame.render_widget(Paragraph::new(Text::from(lines)), area);
                },
            ),
            text_rect,
        )),
    )
}

/// Builds the inspector's display lines for the hovered tile.
fn inspector_lines(state: &AppState, size: Vector) -> Vec<Line<'static>> {
    let Some(cursor) = state.viewport_cursor() else {
        return Vec::new();
    };
    let coord = state.viewport_cell_to_world(size, cursor);
    let inspector = state.inspector_at(coord);

    let detail_style = Style::default().fg(Color::DarkGray);
    let mut lines: Vec<Line> = Vec::new();

    for (index, entry) in inspector.entries.iter().enumerate() {
        if index > 0 {
            lines.push(Line::from(""));
        }
        let (glyph, color) = entry.marker;
        // Heading: a leading space, the glyph enclosed in `[ ]`, then the name.
        lines.push(Line::from(vec![
            Span::raw(" ["),
            Span::styled(glyph.to_string(), Style::default().fg(color)),
            Span::raw("] "),
            Span::raw(entry.name.clone()),
        ]));
        for detail in &entry.details {
            lines.push(Line::styled(format!("  {detail}"), detail_style));
        }
    }

    lines
}

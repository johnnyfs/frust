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

/// Fixed inspector panel size, in terminal cells.
const PANEL_W: u16 = 28;
const PANEL_H: u16 = 16;
/// Gaps between the panel and the screen edges.
const RIGHT_GAP: u16 = 1;
const TOP_GAP: u16 = 1;

/// Builds the inspector panel, or `None` when it should not be shown:
/// when nothing is hovered yet, or the screen is too small to hold the
/// fixed panel plus its edge gaps. Editor mode (added separately) can also
/// suppress it by gating here on game mode.
pub fn view(state: &AppState, area: Rect) -> Option<ViewNode<AppState, AppMessage>> {
    state.viewport_cursor()?;
    if area.width < PANEL_W + RIGHT_GAP || area.height < PANEL_H + TOP_GAP {
        return None;
    }

    let panel_x = area.x + area.width - (PANEL_W + RIGHT_GAP);
    let panel_y = area.y + TOP_GAP;
    let panel_rect = Rect::new(panel_x, panel_y, PANEL_W, PANEL_H);
    let text_rect = Rect::new(
        panel_x + 1,
        panel_y + 1,
        PANEL_W - 2,
        PANEL_H - 2,
    );

    // The viewport fills the whole screen, so the cursor it stored is a local
    // cell within `area`; convert it back with the same size.
    let size = Vector {
        x: area.width as i32,
        y: area.height as i32,
    };

    Some(ViewNode::new(
        Panel::new("tile-inspector").borders(true).clear(true),
        panel_rect,
    )
    .child(ViewNode::new(
        CustomView::new("tile-inspector-body", move |frame, area, state: &AppState| {
            frame.render_widget(Paragraph::new(inspector_text(state, size)), area);
        }),
        text_rect,
    )))
}

fn inspector_text(state: &AppState, size: Vector) -> Text<'static> {
    let Some(cursor) = state.viewport_cursor() else {
        return Text::default();
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
        lines.push(Line::from(vec![
            Span::styled(glyph.to_string(), Style::default().fg(color)),
            Span::raw(" "),
            Span::raw(entry.name.clone()),
        ]));
        for detail in &entry.details {
            lines.push(Line::styled(format!("  {detail}"), detail_style));
        }
    }

    Text::from(lines)
}

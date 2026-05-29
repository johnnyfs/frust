use ratatui::{layout::Rect, widgets::Paragraph};

use crate::{
    app::{AppMessage, AppState},
    tui::{
        ViewNode,
        widgets::{CustomView, Panel},
    },
};

pub fn view(state: &AppState, area: Rect) -> ViewNode<AppState, AppMessage> {
    let text_width = state.current_area_name.len().try_into().unwrap_or(u16::MAX);
    let panel_width = text_width.saturating_add(4).min(area.width);
    let panel_height = 3.min(area.height);
    let panel_x = area.x + area.width.saturating_sub(panel_width) / 2;
    let panel_y = area.y + 1.min(area.height.saturating_sub(panel_height));
    let panel_rect = Rect::new(panel_x, panel_y, panel_width, panel_height);
    let text_rect = Rect::new(
        panel_x.saturating_add(2),
        panel_y.saturating_add(1),
        text_width.min(panel_width.saturating_sub(4)),
        1.min(panel_height.saturating_sub(2)),
    );

    ViewNode::new(
        Panel::new("area-name-box").borders(true).clear(true),
        panel_rect,
    )
    .child(ViewNode::new(
        CustomView::new("area-name", |frame, area, state: &AppState| {
            frame.render_widget(Paragraph::new(state.current_area_name), area);
        }),
        text_rect,
    ))
}

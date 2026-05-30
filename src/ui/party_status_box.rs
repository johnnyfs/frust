use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::Paragraph,
};

use crate::{
    app::{AppMessage, AppState},
    tui::{
        ViewNode,
        widgets::{CustomView, Panel},
    },
    view::party_status::PartyStatusMember,
};

/// Screen row of the first box's top border.
const FIRST_BOX_TOP: u16 = 8;
/// Screen column of every box's left border.
const BOX_LEFT: u16 = 2;
/// Fixed box height: border, name, class, hp, movement, border.
const BOX_HEIGHT: u16 = 6;
/// Distance between consecutive box tops (box height plus one blank row).
const BOX_STRIDE: u16 = BOX_HEIGHT + 1;
/// Minimum interior width; fits the longest starter-party content (`Lv 1 Fighter`).
const MIN_CONTENT_WIDTH: u16 = 12;

pub fn view(state: &AppState, area: Rect) -> ViewNode<AppState, AppMessage> {
    let status = state.party_status_view();
    let border_style = if state.is_turn_based() {
        Style::default().fg(Color::LightRed)
    } else {
        Style::default()
    };

    let content_width = status
        .members
        .iter()
        .map(member_content_width)
        .max()
        .unwrap_or(0)
        .max(MIN_CONTENT_WIDTH);
    let box_width = content_width.saturating_add(4); // borders + one pad column each side

    let mut container = ViewNode::new(
        CustomView::new("party-status", |_frame, _area, _state: &AppState| {}),
        area,
    );

    let box_left = area.x.saturating_add(BOX_LEFT);
    let area_right = area.x.saturating_add(area.width);
    let area_bottom = area.y.saturating_add(area.height);

    for (index, member) in status.members.iter().enumerate() {
        let box_top = area
            .y
            .saturating_add(FIRST_BOX_TOP)
            .saturating_add(BOX_STRIDE.saturating_mul(index as u16));

        // Only draw boxes that fully fit; ratatui panics on out-of-bounds rects.
        if box_left.saturating_add(box_width) > area_right
            || box_top.saturating_add(BOX_HEIGHT) > area_bottom
        {
            break;
        }

        let box_rect = Rect::new(box_left, box_top, box_width, BOX_HEIGHT);
        container.push_child(member_box(
            *member,
            box_rect,
            content_width,
            border_style,
            index,
        ));
    }

    container
}

fn member_box(
    member: PartyStatusMember,
    box_rect: Rect,
    content_width: u16,
    border_style: Style,
    index: usize,
) -> ViewNode<AppState, AppMessage> {
    let content_x = box_rect.x.saturating_add(2); // skip the border and one pad column
    let line_rect =
        |row: u16| Rect::new(content_x, box_rect.y.saturating_add(row), content_width, 1);

    let name = member.name;
    let name_color = member.name_color;
    let class = class_line(&member);
    let hp = hp_line(&member);
    let movement = movement_line(&member);

    ViewNode::new(
        Panel::new(format!("party-status-box-{index}"))
            .borders(true)
            .clear(true)
            .border_style(border_style),
        box_rect,
    )
    .child(ViewNode::new(
        CustomView::new(
            format!("party-status-name-{index}"),
            move |frame, area, _state: &AppState| {
                frame.render_widget(
                    Paragraph::new(name).style(Style::default().fg(name_color)),
                    area,
                );
            },
        ),
        line_rect(1),
    ))
    .child(ViewNode::new(
        CustomView::new(
            format!("party-status-class-{index}"),
            move |frame, area, _state: &AppState| {
                frame.render_widget(Paragraph::new(class.clone()), area);
            },
        ),
        line_rect(2),
    ))
    .child(ViewNode::new(
        CustomView::new(
            format!("party-status-hp-{index}"),
            move |frame, area, _state: &AppState| {
                frame.render_widget(Paragraph::new(hp.clone()), area);
            },
        ),
        line_rect(3),
    ))
    .child(ViewNode::new(
        CustomView::new(
            format!("party-status-move-{index}"),
            move |frame, area, _state: &AppState| {
                if let Some(line) = movement.clone() {
                    frame.render_widget(Paragraph::new(line), area);
                }
            },
        ),
        line_rect(4),
    ))
}

fn class_line(member: &PartyStatusMember) -> String {
    format!("Lv {} {}", member.level, member.class_label)
}

fn hp_line(member: &PartyStatusMember) -> String {
    format!("hp {}/{}", member.hp, member.max_hp)
}

fn movement_line(member: &PartyStatusMember) -> Option<String> {
    member
        .movement
        .map(|movement| format!("Mv {}/{} m", movement.spent, movement.remaining))
}

fn member_content_width(member: &PartyStatusMember) -> u16 {
    let mut width = member.name.len();
    width = width.max(class_line(member).len());
    width = width.max(hp_line(member).len());
    if let Some(line) = movement_line(member) {
        width = width.max(line.len());
    }
    width.try_into().unwrap_or(u16::MAX)
}

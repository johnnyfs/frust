use std::{
    io::{self},
    time::Duration,
};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use frust::{
    FocusState, InputPolicy, Layer, UiEvent, ViewNode, ViewTree, render_tree,
    route_event,
    widgets::{CellGrid, CustomView, Panel},
};
use ratatui::{Terminal, backend::CrosstermBackend, layout::Rect};

const PRINTABLE_START: u8 = b' ';
const N_PRINTABLE_CHARS: u16 = 95;
const BRIDGEPORT_OUTSKIRTS: &str = "Bridgeport Outskirts";

#[derive(Debug, Default)]
struct AppState {
    current_area_name: &'static str,
    quit: bool,
}

struct Msg {
    
}


fn main() -> io::Result<()> {
    let mut stdout = io::stdout();

    let result: io::Result<()> = (|| {
        enable_raw_mode()?;
        execute!(stdout, EnterAlternateScreen)?;

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        let mut state = AppState {
            current_area_name: BRIDGEPORT_OUTSKIRTS,
            quit: false,
        };
        let mut focus = FocusState::default();

        while !state.quit {
            terminal.draw(|frame| {
                let area = frame.area();
                let tree = compose(&state, area);
                render_tree(&tree, frame, &state);
            })?;

            if !event::poll(Duration::from_millis(250))? {
                continue;
            }

            let raw_event = event::read()?;
            if should_quit(&raw_event) {
                state.quit = true;
                continue;
            }

            let size = terminal.size()?;
            let area = Rect::new(0, 0, size.width, size.height);
            let tree = compose(&state, area);

            if let Ok(ui_event) = UiEvent::try_from(raw_event) {
                let outcome = route_event(&ui_event, &tree, &state, &focus);
                focus = focus.apply(&outcome.focus_update);
            }
        }

        Ok(())
    })();

    let _ = execute!(io::stdout(), LeaveAlternateScreen);
    let _ = disable_raw_mode();

    result
}

fn compose(state: &AppState, area: Rect) -> ViewTree<AppState, Msg> {
    let len: u16 = state.current_area_name.len().try_into().unwrap();
    let panel_width = len.saturating_add(4);
    let panel_height = 3.min(area.height);
    let panel_x = (area.width / 2)  - (panel_width / 2);
    let panel_y = 1.min(area.height.saturating_sub(panel_height).max(0));
    let panel_rect = Rect::new(panel_x, panel_y, panel_width, panel_height);
    let text_x = panel_x.saturating_add(2);
    let text_y = panel_y.saturating_add(1);
    let text_rect = Rect::new(text_x, text_y, len, 1);


    ViewTree::new(
        frust::root(area)
            .child(ViewNode::new(
                printable_grid(area.width, area.height)
                    .input_policy(InputPolicy::None)
                    .layer(Layer::Base)
                    .z_offset(i32::MIN),
                area,
            ))
            .child(
                ViewNode::new(Panel::new("area-name-box").borders(true).clear(true), panel_rect)
                ).child(
                    ViewNode::new(
                    CustomView::new("area-name",  |frame, area, state: &AppState| {
                        frame.render_widget(
                            ratatui::widgets::Paragraph::new(state.current_area_name),
                            area
                        );
                    }),
                    text_rect,
                    )
                )
            )
}

fn printable_grid(width: u16, height: u16) -> CellGrid {
    let mut grid = CellGrid::new("printable-background", width, height);
    for y in 0..height {
        for x in 0..width {
            let index = (y * width + x) % N_PRINTABLE_CHARS;
            let ch = (PRINTABLE_START + index as u8) as char;
            grid = grid.set_cell(x, y, ch, ratatui::style::Style::default());
        }
    }
    grid
}

fn should_quit(event: &Event) -> bool {
    match event {
        Event::Key(key) if key.kind == KeyEventKind::Press => {
            key.code == KeyCode::Char('q')
                || (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
        }
        _ => false,
    }
}

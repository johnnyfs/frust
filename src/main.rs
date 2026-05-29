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
    FocusState, FocusUpdate, InputPolicy, Layer, UiEvent, ViewId, ViewNode, ViewTree, render_tree,
    route_event,
    widgets::{CellGrid, Modal},
};
use ratatui::{Terminal, backend::CrosstermBackend, layout::Rect};

const PRINTABLE_START: u8 = b' ';
const N_PRINTABLE_CHARS: u16 = 95;

#[derive(Debug, Default)]
struct AppState {
    show_bridgeport: bool,
    quit: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Msg {
    DismissBridgeport,
}

fn main() -> io::Result<()> {
    let mut stdout = io::stdout();

    let result: io::Result<()> = (|| {
        enable_raw_mode()?;
        execute!(stdout, EnterAlternateScreen)?;

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        let mut state = AppState {
            show_bridgeport: true,
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
            sync_modal_focus(&state, &mut focus);

            if let Ok(ui_event) = UiEvent::try_from(raw_event) {
                let outcome = route_event(&ui_event, &tree, &state, &focus);
                for message in outcome.messages {
                    update(&mut state, message);
                }
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
    let modal_width = area.width.saturating_sub(4).clamp(24, 42).min(area.width);
    let modal_height = 5.min(area.height);
    let modal_area = Rect::new(
        area.x + area.width.saturating_sub(modal_width) / 2,
        area.y
            .saturating_add(1)
            .min(area.y.saturating_add(area.height.saturating_sub(1))),
        modal_width,
        modal_height,
    );

    ViewTree::new(
        frust::root(area)
            .child(ViewNode::new(
                printable_grid(area.width, area.height)
                    .input_policy(InputPolicy::None)
                    .layer(Layer::Base)
                    .z_offset(i32::MIN),
                area,
            ))
            .modal_if(
                state.show_bridgeport,
                ViewNode::new(
                    Modal::new("bridgeport-outskirts", "Bridgeport Outskirts")
                        .title("Bridgeport Outskirts")
                        .close_message(Msg::DismissBridgeport)
                        .clear(true),
                    modal_area,
                ),
            ),
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

fn update(state: &mut AppState, message: Msg) {
    match message {
        Msg::DismissBridgeport => state.show_bridgeport = false,
    }
}

fn sync_modal_focus(state: &AppState, focus: &mut FocusState) {
    let mut update = FocusUpdate::default();
    if state.show_bridgeport {
        update.set_active_modal(ViewId::new("bridgeport-outskirts"));
    } else {
        update.clear_active_modal();
    }
    *focus = focus.apply(&update);
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

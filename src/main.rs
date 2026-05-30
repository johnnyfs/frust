use std::{
    io::{self},
    time::Duration,
};

use crossterm::{
    event::{self},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use frust::{
    app::{self, AppState},
    tui::{FocusState, UiEvent, render_tree, route_event},
    ui,
};
use ratatui::{Terminal, backend::CrosstermBackend, layout::Rect};

fn main() -> io::Result<()> {
    let mut stdout = io::stdout();

    let result: io::Result<()> = (|| {
        enable_raw_mode()?;
        execute!(stdout, EnterAlternateScreen)?;

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        let mut state = AppState::default();
        let mut focus = FocusState::default();

        while !state.quit {
            terminal.draw(|frame| {
                let tree = ui::compose(&state, frame.area());
                render_tree(&tree, frame, &state);
            })?;

            if !event::poll(Duration::from_millis(250))? {
                continue;
            }

            let raw_event = event::read()?;
            if app::should_quit(&raw_event) {
                state.quit = true;
                continue;
            }

            if let Some(message) = app::message_for_event(&raw_event) {
                app::update(&mut state, message);
                continue;
            }

            let size = terminal.size()?;
            let area = Rect::new(0, 0, size.width, size.height);
            let tree = ui::compose(&state, area);

            if let Ok(ui_event) = UiEvent::try_from(raw_event) {
                let outcome = route_event(&ui_event, &tree, &state, &focus);
                for message in outcome.messages {
                    app::update(&mut state, message);
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

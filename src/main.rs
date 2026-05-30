use std::{
    io::{self},
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture},
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
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        let mut state = AppState::default();
        let mut focus = FocusState::default();
        let mut last_step = Instant::now();

        while !state.quit {
            let now = Instant::now();
            while now.duration_since(last_step) >= app::PLAYER_STEP_INTERVAL {
                app::tick(&mut state);
                last_step += app::PLAYER_STEP_INTERVAL;
            }

            terminal.draw(|frame| {
                let tree = ui::compose(&state, frame.area());
                render_tree(&tree, frame, &state);
            })?;

            let poll_timeout = app::PLAYER_STEP_INTERVAL
                .saturating_sub(last_step.elapsed())
                .min(Duration::from_millis(250));
            if !event::poll(poll_timeout)? {
                continue;
            }

            let raw_event = event::read()?;
            if app::should_quit(&raw_event) {
                state.quit = true;
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

    let _ = execute!(io::stdout(), DisableMouseCapture, LeaveAlternateScreen);
    let _ = disable_raw_mode();

    result
}

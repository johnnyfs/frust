use std::io::{self};
use crossterm::{execute, terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode}};
use ratatui::{Terminal, backend::{CrosstermBackend}, widgets::{Block, Borders, Paragraph}};

fn main() -> io::Result<()> {
    let mut stdout = io::stdout();

    let result: io::Result<()> = (|| {
        enable_raw_mode()?;
        execute!(stdout, EnterAlternateScreen)?;

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Test draw and wait
        terminal.clear()?;
        terminal.draw(|frame| {
            let area = frame.area();
            let widget = Paragraph::new("Hello there").block(Block::default().title("Demo").borders(Borders::ALL));

            frame.render_widget(widget, area);
        })?;

        crossterm::event::read()?;

        Ok(())
    })();

    let _ = execute!(io::stdout(), LeaveAlternateScreen);
    let _ = disable_raw_mode();

    result
}

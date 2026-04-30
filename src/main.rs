use crossterm::ExecutableCommand;
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::EnterAlternateScreen;
use crossterm::terminal::LeaveAlternateScreen;
use crossterm::terminal::disable_raw_mode;
use crossterm::terminal::enable_raw_mode;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io::{Result, stdout};

struct TerminalGuard;

impl TerminalGuard {
    fn new() -> Result<Self> {
        stdout().execute(EnterAlternateScreen)?;
        enable_raw_mode()?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = stdout().execute(LeaveAlternateScreen);
    }
}

fn main() -> Result<()> {
    let _terminal_guard = TerminalGuard::new()?;

    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|frame| {
            let area = frame.area(); // 整个屏幕区域
            let block = Block::default().borders(Borders::ALL).title("Hello");
            let paragraph = Paragraph::new("Hello, Ratatui! 🐭").block(block);

            frame.render_widget(paragraph, area);
        })?;

        // 异步检查 50ms 内按键按下
        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }
    }

    Ok(())
}

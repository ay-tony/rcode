use crossterm::ExecutableCommand;
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::EnterAlternateScreen;
use crossterm::terminal::LeaveAlternateScreen;
use crossterm::terminal::disable_raw_mode;
use crossterm::terminal::enable_raw_mode;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::widgets::Paragraph;
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

struct App {
    input: String,
}

impl Default for App {
    fn default() -> Self {
        Self {
            input: String::new(),
        }
    }
}

fn main() -> Result<()> {
    let _terminal_guard = TerminalGuard::new()?;

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    let mut app = App::default();

    loop {
        terminal.draw(|frame| {
            let area = frame.area(); // 整个屏幕区域

            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(70), // 消息
                    Constraint::Length(1),      // 状态
                    Constraint::Min(3),         // 输入
                ]);

            let [messages_area, status_area, input_area] = layout.split(area)[..] else {
                return;
            };

            frame.render_widget(Paragraph::new("messages here"), messages_area);
            frame.render_widget(
                Paragraph::new("Ready")
                    .style(Style::default().bg(Color::DarkGray).fg(Color::White)),
                status_area,
            );
            frame.render_widget(Paragraph::new(format!("> {}", app.input)), input_area);
        })?;

        // 异步检查 50ms 内按键按下
        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char(c) => app.input.push(c),
                    KeyCode::Backspace => {
                        app.input.pop();
                    }
                    KeyCode::Esc => break,
                    _ => {}
                }
            }
        }
    }

    Ok(())
}

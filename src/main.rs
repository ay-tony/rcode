use crossterm::ExecutableCommand;
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::EnterAlternateScreen;
use crossterm::terminal::LeaveAlternateScreen;
use crossterm::terminal::disable_raw_mode;
use crossterm::terminal::enable_raw_mode;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Paragraph, Wrap};
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

enum Message {
    User(String),
}

struct App {
    input: String,
    width: u16,
    messages: Vec<Message>,
    messages_scroll_pos: u16,
    messages_area_height: u16,
}

impl Default for App {
    fn default() -> Self {
        Self {
            input: String::new(),
            messages: Vec::new(),
            messages_scroll_pos: 0,
            messages_area_height: 60,
            width: 80,
        }
    }
}

fn main() -> Result<()> {
    let _terminal_guard = TerminalGuard::new()?;

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    let mut app = App::default();

    loop {
        let mut message_text = String::new();
        for message in &app.messages {
            let Message::User(content) = message;
            message_text.push_str(&format!("[User] {}\n", content));
        }

        terminal.draw(|frame| {
            let area = frame.area(); // 整个屏幕区域
            app.width = area.width;

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
            app.messages_area_height = messages_area.height;

            frame.render_widget(
                Paragraph::new(message_text.clone())
                    .scroll((app.messages_scroll_pos, 0))
                    .wrap(Wrap { trim: true }),
                messages_area,
            );
            frame.render_widget(
                Paragraph::new("Ready")
                    .style(Style::default().bg(Color::DarkGray).fg(Color::White)),
                status_area,
            );
            frame.render_widget(
                Paragraph::new(format!("> {}", app.input)).wrap(Wrap { trim: true }),
                input_area,
            );
        })?;

        // 异步检查 50ms 内按键按下
        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char(c) => app.input.push(c),
                    KeyCode::Backspace => {
                        app.input.pop();
                    }
                    KeyCode::Enter => {
                        if !app.input.is_empty() {
                            app.messages.push(Message::User(app.input.clone()));
                            app.input.clear();
                        }
                    }
                    KeyCode::Up => {
                        app.messages_scroll_pos = app.messages_scroll_pos.saturating_sub(1);
                    }
                    KeyCode::Down => {
                        let max_messages_lines: u16 = Paragraph::new(message_text)
                            .wrap(Wrap { trim: true })
                            .line_count(app.width)
                            as u16;
                        app.messages_scroll_pos = (app.messages_scroll_pos + 1)
                            .min(max_messages_lines.saturating_sub(app.messages_area_height));
                    }
                    KeyCode::Esc => break,
                    _ => {}
                }
            }
        }
    }

    Ok(())
}

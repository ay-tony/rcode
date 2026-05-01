use crate::app::App;
use crate::render::text::format_messages;
use crossterm::ExecutableCommand;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Paragraph, Wrap},
};
use std::{error::Error, io::stdout};

/// RAII guard：进入 alternate screen 和 raw mode，Drop 时自动恢复。
pub struct TerminalGuard;

impl TerminalGuard {
    pub fn new() -> Result<Self, Box<dyn Error>> {
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

/// 绘制整个 TUI 界面。同时更新 App 中的派生值（width / height / max_scroll）。
pub fn draw(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    app.width = area.width;

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(70),
            Constraint::Length(1),
            Constraint::Min(3),
        ]);

    let [messages_area, status_area, input_area] = layout.split(area)[..] else {
        return;
    };
    app.messages_area_height = messages_area.height;

    let message_text = format_messages(app.messages());
    let max_lines = Paragraph::new(message_text.clone())
        .wrap(Wrap { trim: true })
        .line_count(app.width) as u16;
    app.max_scroll = max_lines.saturating_sub(app.messages_area_height);

    frame.render_widget(
        Paragraph::new(message_text)
            .scroll((app.scroll_pos(), 0))
            .wrap(Wrap { trim: true }),
        messages_area,
    );

    frame.render_widget(
        Paragraph::new("Ready").style(Style::default().bg(Color::DarkGray).fg(Color::White)),
        status_area,
    );

    frame.render_widget(
        Paragraph::new(format!("> {}", app.input())).wrap(Wrap { trim: true }),
        input_area,
    );
}

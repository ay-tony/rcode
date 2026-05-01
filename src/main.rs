mod agent;
mod app;
mod config;
mod event;
mod handler;
mod render;
mod tools;

use ratatui::{Terminal, backend::CrosstermBackend};
use std::{error::Error, io::stdout};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let _terminal_guard = render::tui::TerminalGuard::new()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    let mut app = app::App::default();
    let mut agent = crate::agent::Agent::new(crate::config::Config::from_file(
        &crate::config::resolve_config_path()?,
    )?)?;

    loop {
        terminal.draw(|frame| render::tui::draw(frame, &mut app))?;

        if crossterm::event::poll(std::time::Duration::from_millis(50))? {
            if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
                match event::handle_key(&mut app, key) {
                    event::EventResult::Exit => break,
                    event::EventResult::SendMessage(msg) => {
                        handler::handle_user_message(&mut app, &mut agent, msg).await
                    }
                    event::EventResult::Continue => {}
                }
            }
        }
    }

    Ok(())
}

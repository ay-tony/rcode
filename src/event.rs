use crate::app::App;
use crossterm::event::{KeyCode, KeyEvent};

pub enum EventResult {
    Continue,
    Exit,
    SendMessage(String),
}

pub fn handle_key(app: &mut App, key: KeyEvent) -> EventResult {
    match key.code {
        KeyCode::Char(c) => {
            app.push_input_char(c);
            EventResult::Continue
        }
        KeyCode::Backspace => {
            app.pop_input();
            EventResult::Continue
        }
        KeyCode::Enter => {
            if let Some(msg) = app.take_input() {
                EventResult::SendMessage(msg)
            } else {
                EventResult::Continue
            }
        }
        KeyCode::Up => {
            app.scroll_up();
            EventResult::Continue
        }
        KeyCode::Down => {
            app.scroll_down();
            EventResult::Continue
        }
        KeyCode::Esc => EventResult::Exit,
        _ => EventResult::Continue,
    }
}

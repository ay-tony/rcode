#[derive(Clone, Debug)]
pub enum Message {
    User(String),
    Assistant(String),
    Error(String),
}

pub struct App {
    input: String,
    pub(crate) width: u16,
    messages: Vec<Message>,
    messages_scroll_pos: u16,
    pub(crate) messages_area_height: u16,
    pub(crate) max_scroll: u16,
}

impl Default for App {
    fn default() -> Self {
        Self {
            input: String::new(),
            messages: Vec::new(),
            messages_scroll_pos: 0,
            messages_area_height: 60,
            width: 80,
            max_scroll: 0,
        }
    }
}

impl App {
    pub fn push_input_char(&mut self, c: char) {
        self.input.push(c);
    }
    pub fn pop_input(&mut self) {
        self.input.pop();
    }
    pub fn scroll_up(&mut self) {
        self.messages_scroll_pos = self.messages_scroll_pos.saturating_sub(1);
    }
    pub fn scroll_down(&mut self) {
        self.messages_scroll_pos = (self.messages_scroll_pos + 1).min(self.max_scroll);
    }

    pub fn take_input(&mut self) -> Option<String> {
        if self.input.is_empty() {
            None
        } else {
            Some(std::mem::take(&mut self.input))
        }
    }

    pub fn input(&self) -> &str {
        &self.input
    }
    pub fn messages(&self) -> &[Message] {
        &self.messages
    }
    pub fn scroll_pos(&self) -> u16 {
        self.messages_scroll_pos
    }

    pub fn push_message(&mut self, msg: Message) {
        self.messages.push(msg);
    }
}

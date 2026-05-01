use crate::app::Message;
use std::error::Error;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub fn count_lines_in_terminal(text: &str, term_width: usize) -> Result<usize, Box<dyn Error>> {
    let lines = text.lines();
    let mut result = if text.width() == 0 {
        0
    } else {
        lines.map(|line| count_visual_lines(line, term_width)).sum()
    };
    if text.ends_with('\n') {
        result += 1;
    }
    Ok(result)
}

fn count_visual_lines(line: &str, term_width: usize) -> usize {
    if line.is_empty() {
        return 1;
    }
    let mut lines = 0;
    let mut current_width = 0;
    for c in line.chars() {
        let w = c.width().unwrap_or(0);
        if current_width + w > term_width {
            lines += 1;
            current_width = w;
        } else {
            current_width += w;
        }
    }
    if current_width > 0 {
        lines += 1;
    }
    lines
}

pub fn format_messages(messages: &[Message]) -> String {
    let mut text = String::new();
    for message in messages {
        match message {
            Message::User(content) => text.push_str(&format!("[User] {}\n", content)),
            Message::Assistant(content) => text.push_str(&format!("[Assistant] {}\n", content)),
            Message::Error(content) => text.push_str(&format!("[Error] {}\n", content)),
        }
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_string_returns_zero() {
        assert_eq!(count_lines_in_terminal("", 80).unwrap(), 0);
    }

    #[test]
    fn single_line_no_newline() {
        assert_eq!(count_lines_in_terminal("hello", 80).unwrap(), 1);
    }

    #[test]
    fn trailing_newline_counts_as_extra_line() {
        assert_eq!(count_lines_in_terminal("hello\n", 80).unwrap(), 2);
    }

    #[test]
    fn multiple_physical_lines() {
        assert_eq!(count_lines_in_terminal("a\nb\nc", 80).unwrap(), 3);
    }

    #[test]
    fn many_newlines() {
        assert_eq!(count_lines_in_terminal("\n\n\n\n", 80).unwrap(), 5);
    }

    #[test]
    fn long_line_wraps() {
        let text = "a".repeat(100) + "\n";
        assert_eq!(count_lines_in_terminal(&text, 80).unwrap(), 3);
    }

    #[test]
    fn cjk_wrap() {
        assert_eq!(
            count_lines_in_terminal("天地玄黄\n\n宇宙洪荒", 2).unwrap(),
            9
        );
        assert_eq!(count_lines_in_terminal("天地玄黄宇宙洪荒", 3).unwrap(), 8);
    }

    #[test]
    fn format_user_and_assistant() {
        let msgs = vec![
            Message::User("hello".into()),
            Message::Assistant("world".into()),
        ];
        assert_eq!(format_messages(&msgs), "[User] hello\n[Assistant] world\n");
    }

    #[test]
    fn format_error() {
        let msgs = vec![Message::Error("oops".into())];
        assert_eq!(format_messages(&msgs), "[Error] oops\n");
    }
}

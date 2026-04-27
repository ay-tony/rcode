use crossterm::terminal;
use std::error::Error;
use unicode_width::UnicodeWidthStr;

pub fn count_lines_in_terminal(text: &str) -> Result<usize, Box<dyn Error>> {
    let (col, _) = terminal::size()?;
    let lines = text.lines();
    let mut result = if text.width() == 0 {
        0
    } else {
        lines
            .map(|line| {
                let w = line.width();
                if w == 0 {
                    1
                } else {
                    (line.width() + col as usize - 1) / (col as usize)
                }
            })
            .sum()
    };
    if text.ends_with('\n') {
        result += 1;
    }
    Ok(result)
}

pub fn clear_lines_above(n: usize) {
    print!("\x1B[{n}A\x1B[G\x1B[0J");
}

pub fn render_markdown(text: &str) {
    let skin = termimad::MadSkin::default();
    println!("{}", skin.term_text(text));
}

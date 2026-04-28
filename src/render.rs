use std::error::Error;
use unicode_width::{UnicodeWidthStr, UnicodeWidthChar};

fn count_visual_lines(line: &str, term_width: usize) -> usize {
    if line.is_empty() {
        return 1;
    }
    
    let mut lines = 0;
    let mut current_width = 0;
    
    for c in line.chars() {
        let w = c.width().unwrap_or(0);
        // 如果当前行放不下这个字符，先换行
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

pub fn count_lines_in_terminal(text: &str, term_width: usize) -> Result<usize, Box<dyn Error>> {
    let lines = text.lines();
    let mut result = if text.width() == 0 {
        0
    } else {
        lines
            .map(|line| count_visual_lines(line, term_width))
            .sum()
    };
    if text.ends_with('\n') {
        result += 1;
    }
    Ok(result)
}

pub fn clear_lines_above(n: usize) {
    print!("\n\x1B[{n}A\x1B[G\x1B[0J");
}

pub fn render_markdown(text: &str) {
    let skin = termimad::MadSkin::default();
    println!("{}", skin.term_text(text));
}

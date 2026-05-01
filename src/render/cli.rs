pub fn render_markdown(text: &str) {
    println!("{}", termimad::MadSkin::default().term_text(text));
}

pub fn clear_lines_above(n: usize) {
    print!("\n\x1B[{n}F\x1B[0J");
}

#[derive(Debug)]
pub struct Position {
    pub byte: usize,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

impl Position {
    pub fn new() -> Self {
        Position {
            byte: 0,
            line: 1,
            col: 1,
        }
    }

    pub fn step(&mut self, c: char) {
        self.byte += c.len_utf8();
        self.col += 1;
        if c == '\n' || c == '\u{0085}' || c == '\u{2028}' || c == '\u{2029}' {
            self.line += 1;
            self.col = 1;
        }
    }
}

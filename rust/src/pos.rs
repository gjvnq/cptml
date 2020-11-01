#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub byte: usize,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

impl Span {
    pub fn new() -> Self {
        Span {
            start: Position::new(),
            end: Position::new(),
        }
    }
    pub fn new2(
        start_byte: usize,
        start_line: usize,
        start_col: usize,
        end_byte: usize,
        end_line: usize,
        end_col: usize,
    ) -> Self {
        Span {
            start: Position::new2(start_byte, start_line, start_col),
            end: Position::new2(end_byte, end_line, end_col),
        }
    }

    pub fn new_from(pos: Position) -> Self {
        Span {
            start: pos,
            end: pos,
        }
    }

    pub fn step(&mut self, c: char) {
        self.end.step(c)
    }

    pub fn rotate(&mut self) {
        self.start = self.end;
    }

    // in bytes
    pub fn len(self) -> usize {
        self.end.byte - self.start.byte
    }
}

impl Position {
    pub fn new() -> Self {
        Position {
            byte: 0,
            line: 1,
            col: 0,
        }
    }
    pub fn new2(byte: usize, line: usize, col: usize) -> Self {
        Position {
            byte: byte,
            line: line,
            col: col,
        }
    }

    pub fn step(&mut self, c: char) {
        self.byte += c.len_utf8();
        self.col += 1;
        if c == '\n' || c == '\u{0085}' || c == '\u{2028}' || c == '\u{2029}' {
            self.line += 1;
            self.col = 0;
        }
    }

    pub fn start_span(&self) -> Span {
        Span {
            start: *self,
            end: *self,
        }
    }
}

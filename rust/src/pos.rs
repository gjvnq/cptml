#[derive(Debug)]
pub struct Position {
    byte: usize,
    line: usize,
    col: usize,
}

#[derive(Debug)]
pub struct Span {
	start: Position,
	end: Position,
}

impl Position {
	pub fn step(&mut self, c: char) {
		self.byte += c.len_utf8();
		self.col += 1;
		if c == '\n' || c == '\u{0085}' || c == '\u{2028}' || c == '\u{2029}' {
			self.line += 1;
			self.col = 1;
		}
	}
}
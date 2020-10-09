#[derive(Debug)]
pub struct Position {
    byte: u64,
    line: u64,
    col: u64,
}

#[derive(Debug)]
pub struct Span {
	start: Position,
	end: Position,
}
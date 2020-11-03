use crate::pos::Position;
use crate::pos::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParserError {
    IllegalChar(Position, char),
    IllegalChar2(Position, char, Vec<char>),
    IllegalCharMsg(Position, char, String),
    MissingTerminator(Position, char),
    MissingLocalName(Position),
    IllegalEscapeSequence(Position, String),
    IllegalNumber(Span, String),
    EndOfInput,
}
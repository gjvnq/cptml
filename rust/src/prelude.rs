pub use crate::pos::Position;
pub use crate::pos::Span;
pub use crate::chars::CharErrorEnum;

pub trait ByteReader: core::fmt::Debug + Iterator<Item = u8> {}
impl ByteReader for std::str::Bytes<'_> {}

pub type CResult<T> = Result<T, AnyError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnyError {
    CharError(CharErrorEnum),
    OutOfRange(isize, isize, isize),
    EndOfInput,
    // IllegalChar(Position, char),
    // IllegalChar2(Position, char, Vec<char>),
    // IllegalCharMsg(Position, char, String),
    // MissingTerminator(Position, char),
    // MissingLocalName(Position),
    // IllegalEscapeSequence(Position, String),
    // IllegalNumber(Span, String),
    // NotAttributeValue(Span, String),
    // NotReadyYet,
}

impl std::convert::From<CharErrorEnum> for AnyError {
    fn from(err: CharErrorEnum) -> Self {
        AnyError::CharError(err)
    }
}
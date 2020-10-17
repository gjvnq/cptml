//! This module/file contains low level stuff you probably should not use.

#![allow(dead_code)]
#![allow(unused_variables)]

use crate::peek_reader::PeekReader;
use crate::pos::Span;
use core::fmt::Debug;
use std::mem;

#[derive(Debug)]
pub struct RawName {
    pub view: String,
    pub special: bool,
    pub prefix: String,
    pub local: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RawToken {
    CodeBlock(Span, String),
    InlineText(Span, String),
    InlineMathText(Span, String),
    DisplayMathText(Span, String),
    AttributeName(Span, String),
    BooleanValue(Span, bool),
    IntegerValue(Span, i64),
    FloatValue(Span, i64, i64),
    StringValue(Span, String),
    CurlyTagStart(Span, String),
    CurlyTagEnd(Span),
    PointyTagStart(Span, String),
    PointyTagEnd(Span, String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    CodeBlock,
    InlineText(TextEscapeState),
    InlineMathText,
    DisplayMathText,
    AttributeName,
    BooleanValue,
    IntegerValue,
    FloatValue,
    StringValue,
    CurlyTagStart,
    CurlyTagEnd,
    PointyTagStart,
    PointyTagEnd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TextEscapeState {
    None,
    Slash,
    Unicode,
}

pub trait ByteReader: Debug + Iterator<Item = u8> {}

impl ByteReader for std::str::Bytes<'_> {}

#[derive(Debug)]
pub struct RawParser {
    src: Box<PeekReader>,
    txt: String,
    tmp: String,
    span: Span,
    state: State,
    result: Option<RawToken>,
    done: bool,
    skip: bool,
}

impl RawParser {
    pub fn new(reader: Box<dyn ByteReader>) -> Self {
        RawParser {
            src: Box::new(PeekReader::new(reader)),
            txt: "".to_string(),
            tmp: "".to_string(),
            state: State::InlineText(TextEscapeState::None),
            span: Span::new(),
            result: None,
            done: false,
            skip: false,
        }
    }

    #[allow(mutable_borrow_reservation_conflict)]
    fn until_yield(&mut self) {
        self.result = None;
        if self.done {
            return;
        }

        self.span = Span::new_from(self.src.get_pos());
        while self.result.is_none() {
            let c = match self.skip {
                false => self.src.pop(),
                true => self.src.peek(0),
            };
            self.skip = false;
            // Finish if EOF
            if c == '\0' {
                self.done = true;
                match &self.state {
                    State::InlineText(substate) => self.result_text(*substate),
                    State::CurlyTagStart => self.result_curly_start(),
                    State::CurlyTagEnd => self.result_curly_start(),
                    State::PointyTagStart => self.result_pointy_start(),
                    State::PointyTagEnd => self.result_pointy_start(),
                    _ => panic!("unexpected state: {:?}", self.state),
                }
                return;
            }
            // Process new char
            match &self.state {
                State::InlineText(substate) => self.mode_text(c, *substate),
                State::CurlyTagStart => self.mode_curly_start(c),
                State::CurlyTagEnd => self.mode_curly_start(c),
                State::PointyTagStart => self.mode_pointy_start(c),
                State::PointyTagEnd => self.mode_pointy_start(c),
                _ => panic!("unexpected state: {:?}", self.state),
            }
        }
    }

    fn result_pointy_end(&mut self) {}

    fn mode_pointy_end(&mut self, c: char) {}

    fn result_pointy_start(&mut self) {}

    fn mode_pointy_start(&mut self, c: char) {}

    fn result_curly_end(&mut self) {}

    fn mode_curly_end(&mut self, c: char) {}

    fn result_curly_start(&mut self) {}

    fn mode_curly_start(&mut self, c: char) {}

    fn result_text(&mut self, escape: TextEscapeState) {
        if self.txt.len() > 0 {
            self.result = Some(RawToken::InlineText(self.span, self.txt.clone()));
            self.txt.clear();
        }
    }

    fn mode_text(&mut self, c: char, substate: TextEscapeState) {
        let mut escape = substate;

        if (c == '{' || c == '}' || c == '<' || c == '>')
            && escape == TextEscapeState::None
            && !self.src.peek(1).is_whitespace()
        {
            self.result_text(escape);
            match c {
                '{' => self.state = State::CurlyTagStart,
                '}' => self.state = State::CurlyTagEnd,
                '<' => self.state = State::PointyTagStart,
                '>' => self.state = State::PointyTagStart,
                _ => {}
            }
            self.skip = true;
        } else {
            self.txt.push(c);
            self.span.step(c);
            if c == '\\' && escape == TextEscapeState::None {
                escape = TextEscapeState::Slash;
            } else if escape == TextEscapeState::Slash {
                if c == 'u' {
                    escape = TextEscapeState::Unicode;
                } else {
                    escape = TextEscapeState::None;
                }
            } else if escape == TextEscapeState::Slash && c == ';' {
                escape = TextEscapeState::None;
            }

            self.state = State::InlineText(escape);
        }
    }
}

impl Iterator for RawParser {
    type Item = RawToken;

    fn next(&mut self) -> Option<Self::Item> {
        self.until_yield();

        let mut ans: Option<RawToken> = None;
        mem::swap(&mut self.result, &mut ans);
        return ans;
    }
}

#[cfg(test)]
mod tests {
    use crate::raw_parser::*;

    #[test]
    fn test_1() {
        let s = "";
        let mut parser = RawParser::new(Box::new(s.bytes()));
        assert_eq!(parser.next(), None);

        let s = "a";
        let mut parser = RawParser::new(Box::new(s.bytes()));
        assert_eq!(
            parser.next(),
            Some(RawToken::InlineText(
                Span::new2(0, 1, 1, 1, 1, 2),
                "a".to_string()
            ))
        );
        assert_eq!(parser.next(), None);

        let s = "a{";
        let mut parser = RawParser::new(Box::new(s.bytes()));
        assert_eq!(
            parser.next(),
            Some(RawToken::InlineText(
                Span::new2(0, 1, 1, 1, 1, 2),
                "a".to_string()
            ))
        );
        assert_eq!(parser.next(), None);

        let s = "hello world! {";
        let mut parser = RawParser::new(Box::new(s.bytes()));
        assert_eq!(
            parser.next(),
            Some(RawToken::InlineText(
                Span::new2(0, 1, 1, 13, 1, 14),
                "hello world! ".to_string()
            ))
        );
        assert_eq!(parser.next(), None);

        let s = "hello > world!{ ";
        let mut parser = RawParser::new(Box::new(s.bytes()));
        assert_eq!(
            parser.next(),
            Some(RawToken::InlineText(
                Span::new2(0, 1, 1, 16, 1, 17),
                "hello > world!{ ".to_string()
            ))
        );
        assert_eq!(parser.next(), None);

        let s = "hello } world!{!";
        let mut parser = RawParser::new(Box::new(s.bytes()));
        assert_eq!(
            parser.next(),
            Some(RawToken::InlineText(
                Span::new2(0, 1, 1, 14, 1, 15),
                "hello } world!".to_string()
            ))
        );

        let s = "\\t} \\{\\s ";
        let mut parser = RawParser::new(Box::new(s.bytes()));
        assert_eq!(
            parser.next(),
            Some(RawToken::InlineText(
                Span::new2(0, 1, 1, 9, 1, 10),
                "\\t} \\{\\s ".to_string()
            ))
        );
    }
}

//! This module/file contains low level stuff you probably should not use.

#![allow(dead_code)]
#![allow(unused_variables)]

use crate::hacks::is_valid_id_first_char;
use crate::hacks::is_valid_id_next_char;
use crate::hacks::ByteReader;
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

type GotFirstLetter = bool;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    CodeBlock,
    InlineText(TextEscapeState),
    InlineMathText,
    DisplayMathText,
    AttributeName(GotFirstLetter), // TODO: add another substate for curly vs pointy
    BooleanValue,
    NumericValue,
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

#[derive(Debug)]
pub struct RawTokenizer {
    src: Box<PeekReader>,
    txt: String,
    tmp: String,
    span: Span,
    state: State,
    result: Option<RawToken>,
    done: bool,
    repeat_c: bool,
}

impl RawTokenizer {
    pub fn new(reader: Box<dyn ByteReader>) -> Self {
        RawTokenizer {
            src: Box::new(PeekReader::new(reader)),
            txt: "".to_string(),
            tmp: "".to_string(),
            state: State::InlineText(TextEscapeState::None),
            span: Span::new(),
            result: None,
            done: false,
            repeat_c: false,
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
            let c = match self.repeat_c {
                false => self.src.pop(),
                true => self.src.peek(0),
            };
            self.repeat_c = false;
            // Finish if EOF
            if c == '\0' {
                self.done = true;
                match &self.state {
                    State::InlineText(substate) => self.result_text(*substate),
                    State::CurlyTagStart => self.result_curly_start(),
                    State::CurlyTagEnd => self.result_curly_end(),
                    State::PointyTagStart => self.result_pointy_start(),
                    State::PointyTagEnd => self.result_pointy_end(),
                    State::AttributeName(_) => self.result_attribute_name(),
                    _ => panic!("unexpected state: {:?}", self.state),
                }
                return;
            }
            // Process new char
            match &self.state {
                State::InlineText(substate) => self.mode_text(c, *substate),
                State::CurlyTagStart => self.mode_curly_start(c),
                State::CurlyTagEnd => self.mode_curly_end(c),
                State::PointyTagStart => self.mode_pointy_start(c),
                State::PointyTagEnd => self.mode_pointy_end(c),
                State::AttributeName(substate) => self.mode_attribute_name(c, *substate),
                _ => panic!("unexpected state: {:?}", self.state),
            }
        }
    }

    fn result_attribute_name(&mut self) {
        self.result = Some(RawToken::AttributeName(self.span, self.txt.clone()));
    }

    fn mode_attribute_name(&mut self, c: char, substate: bool) {
        let mut got_first_letter = substate;

        let id_char = match got_first_letter {
            false => is_valid_id_first_char(c),
            true => is_valid_id_next_char(c)
        };
        if !got_first_letter {
            if id_char {
                got_first_letter = true;
                self.state = State::AttributeName(got_first_letter);
            } else if !c.is_whitespace() {
                // attribute names can't begin with digits
                panic!("unexpected character {:?} at {:?}", c, self.span.end);
            }
        }
        if id_char || c == '=' || !got_first_letter || c == ';' {
            self.txt.push(c);
            self.span.step(c);
        }

        if c == '=' || (c.is_whitespace() && got_first_letter) {
            self.result_attribute_name();
            let next_c = self.src.peek(1);
            if next_c == '\"' {
                self.state = State::StringValue;
            } else if next_c == 'f' || next_c == 't' {
                self.state = State::BooleanValue;
            } else if next_c.is_ascii_digit() || next_c == '.' {
                self.state = State::NumericValue;
            } else if c.is_whitespace() {
                self.state = State::AttributeName(false);
            } else {
                panic!("unexpected character {:?} at {:?}", c, self.src.get_pos());
            }
        } else if c == ';' || c == '}' || c == '>' {
            self.state = match c {
                ';' => State::InlineText(TextEscapeState::None),
                '}' => State::CurlyTagEnd,
                '>' => State::PointyTagEnd,
                _ => unreachable!(),
            };
            self.repeat_c = match c {
                ';' => false,
                _ => true
            };
            self.result_attribute_name();
        } else if !id_char {
            panic!("unexpected character {:?} at {:?}", c, self.span.end);
        }
    }

    fn result_pointy_end(&mut self) {}

    fn mode_pointy_end(&mut self, c: char) {}

    fn result_pointy_start(&mut self) {}

    fn mode_pointy_start(&mut self, c: char) {}

    fn result_curly_end(&mut self) {
        self.result = Some(RawToken::CurlyTagEnd(self.span));
    }

    fn mode_curly_end(&mut self, c: char) {
        self.span.step(c);
        self.state = State::InlineText(TextEscapeState::None);
        self.result_curly_end();
    }

    fn result_curly_start(&mut self) {
        self.result = Some(RawToken::CurlyTagStart(self.span, self.txt.clone()));
    }

    fn mode_curly_start(&mut self, c: char) {
        let last_c = self.src.peek(-1);
        if last_c.is_whitespace() && c.is_alphabetic() && self.txt.len() > 0 {
            println!("{:?} {:?} {:?}", self.txt, last_c, c);
            self.result_curly_start();
            self.state = State::AttributeName(true);
            self.repeat_c = true;
            return;
        }
        if c != '}' {
            self.txt.push(c);
            self.span.step(c);
        }

        if c == '}' || c == ';' {
            match c {
                '}' => self.state = State::CurlyTagEnd,
                ';' => self.state = State::InlineText(TextEscapeState::None),
                _ => {}
            }
            self.result_curly_start();
        }
        if c == '}' {
            self.repeat_c = true;
            return;
        }
    }

    fn result_text(&mut self, escape: TextEscapeState) {
        if self.txt.len() > 0 {
            self.result = Some(RawToken::InlineText(self.span, self.txt.clone()));
        }
    }

    fn mode_text(&mut self, c: char, substate: TextEscapeState) {
        let mut escape = substate;

        let last_c = self.src.peek(-1);
        let next_c = self.src.peek(1);
        let between_spaces = (next_c == ' ' || next_c == '\0') && last_c == ' ';
        if (c == '{' || c == '}' || c == '<' || c == '|')
            && escape == TextEscapeState::None
            && !between_spaces
        {
            self.result_text(escape);
            match c {
                '{' => self.state = State::CurlyTagStart,
                '}' => self.state = State::CurlyTagEnd,
                '<' => self.state = State::PointyTagStart,
                '|' => self.state = State::PointyTagEnd,
                _ => {}
            }
            self.repeat_c = true;
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

impl Iterator for RawTokenizer {
    type Item = RawToken;

    fn next(&mut self) -> Option<Self::Item> {
        self.until_yield();
        self.txt.clear();

        let mut ans: Option<RawToken> = None;
        mem::swap(&mut self.result, &mut ans);
        return ans;
    }
}

#[cfg(test)]
mod tests {
    use crate::raw_tokenizer::*;

    #[test]
    fn test_1() {
        let s = "";
        let mut parser = RawTokenizer::new(Box::new(s.bytes()));
        assert_eq!(parser.next(), None);

        let s = "a";
        let mut parser = RawTokenizer::new(Box::new(s.bytes()));
        assert_eq!(
            parser.next(),
            Some(RawToken::InlineText(
                Span::new2(0, 1, 0, 1, 1, 1),
                "a".to_string()
            ))
        );
        assert_eq!(parser.next(), None);

        let s = "a{";
        let mut parser = RawTokenizer::new(Box::new(s.bytes()));
        assert_eq!(
            parser.next(),
            Some(RawToken::InlineText(
                Span::new2(0, 1, 0, 1, 1, 1),
                "a".to_string()
            ))
        );

        let s = "hello world! {";
        let mut parser = RawTokenizer::new(Box::new(s.bytes()));
        assert_eq!(
            parser.next(),
            Some(RawToken::InlineText(
                Span::new2(0, 1, 0, 14, 1, 14),
                "hello world! {".to_string()
            ))
        );

        let s = "hello > world!{ ";
        let mut parser = RawTokenizer::new(Box::new(s.bytes()));
        assert_eq!(
            parser.next(),
            Some(RawToken::InlineText(
                Span::new2(0, 1, 0, 14, 1, 14),
                "hello > world!".to_string()
            ))
        );

        let s = "hello } world!{!";
        let mut parser = RawTokenizer::new(Box::new(s.bytes()));
        assert_eq!(
            parser.next(),
            Some(RawToken::InlineText(
                Span::new2(0, 1, 0, 14, 1, 14),
                "hello } world!".to_string()
            ))
        );

        let s = "\\t } \\{\\s ";
        let mut parser = RawTokenizer::new(Box::new(s.bytes()));
        assert_eq!(
            parser.next(),
            Some(RawToken::InlineText(
                Span::new2(0, 1, 0, 10, 1, 10),
                "\\t } \\{\\s ".to_string()
            ))
        );
        assert_eq!(parser.next(), None);
    }

    #[test]
    fn test_2() {
        let s = "abc {icon}{em; hi! }\n{em ; Hi!\\s} ";
        let mut parser = RawTokenizer::new(Box::new(s.bytes()));
        assert_eq!(
            parser.next(),
            Some(RawToken::InlineText(
                Span::new2(0, 1, 0, 4, 1, 4),
                "abc ".to_string()
            ))
        );
        assert_eq!(
            parser.next(),
            Some(RawToken::CurlyTagStart(
                Span::new2(5, 1, 5, 10, 1, 10),
                "{icon".to_string()
            ))
        );
        assert_eq!(
            parser.next(),
            Some(RawToken::CurlyTagEnd(Span::new2(10, 1, 10, 11, 1, 11)))
        );
        assert_eq!(
            parser.next(),
            Some(RawToken::CurlyTagStart(
                Span::new2(10, 1, 10, 14, 1, 14),
                "{em;".to_string()
            ))
        );
        assert_eq!(
            parser.next(),
            Some(RawToken::InlineText(
                Span::new2(14, 1, 14, 19, 1, 19),
                " hi! ".to_string()
            ))
        );
        assert_eq!(
            parser.next(),
            Some(RawToken::CurlyTagEnd(Span::new2(20, 1, 20, 21, 1, 21)))
        );
        assert_eq!(
            parser.next(),
            Some(RawToken::InlineText(
                Span::new2(20, 1, 20, 21, 2, 0),
                "\n".to_string()
            ))
        );
        assert_eq!(
            parser.next(),
            Some(RawToken::CurlyTagStart(
                Span::new2(22, 2, 1, 27, 2, 6),
                "{em ;".to_string()
            ))
        );
        assert_eq!(
            parser.next(),
            Some(RawToken::InlineText(
                Span::new2(26, 2, 5, 32, 2, 11),
                " Hi!\\s".to_string()
            ))
        );
        assert_eq!(
            parser.next(),
            Some(RawToken::CurlyTagEnd(Span::new2(33, 2, 12, 34, 2, 13)))
        );
        assert_eq!(
            parser.next(),
            Some(RawToken::InlineText(
                Span::new2(33, 2, 12, 34, 2, 13),
                " ".to_string()
            ))
        );
        assert_eq!(parser.next(), None);
    }

    #[test]
    fn test_3() {
        let s = "abc {icon bool_attr id=\"abc\" num=1 val=.5; text}{end id=\"e\"}";
        let mut parser = RawTokenizer::new(Box::new(s.bytes()));
        assert_eq!(
            parser.next(),
            Some(RawToken::InlineText(
                Span::new2(0, 1, 0, 4, 1, 4),
                "abc ".to_string()
            ))
        );
        assert_eq!(
            parser.next(),
            Some(RawToken::CurlyTagStart(
                Span::new2(5, 1, 5, 11, 1, 11),
                "{icon ".to_string()
            ))
        );
        assert_eq!(
            parser.next(),
            Some(RawToken::AttributeName(
                Span::new2(11, 1, 11, 20, 1, 20),
                "bool_attr".to_string()
            ))
        );
        assert_eq!(
            parser.next(),
            Some(RawToken::AttributeName(
                Span::new2(20, 1, 20, 23, 1, 23),
                "id=".to_string()
            ))
        );
        assert_eq!(
            parser.next(),
            Some(RawToken::CurlyTagEnd(Span::new2(10, 1, 10, 11, 1, 11)))
        );
        assert_eq!(
            parser.next(),
            Some(RawToken::CurlyTagStart(
                Span::new2(10, 1, 10, 14, 1, 14),
                "{em;".to_string()
            ))
        );
        assert_eq!(
            parser.next(),
            Some(RawToken::InlineText(
                Span::new2(14, 1, 14, 19, 1, 19),
                " hi! ".to_string()
            ))
        );
        assert_eq!(
            parser.next(),
            Some(RawToken::CurlyTagEnd(Span::new2(20, 1, 20, 21, 1, 21)))
        );
        assert_eq!(
            parser.next(),
            Some(RawToken::InlineText(
                Span::new2(20, 1, 20, 21, 2, 0),
                "\n".to_string()
            ))
        );
        assert_eq!(
            parser.next(),
            Some(RawToken::CurlyTagStart(
                Span::new2(22, 2, 1, 27, 2, 6),
                "{em ;".to_string()
            ))
        );
        assert_eq!(
            parser.next(),
            Some(RawToken::InlineText(
                Span::new2(26, 2, 5, 32, 2, 11),
                " Hi!\\s".to_string()
            ))
        );
        assert_eq!(
            parser.next(),
            Some(RawToken::CurlyTagEnd(Span::new2(33, 2, 12, 34, 2, 13)))
        );
        assert_eq!(
            parser.next(),
            Some(RawToken::InlineText(
                Span::new2(33, 2, 12, 34, 2, 13),
                " ".to_string()
            ))
        );
        assert_eq!(parser.next(), None);
    }
}

//! This module/file contains low level stuff you probably should not use.

#![allow(dead_code)]
#![allow(unused_variables)]


use crate::pos::Position;
use crate::hacks::is_valid_id_first_char;
use crate::hacks::is_valid_id_next_char;
use crate::hacks::ByteReader;
use crate::peek_reader::PeekReader;
use crate::pos::Span;
use core::fmt::Debug;
use std::mem;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawName {
    pub view: String,
    pub special: bool,
    pub prefix: String,
    pub local: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DualString {
    raw: String,
    parsed: String
}

#[derive(Debug, Clone, PartialEq)]
pub enum Number {
    Integer(i64),
    Float(f64),
}

impl Eq for Number {
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    // (span, raw, parsed)
    CodeBlock(Span, String, String),
    Whitespace(Span, String, String),
    TextMarker(Span, char),
    InlineText(Span, String, String),
    InlineMathText(Span, String, String),
    DisplayMathText(Span, String, String),
    AttributeName(Span, String, RawName),
    NumericValue(Span, String, Number),
    StringValue(Span, String, String),
    CurlyTagStart(Span, String, RawName),
    CurlyTagEnd(Span),
    PointyTagStart(Span, String, RawName),
    PointyTagEnd(Span, char),
}

#[derive(Debug, Clone, PartialEq, Eq)]
// The idea is that simply printing an array of RawToken you get the exact same thing as the input
pub enum RawToken {
    CodeBlock(Span, String),
    Whitespace(Span, String),
    TextMarker(Span, char),
    InlineText(Span, String),
    InlineMathText(Span, String),
    DisplayMathText(Span, String),
    AttributeName(Span, String),
    NumericValue(Span, String),
    StringValue(Span, String),
    CurlyTagStart(Span, String),
    CurlyTagEnd(Span),
    PointyTagStart(Span, String),
    PointyTagEnd(Span, String),
}

type GotFirstLetter = bool;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    CodeBlock,
    TextMarker,
    Whitespace,
    InlineText(TextEscapeState),
    InlineMathText,
    DisplayMathText,
    AttributeName(GotFirstLetter),
    BooleanValue,
    NumericValue,
    StringValue,
    CurlyTagStart,
    CurlyTagEnd,
    PointyTagStart,
    PointyTagEnd,
}

#[derive(Debug)]
struct State {
    mode: Mode,
    after_whitespace: Option<Mode>,
    text_escape: Option<TextEscapeState>,
    inside_tag: TagType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TagType {
    NotTag,
    CurlyTag,
    PointyTag,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TextEscapeState {
    Normal,
    Slash,
    Unicode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenizerError {
    IllegalChar(Position, char),
    IllegalEscapeSequence(Position, String)
}

fn next_state(src: &mut PeekReader, state: &mut State) -> Option<TokenizerError> {
    let (last_c, pop_c, next_c) = (src.peek(-1), src.peek(0), src.peek(1));
    unimplemented!()
}

fn parse_whitespace(src: &mut PeekReader, state: &mut State) -> Result<Token, TokenizerError> {
    let (last_c, pop_c, next_c) = (src.peek(-1), src.peek(0), src.peek(1));
    unimplemented!()
}

fn parse_inline_text(src: &mut PeekReader, state: &mut State) -> Result<Token, TokenizerError> {
    let (last_c, pop_c, next_c) = (src.peek(-1), src.peek(0), src.peek(1));
    unimplemented!()
}

fn parse_tag_start(src: &mut PeekReader, state: &mut State) -> Result<Token, TokenizerError> {
    let (last_c, pop_c, next_c) = (src.peek(-1), src.peek(0), src.peek(1));
    unimplemented!()
}

fn parse_attr_name(src: &mut PeekReader, state: &mut State) -> Result<Token, TokenizerError> {
    let (last_c, pop_c, next_c) = (src.peek(-1), src.peek(0), src.peek(1));
    unimplemented!()
}

fn parse_string_value(src: &mut PeekReader, state: &mut State) -> Result<Token, TokenizerError> {
    let (last_c, pop_c, next_c) = (src.peek(-1), src.peek(0), src.peek(1));
    unimplemented!()
}

fn parse_numeric_value(src: &mut PeekReader, state: &mut State) -> Result<Token, TokenizerError> {
    let (last_c, pop_c, next_c) = (src.peek(-1), src.peek(0), src.peek(1));
    unimplemented!()
}

fn parse_tag_end(src: &mut PeekReader, state: &mut State) -> Result<Token, TokenizerError> {
    let (last_c, pop_c, next_c) = (src.peek(-1), src.peek(0), src.peek(1));
    unimplemented!()
}

// the other functions don't actually calculate the span, they just return a dummy value for it
fn parse_next_token(src: &mut PeekReader, state: &mut State) -> Result<Token, TokenizerError> {
    let (last_c, pop_c, next_c) = (src.peek(-1), src.peek(0), src.peek(1));
    let mut span = Span::new();
    span.start = src.get_pos();
    // do stuff
    span.end = src.get_pos();    
    unimplemented!()
}

#[derive(Debug)]
pub struct RawTokenizer {
    src: Box<PeekReader>,
    txt: String,
    tmp: String,
    span: Span,
    state: Mode,
    result: Option<RawToken>,
    done: bool,
    repeat_c: bool,
    inside_tag: TagType,
}

// TODO: untangle this mess!
impl RawTokenizer {
    pub fn new(reader: Box<dyn ByteReader>) -> Self {
        RawTokenizer {
            src: Box::new(PeekReader::new(reader)),
            txt: "".to_string(),
            tmp: "".to_string(),
            state: Mode::InlineText(TextEscapeState::Normal),
            span: Span::new(),
            result: None,
            done: false,
            repeat_c: false,
            inside_tag: TagType::NotTag,
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
                    Mode::InlineText(substate) => self.result_text(*substate),
                    Mode::CurlyTagStart => self.result_curly_start(),
                    Mode::CurlyTagEnd => self.result_curly_end(),
                    Mode::PointyTagStart => self.result_pointy_start(),
                    Mode::PointyTagEnd => self.result_pointy_end(),
                    Mode::StringValue => self.result_string_value(),
                    Mode::NumericValue => self.result_numeric_value(),
                    Mode::TextMarker => self.result_text_marker(c),
                    Mode::AttributeName(_) => self.result_attribute_name(),
                    _ => panic!("unexpected state: {:?}", self.state),
                }
                return;
            }
            // Process new char
            match &self.state {
                Mode::InlineText(substate) => self.mode_text(c, *substate),
                Mode::CurlyTagStart => self.mode_curly_start(c),
                Mode::CurlyTagEnd => self.mode_curly_end(c),
                Mode::PointyTagStart => self.mode_pointy_start(c),
                Mode::PointyTagEnd => self.mode_pointy_end(c),
                Mode::StringValue => self.mode_string_value(c),
                Mode::NumericValue => self.mode_numeric_value(c),
                Mode::TextMarker => self.result_text_marker(c),
                Mode::AttributeName(got_first_letter) => {
                    self.mode_attribute_name(c, *got_first_letter)
                }
                _ => panic!("unexpected state: {:?}", self.state),
            }
        }
    }

    fn peek_next_state_attr(&mut self, dist: isize) {
        let next_c = self.src.peek(dist);
        println!("peek_next_state_attr({}) {:?}", dist, next_c);
        if next_c == '}' && self.inside_tag == TagType::CurlyTag {
            self.state = Mode::CurlyTagEnd;
        } else if next_c == '>' && self.inside_tag == TagType::PointyTag {
            self.state = Mode::PointyTagEnd;
        } else if next_c == ';' && self.inside_tag == TagType::CurlyTag {
            self.state = Mode::TextMarker;
        } else if next_c == '|' && self.inside_tag == TagType::PointyTag {
            self.state = Mode::TextMarker;
        } else if next_c == '}' || next_c == '>' || next_c == ';' || next_c == '|' {
            let mut pos = self.src.get_pos().clone();
            pos.step(next_c);
            panic!("unexpected character {:?} at {:?}", next_c, pos)
        } else {
            self.state = Mode::AttributeName(false);
        }
    }

    fn result_text_marker(&mut self, c: char) {
        self.span.step(c);
        self.result = Some(RawToken::TextMarker(self.span, c));
        self.state = Mode::InlineText(TextEscapeState::Normal);
    }

    fn result_numeric_value(&mut self) {
        self.result = Some(RawToken::NumericValue(self.span, self.txt.clone()));
    }

    fn mode_numeric_value(&mut self, c: char) {
        if c.is_ascii_digit() || c == '.' || c == '_' {
            self.txt.push(c);
            self.span.step(c);
        } else if c.is_whitespace() {
            self.state = Mode::AttributeName(false);
            self.repeat_c = true;
            self.result_numeric_value();
        } else {
            self.peek_next_state_attr(0);
            self.repeat_c = true;
            self.result_numeric_value();
        }
    }

    fn result_string_value(&mut self) {
        self.result = Some(RawToken::StringValue(self.span, self.txt.clone()));
    }

    fn mode_string_value(&mut self, c: char) {
        let last_c = self.src.peek(-1);
        self.txt.push(c);
        self.span.step(c);
        println!("mode_string_value: `{}`", self.txt);

        if c == '\"' && last_c != '\\' && self.txt.len() > 1 {
            println!("{:?} ({:?}) {:?}", last_c, c, self.src.peek(1));
            self.result_string_value();
            self.peek_next_state_attr(1);
        }
    }

    fn result_attribute_name(&mut self) {
        self.result = Some(RawToken::AttributeName(self.span, self.txt.clone()));
    }

    fn mode_attribute_name(&mut self, c: char, first: bool) {
        let mut got_first_letter = first;

        let id_char = match got_first_letter {
            false => is_valid_id_first_char(c),
            true => is_valid_id_next_char(c),
        };
        if !got_first_letter {
            if id_char {
                got_first_letter = true;
                self.state = Mode::AttributeName(got_first_letter);
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
                self.state = Mode::StringValue;
            } else if next_c == 'f' || next_c == 't' {
                self.state = Mode::BooleanValue;
            } else if next_c.is_ascii_digit() || next_c == '.' {
                self.state = Mode::NumericValue;
            } else if c.is_whitespace() {
                self.state = Mode::AttributeName(false);
            } else {
                panic!("unexpected character {:?} at {:?}", c, self.src.get_pos());
            }
        } else if c == ';' {
            self.state = Mode::InlineText(TextEscapeState::Normal);
            self.repeat_c = false;
            self.result_attribute_name();
        } else if c == '}' && self.inside_tag == TagType::CurlyTag {
            self.state = Mode::CurlyTagEnd;
            self.repeat_c = true;
            self.result_attribute_name();
        } else if c == '>' && self.inside_tag == TagType::PointyTag {
            self.state = Mode::PointyTagEnd;
            self.repeat_c = true;
            self.result_attribute_name();
        } else if !id_char && !c.is_whitespace() {
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
        self.state = Mode::InlineText(TextEscapeState::Normal);
        self.result_curly_end();
        self.inside_tag = TagType::NotTag;
    }

    fn result_curly_start(&mut self) {
        self.result = Some(RawToken::CurlyTagStart(self.span, self.txt.clone()));
        self.inside_tag = TagType::CurlyTag;
    }

    fn mode_curly_start(&mut self, c: char) {
        let last_c = self.src.peek(-1);
        if last_c.is_whitespace() && c.is_alphabetic() && self.txt.len() > 0 {
            println!("{:?} {:?} {:?}", self.txt, last_c, c);
            self.result_curly_start();
            self.state = Mode::AttributeName(true);
            self.repeat_c = true;
            return;
        }
        if c != '}' {
            self.txt.push(c);
            self.span.step(c);
        }

        if c == '}' || c == ';' {
            match c {
                '}' => self.state = Mode::CurlyTagEnd,
                ';' => self.state = Mode::InlineText(TextEscapeState::Normal),
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
            && escape == TextEscapeState::Normal
            && !between_spaces
        {
            self.result_text(escape);
            match c {
                '{' => self.state = Mode::CurlyTagStart,
                '}' => self.state = Mode::CurlyTagEnd,
                '<' => self.state = Mode::PointyTagStart,
                '|' => self.state = Mode::PointyTagEnd,
                _ => {}
            }
            self.repeat_c = true;
        } else {
            self.txt.push(c);
            self.span.step(c);
            if c == '\\' && escape == TextEscapeState::Normal {
                escape = TextEscapeState::Slash;
            } else if escape == TextEscapeState::Slash {
                if c == 'u' {
                    escape = TextEscapeState::Unicode;
                } else {
                    escape = TextEscapeState::Normal;
                }
            } else if escape == TextEscapeState::Slash && c == ';' {
                escape = TextEscapeState::Normal;
            }

            self.state = Mode::InlineText(escape);
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
        let s = "abc {icon bool_attr id=\"ab\\\"c\" num=1 val=.5; text   }{end id=\"e\"}";
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
            Some(RawToken::StringValue(
                Span::new2(23, 1, 23, 30, 1, 30),
                "\"ab\\\"c\"".to_string()
            ))
        );
        assert_eq!(
            parser.next(),
            Some(RawToken::AttributeName(
                Span::new2(30, 1, 30, 35, 1, 35),
                " num=".to_string()
            ))
        );
        assert_eq!(
            parser.next(),
            Some(RawToken::NumericValue(
                Span::new2(35, 1, 35, 36, 1, 36),
                "1".to_string()
            ))
        );
        assert_eq!(
            parser.next(),
            Some(RawToken::AttributeName(
                Span::new2(37, 1, 37, 42, 1, 42),
                " val=".to_string()
            ))
        );
        assert_eq!(
            parser.next(),
            Some(RawToken::NumericValue(
                Span::new2(41, 1, 41, 43, 1, 43),
                ".5".to_string()
            ))
        );
        assert_eq!(
            parser.next(),
            Some(RawToken::TextMarker(Span::new2(44, 1, 44, 45, 1, 45), ';'))
        );
        assert_eq!(
            parser.next(),
            Some(RawToken::InlineText(
                Span::new2(44, 1, 44, 52, 1, 52),
                " text   ".to_string()
            ))
        );
        assert_eq!(
            parser.next(),
            Some(RawToken::CurlyTagEnd(Span::new2(53, 1, 53, 54, 1, 54)))
        );
        assert_eq!(
            parser.next(),
            Some(RawToken::CurlyTagStart(
                Span::new2(53, 1, 53, 58, 1, 58),
                "{end ".to_string()
            ))
        );
        // println!("{:?}", parser);
        assert_eq!(
            parser.next(),
            Some(RawToken::AttributeName(
                Span::new2(59, 1, 59, 62, 1, 62),
                "id=".to_string()
            ))
        );
        assert_eq!(
            parser.next(),
            Some(RawToken::StringValue(
                Span::new2(61, 1, 61, 64, 1, 64),
                "\"e\"".to_string()
            ))
        );
        assert_eq!(
            parser.next(),
            Some(RawToken::CurlyTagEnd(
                Span::new2(64, 1, 64, 65, 1, 65)
            ))
        );
        assert_eq!(parser.next(), None);
    }
}

//! This module/file contains low level stuff you probably should not use.

#![allow(dead_code)]
#![allow(unused_variables)]

use std::mem;
use crate::peek_reader::PeekReader;
use crate::pos::Span;
use core::fmt::Debug;

#[derive(Debug)]
pub struct RawName {
    pub view: String,
    pub special: bool,
    pub prefix: String,
    pub local: String,
}

#[derive(Debug,Clone,PartialEq,Eq)]
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

#[derive(Debug,Clone,Copy)]
enum State {
    CodeBlock,
    InlineText,
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
    in_escape: bool,
    done: bool,
    clean: bool,
}

impl RawParser {
    pub fn new(reader: Box<dyn ByteReader>) -> Self {
        RawParser {
            src: Box::new(PeekReader::new(reader)),
            txt: "".to_string(),
            tmp: "".to_string(),
            state: State::InlineText,
            span: Span::new(),
            result: None,
            in_escape: false,
            done: false,
            clean: true,
        }
    }

    fn until_yield(&mut self) {
        self.result = None;
        if self.done {
            return
        }
        while self.result.is_none() {
            let c = self.src.pop();
            // Finish if EOF
            if c == '\0' {
                self.done = true;
                match &self.state {
                    State::InlineText => self.result_text(),
                    _ => panic!("unexpected state")
                }
                return
            }
            // println!("{:?}", self.src.get_pos());
            if self.clean {
                self.clean = false;
                if self.src.get_pos().byte != 1 {
                    self.span = Span::new_from(self.src.get_pos());
                }
            }
            // Process new char
            match &self.state {
                State::InlineText => self.mode_text(c),
                _ => panic!("unexpected state")
            }
        }
    }

    fn result_text(&mut self) {
        self.result = Some(RawToken::InlineText(self.span, self.txt.clone()));
        self.txt.clear();
    }

    fn mode_text(&mut self, c: char) {
        if c == '{' && !self.src.peek(1).is_whitespace() {
            self.result_text();
        } else {
            self.txt.push(c);
            self.span.step(c);
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
        assert_eq!(parser.next(), Some(RawToken::InlineText(Span::new2(0,1,1,0,1,1), "".to_string())));
        assert_eq!(parser.next(), None);

        let s = "a";
        let mut parser = RawParser::new(Box::new(s.bytes()));
        assert_eq!(parser.next(), Some(RawToken::InlineText(Span::new2(0,1,1,1,1,2), "a".to_string())));
        assert_eq!(parser.next(), None);

        let s = "a{";
        let mut parser = RawParser::new(Box::new(s.bytes()));
        assert_eq!(parser.next(), Some(RawToken::InlineText(Span::new2(0,1,1,1,1,2), "a".to_string())));
        // assert_eq!(parser.next(), None);

        let s = "hello world! {";
        let mut parser = RawParser::new(Box::new(s.bytes()));
        assert_eq!(parser.next(), Some(RawToken::InlineText(Span::new2(0,1,1,13,1,14), "hello world! ".to_string())));
        // assert_eq!(parser.next(), None);

        let s = "hello world!{ ";
        let mut parser = RawParser::new(Box::new(s.bytes()));
        assert_eq!(parser.next(), Some(RawToken::InlineText(Span::new2(0,1,1,14,1,15), "hello world!{ ".to_string())));

        let s = "hello world!{!";
        let mut parser = RawParser::new(Box::new(s.bytes()));
        assert_eq!(parser.next(), Some(RawToken::InlineText(Span::new2(0,1,1,12,1,13), "hello world!".to_string())));
    }
}
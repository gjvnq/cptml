//! This module/file contains low level stuff you probably should not use.

#![allow(dead_code)]
#![allow(unused_variables)]

use crate::hacks::{bytes_to_char, char_size};
use crate::pos::Position;
use crate::pos::Span;
use core::fmt::Debug;

const READ_BUF_SIZE: usize = 64;
const PEEK_RESERVE: usize = 4;
const READ_LIMIT: usize = READ_BUF_SIZE - PEEK_RESERVE;

#[derive(Debug)]
pub struct RawName {
    pub view: String,
    pub special: bool,
    pub prefix: String,
    pub local: String,
}

#[derive(Debug)]
pub enum RawToken {
    CodeBlock(Span, String),
    InlineText(Span, String),
    InlineMathText(Span, String),
    DisplayMathText(Span, String),
    AttributeName(Span, String),
    BooleanValue(Span, bool),
    IntegerValue(Span, i64),
    FloatValue(Span, f64),
    StringValue(Span, String),
    CurlyTagStart(Span, String),
    CurlyTagEnd(Span),
    PointyTagStart(Span, String),
    PointyTagEnd(Span, String),
}

pub trait ByteReader: Debug + Iterator<Item = u8> {}

impl ByteReader for std::str::Bytes<'_> {}

#[derive(Debug)]
pub struct RawParser {
    buf: [char; READ_BUF_SIZE],
    buf_pos: usize,
    pos: Position,
    src: Box<dyn ByteReader>,
    eof: bool,
}

impl RawParser {
    pub fn new(reader: Box<dyn ByteReader>) -> Self {
        let mut ans = RawParser {
            buf: ['\0'; READ_BUF_SIZE],
            buf_pos: 0,
            eof: false,
            pos: Position::new(),
            src: reader,
        };
        ans.read_cycle();
        ans
    }

    fn peek(&mut self, dist: usize) -> char {
        if 0 == dist || dist > 3 {
            panic!("RawParser::peek(n={}), n is limited to 3", dist);
        }
        self.buf[self.buf_pos + dist]
    }

    fn pop(&mut self) -> char {
        if self.buf_pos > READ_LIMIT {
            self.read_cycle()
        }
        let ans = self.buf[self.buf_pos];
        self.buf_pos += 1;
        self.pos.step(ans);
        ans
    }

    fn read_cycle(&mut self) {
        // Copy chars we had reserved for peeking to the begining of the buffer
        if self.pos.byte != 0 {
            let mut i = 0;
            let mut j = self.buf_pos;
            while j < self.buf.len() {
                self.buf[i] = self.buf[j];
                i += 1;
                j += 1;
            }
        }

        let mut i = match self.pos.byte {
            0 => 0,
            _ => self.buf.len() - self.buf_pos,
        };
        self.buf_pos = 0;
        let mut buf: [u8; 4] = [0; 4];
        while i < self.buf.len() {
            // Read a single byte
            let res = self.src.next();
            if res.is_none() {
                self.eof = true;
                buf[0] = b'\0';
            } else {
                buf[0] = res.unwrap();
            }
            // Find out how many bytes we will need to read for this character
            let s = char_size(buf[0]);
            for j in 1..s {
                // Read more bytes if necessary (code ponts above 007F)
                let res = self.src.next();
                buf[j] = res.unwrap();
            }
            // Decode the character and save it
            self.buf[i] = bytes_to_char(&buf).0;
            i += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::raw_parser::*;

    #[test]
    fn it_works() {
        let s = "hello world!§......................................................................§ªº冬";
        let mut parser = RawParser::new(Box::new(s.bytes()));
        assert_eq!('h', parser.pop());
        assert_eq!('e', parser.pop());
        assert_eq!('l', parser.pop());
        assert_eq!('l', parser.pop());
        assert_eq!('o', parser.pop());
        assert_eq!(' ', parser.pop());
        assert_eq!('w', parser.pop());
        assert_eq!('o', parser.pop());
        assert_eq!('r', parser.pop());
        assert_eq!('l', parser.pop());
        assert_eq!('d', parser.pop());
        assert_eq!('!', parser.pop());
        assert_eq!('§', parser.pop());
        for i in 0..70 {
            assert_eq!('.', parser.pop());
        }
        assert_eq!('ª', parser.peek(1));
        assert_eq!('º', parser.peek(2));
        assert_eq!('冬', parser.peek(3));
        assert_eq!('§', parser.pop());
        assert_eq!('ª', parser.pop());
        assert_eq!('º', parser.pop());
        assert_eq!('冬', parser.pop());
        assert_eq!('\0', parser.pop());
    }
}

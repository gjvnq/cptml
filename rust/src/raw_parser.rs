//! This module/file contains low level stuff you probably should not use.

#![allow(dead_code)]
#![allow(unused_variables)]

use std::io;
use std::io::Read;
use crate::pos::Position;
use crate::pos::Span;
use crate::hacks::read_single_char;

const READ_BUF_SIZE: usize = 64*4;
const PEEK_RESERVE: usize = 4*4;
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

// #[derive(Debug)]
pub struct RawParser<'a> {
	str: String,
	buf: [u8; READ_BUF_SIZE],
	buf_pos: usize,
	pos: Position,
	src: &'a mut dyn Read,
	eof: bool,
	read_err: Option<io::Error>,
}

impl RawParser<'_> {
	fn peek(&mut self, size: usize) -> char {
		if 0 == size || size > 3 {
			panic!("RawParser::peek(n={}), n is limited to 3", size);
		}

		let mut ans: char = '\0';
		let mut pos = self.buf_pos;
		let mut i = 0;
		while i < size {
			let tmp = read_single_char(&self.buf[pos..]);
			ans = tmp.0;
			pos += tmp.1;
			i += 1;
		}
		ans
	}

	fn pop(&mut self) -> char {
		if self.buf_pos > READ_LIMIT {
			self.read_cycle()
		}

		let ans = read_single_char(&self.buf[self.buf_pos..]);
		ans.0
	}

	fn read_cycle(&mut self) {
		let mut bufu8 : [u8; READ_BUF_SIZE] = [0; READ_BUF_SIZE];
		
		let res = self.src.read(&mut bufu8);
		if res.is_err() {
			self.read_err = Some(res.unwrap_err());
			return
		}
	}
}
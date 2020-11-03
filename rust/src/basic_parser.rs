#![allow(dead_code)]

use crate::errors::ParserError;
use std::collections::HashMap;
use crate::peek_reader::PeekReader;
use crate::raw_tokenizer::BasicName;
use crate::raw_tokenizer::RawTokenizer;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TagType {
	CurlyFull,
    CurlyStart,
    CurlyEnd,
    PointyFull,
    PointyStart,
    PointyEnd,
}

impl TagType {
	pub fn is_pointy(&self) -> bool {
		match self {
			TagType::PointyFull | TagType::PointyStart | TagType::PointyEnd => true,
			_ => false
		}
	}
	pub fn is_curly(&self) -> bool {
		match self {
			TagType::CurlyFull | TagType::CurlyStart | TagType::CurlyEnd => true,
			_ => false
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tag {
	name: BasicName,
	kind: TagType,
	raw: String
}

#[derive(Debug)]
pub struct BasicParser {
	show_full_prefix: bool,
	views: HashMap<String,usize>, // 0 is reserved for curly
	stacks: Vec<Vec<BasicName>>, // 0 is the curly tag stack
	tokenizer: RawTokenizer
}

impl BasicParser {
	pub fn new(src: Box<PeekReader>) -> BasicParser {
		let tokenizer = RawTokenizer::new(src);
		BasicParser {
			show_full_prefix: false,
			views: HashMap::new(),
			stacks: vec![vec![]],
			tokenizer: tokenizer
		}
	}

	fn add_view(&mut self, prefix: &str) {
		let n = self.views.len() + 1;
		self.views.insert(prefix.to_string(), n);
	}

	fn view_to_index(&self, prefix: &str) -> Option<usize> {
		match self.views.get(prefix) {
			Some(v) => Some(*v),
			None => None
		}
	}

	pub fn next(&mut self) -> Result<Tag, ParserError> {
		unimplemented!()
	}
}
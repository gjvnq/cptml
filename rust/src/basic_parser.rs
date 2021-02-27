#![allow(dead_code)]

use crate::errors::ParserError;
use crate::peek_reader::PeekReader;
use crate::pos::Span;
use crate::raw_tokenizer::AttrValue;
use crate::raw_tokenizer::{BasicName, RawToken, RawTokenizer};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TagType {
    CurlyFull,
    CurlyStart,
    CurlyEnd,
    PointyFull,
    PointyStart,
    PointyEnd,
    Virtual,
}

impl TagType {
    pub fn is_pointy(&self) -> bool {
        match self {
            TagType::PointyFull | TagType::PointyStart | TagType::PointyEnd => true,
            _ => false,
        }
    }
    pub fn is_curly(&self) -> bool {
        match self {
            TagType::CurlyFull | TagType::CurlyStart | TagType::CurlyEnd => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attr {
    name: BasicName,
    val: AttrValue
}

impl Attr {
    pub fn is_named(&self) -> bool {
        return !self.name.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tag {
    name: BasicName,
    kind: TagType,
    raw: String,
    attrs: Vec<Attr>,
    span: Span,
}

impl Tag {
    /// Get Positional Attribute (counts from 1)
    pub fn get_pos_attr(&self, pos: usize) -> Option<&Attr> {
        let mut counter = 0;
        for attr in &self.attrs {
            if !attr.is_named() {
                counter += 1;
            } else if counter == pos {
                return Some(attr)
            }
        }
        None
    }

    pub fn get_named_attr(&self, name: &BasicName) -> Option<&Attr> {
        if name.is_empty() {
            // Avoid the mistak of getting a positional argument when you want a named one
            return None
        }
        for attr in &self.attrs {
            if !attr.is_named() {
                continue
            }
            if attr.name == *name {
                return Some(attr)
            }
        }
        None
    }
}

#[derive(Debug)]
pub struct BasicParser {
    show_full_prefix: bool,
    views: HashMap<String, usize>, // 0 is reserved for curly
    stacks: Vec<Vec<BasicName>>,   // 0 is the curly tag stack
    tokenizer: RawTokenizer,
}

impl BasicParser {
    pub fn new(src: Box<PeekReader>) -> BasicParser {
        let tokenizer = RawTokenizer::new(src);
        BasicParser {
            show_full_prefix: false,
            views: HashMap::new(),
            stacks: vec![vec![]],
            tokenizer: tokenizer,
        }
    }

    fn add_view(&mut self, prefix: &str) {
        let n = self.views.len() + 1;
        self.views.insert(prefix.to_string(), n);
    }

    fn view_to_index(&self, prefix: &str) -> Option<usize> {
        match self.views.get(prefix) {
            Some(v) => Some(*v),
            None => None,
        }
    }

    fn process_text(&mut self, span: Span, raw: String, val: String) -> Result<Tag, ParserError> {
        Ok(Tag {
            name: BasicName::new("", true, "", "text"),
            kind: TagType::Virtual,
            raw: raw,
            span: span,
            attrs: vec![Attr {
                name: BasicName::new("", true, "", "val"),
                val: AttrValue::String(val),
            }],
        })
    }

    fn process_curly_tag(
        &mut self,
        span: Span,
        raw: String,
        name: BasicName,
    ) -> Result<Tag, ParserError> {
    	let mut tag_raw = raw.clone();
    	let mut tag_span = span;
        let mut tag = Tag {
            name: name,
            kind: TagType::CurlyStart,
            raw: "".to_string(),
            span: Span::new(),
            attrs: vec![],
        };

        // Process arguments if any
        let mut token = self.tokenizer.next();
        'outer: loop {
            let attr_name: BasicName;
            let attr_val: AttrValue;
            // Get attribute name or get out
            loop {
                println!("B: {:?}", token);
                match token {
                    Ok(RawToken::AttributeName(span, raw, name)) => {
                        attr_name = name;
                        tag_raw += &raw;
                        tag_span.end = span.end;
                        break;
                    }
                    Ok(RawToken::Whitespace(span, raw, _)) => {
                        tag_raw += &raw;
                        tag_span.end = span.end;
                        token = self.tokenizer.next();
                    }
                    Ok(_) => break 'outer,
                    Err(err) => return Err(err),
                };
            }
            // Get attribute value
            token = self.tokenizer.next();
            println!("C: {:?}", token);
            match token {
                Ok(RawToken::AttributeValue(_, raw, val)) => {
                    attr_val = val;
                    tag_raw += &raw;
                        tag_span.end = span.end;
                }
                Ok(tok) => {
                    return Err(ParserError::NotAttributeValue(
                        tok.get_span(),
                        tok.get_raw(),
                    ))
                }
                Err(err) => return Err(err),
            };
            tag.attrs.push(Attr {
                name: attr_name,
                val: attr_val,
            });
            token = self.tokenizer.next();
        }

        // See where to stop
        loop {
		    match token {
		    	Ok(RawToken::Whitespace(span, raw, _)) => {
		    		tag_raw += &raw;
		            tag_span.end = span.end;
		    	},
		    	Ok(RawToken::TextMarker(span, c)) => {
		    		tag_raw += &c.to_string();
		            tag_span.end = span.end;
		            break;
		    	},
		    	Ok(RawToken::CurlyTagEnd(span, c)) => {
		    		tag_raw += &c.to_string();
		            tag_span.end = span.end;
		            tag.kind = TagType::CurlyFull;
		            break;
		    	},
		    	_ => {
		    		self.tokenizer.unnext();
		    		break;
		    	}
		    };
		    token = self.tokenizer.next();
		}

		tag.span = tag_span;
		tag.raw = tag_raw;

        Ok(tag)
    }

    pub fn next(&mut self) -> Result<Tag, ParserError> {
        let token = self.tokenizer.next();
        println!("A: {:?}", token);
        match token {
            Err(err) => Err(err),
            Ok(RawToken::InlineText(span, raw, val)) => self.process_text(span, raw, val),
            Ok(RawToken::CurlyTagStart(span, raw, name)) => self.process_curly_tag(span, raw, name),
            _ => unimplemented!(),
        }
    }
}

#[cfg(test)]
#[path = "basic_parser_test.rs"]
mod tests;

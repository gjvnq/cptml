//! This module/file contains low level stuff you probably should not use.

use crate::pos::Span;

pub enum RawToken<'a> {
	CodeBlock(Span, &'a str),
	InlineText(Span, &'a str),
	InlineMathText(Span, &'a str),
	DisplayMathText(Span, &'a str),
	Attribute(Span, &'a str, &'a str),
	CurlyTagStart(Span, &'a str),
	CurlyTagEnd(Span),
	PointyTagStart(Span, &'a str),
	PointyTagEnd(Span, &'a str),
}
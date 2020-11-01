//! This module/file contains low level stuff you probably should not use.

#![allow(dead_code)]
#![allow(unused_variables)]


use crate::hacks::is_valid_id_char;
use crate::hacks::is_valid_id_first_char;
use crate::hacks::is_valid_id_next_char;
use crate::hacks::u32_to_char;
use crate::hacks::ByteReader;
use crate::peek_reader::PeekReader;
use crate::pos::Position;
use crate::pos::Span;
use core::fmt::Debug;
use std::mem;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BasicName {
    pub view: String,
    pub special: bool,
    pub prefix: String,
    pub local: String,
}

impl BasicName {
    fn new() -> BasicName {
        BasicName {
            view: "".to_string(),
            special: false,
            prefix: "".to_string(),
            local: "".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DualString {
    raw: String,
    parsed: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Number {
    Integer(i64),
    Float(f64),
}

impl Eq for Number {}

// Self Note: when making a tree structure, the original "form" will always be printed if possible (use dirty bit?)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    // (span, raw, parsed)
    CodeBlock(Span, String, String),
    Whitespace(Span, String, String),
    TextMarker(Span, char),
    InlineText(Span, String, String),
    InlineMathText(Span, String, String),
    DisplayMathText(Span, String, String),
    AttributeName(Span, String, BasicName),
    NumericValue(Span, String, Number),
    StringValue(Span, String, String),
    CurlyTagStart(Span, String, BasicName),
    CurlyTagEnd(Span, char),
    PointyTag(Span, String, BasicName, char, char),
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
    StartOfInput,
    CodeBlock,
    TextMarker,
    Whitespace,
    InlineTextNew,
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
    text_escape: TextEscapeState,
    inside_tag: TagType,
}

impl State {
    fn new() -> State {
        State {
            mode: Mode::StartOfInput,
            after_whitespace: None,
            text_escape: TextEscapeState::Normal,
            inside_tag: TagType::NotTag,
        }
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WhitesapeMode {
    NewLine,
    GotFirst,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenizerError {
    IllegalChar(Position, char),
    IllegalChar2(Position, char, Vec<char>),
    IllegalCharMsg(Position, char, String),
    MissingTerminator(Position, char),
    MissingLocalName(Position),
    IllegalEscapeSequence(Position, String),
}

fn next_state(src: &mut PeekReader, state: &mut State) -> Option<TokenizerError> {
    let (last_c, pop_c, next_c) = (src.peek(0), src.peek(1), src.peek(2));
    unimplemented!()
}

fn parse_whitespace(src: &mut PeekReader, state: &mut State) -> Result<Token, TokenizerError> {
    let (last_c, pop_c, next_c) = (src.peek(0), src.peek(1), src.peek(2));
    unimplemented!()
}

fn parse_inline_text(src: &mut PeekReader, state: &mut State) -> Result<Token, TokenizerError> {
    state.mode = Mode::InlineTextNew;
    state.text_escape = TextEscapeState::Normal;
    let mut ans_raw = String::new();
    let mut ans_parsed = String::new();
    let mut buf_unicode = String::new();
    let mut ws = WhitesapeMode::GotFirst;
    let mut last_vis = 0;

    loop {
        let (last_c, pop_c, next_c) = (src.peek(0), src.peek(1), src.peek(2));
        // println!("- {:?} {:?}", ans_raw, pop_c);
        // println!("+ {:?} {:?}", ans_parsed, pop_c);
        if pop_c == '\0' {
            break;
        }
        if state.text_escape == TextEscapeState::Normal {
            // Check if we need to change state
            let pop_special =
                pop_c == '{' || pop_c == '}' || pop_c == '<' || pop_c == '>' || pop_c == '|';
            // println!("pop_special={}, last_c={:?} next_c={:?}", pop_special, last_c, next_c);
            if pop_special && (last_c != ' ' || next_c != ' ') {
                break;
            }
            ans_raw.push(pop_c);

            // Process escape sequences
            if pop_c == '\\' {
                state.text_escape = match next_c {
                    'u' => TextEscapeState::Unicode,
                    _ => TextEscapeState::Slash,
                };
                buf_unicode.clear();
                src.pop();
                continue;
            }

            // Process whitespace relevance
            let c_ws = pop_c == ' ' || pop_c == '\t';
            if ws == WhitesapeMode::NewLine && !c_ws {
                ws = WhitesapeMode::GotFirst;
            }
            if pop_c == '\n' {
                // At the end of the line, trim the strign to tha last "visible" char
                ans_parsed = ans_parsed[..last_vis].to_string();
                ws = WhitesapeMode::NewLine;
                ans_parsed.push(pop_c);
            }
            if ws == WhitesapeMode::GotFirst {
                ans_parsed.push(pop_c);
                if !c_ws {
                    last_vis = ans_parsed.len();
                }
            }

            // Confirm step
            src.pop();
        } else if state.text_escape == TextEscapeState::Slash {
            ans_raw.push(pop_c);
            let real_c = match pop_c {
                '"' => '"',
                '<' => '<',
                '>' => '>',
                '\'' => '\'',
                '\\' => '\\',
                '`' => '`',
                'a' => '\x07',
                'f' => '\x0C',
                'n' => '\n',
                'r' => '\r',
                's' => ' ',
                't' => '\t',
                'v' => '\x0B',
                '{' => '{',
                '|' => '|',
                '}' => '}',
                _ => '\0',
            };
            if real_c == '\0' {
                let s_err = format!("\\{}", next_c);
                return Err(TokenizerError::IllegalEscapeSequence(src.get_pos(), s_err));
            } else {
                ans_parsed.push(real_c);
                src.pop();
                state.text_escape = TextEscapeState::Normal;

                ws = WhitesapeMode::GotFirst;
                last_vis = ans_parsed.len();
            }
        } else if state.text_escape == TextEscapeState::Unicode {
            ans_raw.push(pop_c);

            fn ret_uni_err(
                src: &PeekReader,
                buf_unicode: &String,
            ) -> Result<Token, TokenizerError> {
                let s_err = "\\u".to_string() + &buf_unicode;
                return Err(TokenizerError::IllegalEscapeSequence(src.get_pos(), s_err));
            }

            if buf_unicode.len() == 0 && pop_c == 'u' {
                // do nothing
            } else if pop_c == ';' {
                // finish
                let hex_val = match u32::from_str_radix(&buf_unicode, 16) {
                    Ok(x) => x,
                    _ => return ret_uni_err(src, &buf_unicode),
                };
                let real_c = match u32_to_char(hex_val) {
                    Some(c) => c,
                    _ => return ret_uni_err(src, &buf_unicode),
                };
                ans_parsed.push(real_c);
                state.text_escape = TextEscapeState::Normal;

                // Process whitespace relevance
                ws = WhitesapeMode::GotFirst;
                last_vis = ans_parsed.len();
            } else if pop_c.is_digit(16) {
                buf_unicode.push(pop_c);
            } else {
                buf_unicode.push(pop_c);
                src.pop();
                return ret_uni_err(src, &buf_unicode);
            }
            src.pop();
        } else {
            unreachable!()
        }
    }

    return Ok(Token::InlineText(Span::new(), ans_raw, ans_parsed));
}

fn parse_tag(src: &mut PeekReader, state: &mut State) -> Result<Token, TokenizerError> {
    let (last_c, pop_c, next_c) = (src.peek(0), src.peek(1), src.peek(2));
    let mut name = BasicName::new();
    let mut first = true;
    let mut raw_name = "".to_string();
    let mut has_view = false;
    let start_pos = src.get_pos();

    let tag_type = match pop_c {
        '<' | '|' => TagType::PointyTag,
        '{' => TagType::CurlyTag,
        '}' => return Ok(Token::CurlyTagEnd(Span::new(), pop_c)),
        _ => {
            return Err(TokenizerError::IllegalChar2(
                src.get_pos(),
                pop_c,
                vec!['<', '>', '{', '}'],
            ))
        }
    };
    let open = pop_c;
    let mut close = '\0';
    raw_name.push(src.pop());

    loop {
        let (last_c, pop_c, next_c) = (src.peek(0), src.peek(1), src.peek(2));
        if pop_c == '\0' || pop_c == ' ' {
            break;
        }
        if tag_type == TagType::CurlyTag && (pop_c == ';' || pop_c == '}') {
            break;
        }
        if tag_type == TagType::PointyTag && (pop_c == '|' || pop_c == '>') {
            close = pop_c;
            raw_name.push(src.pop());
            break;
        }
        if first && pop_c == '!' {
            name.special = true;
            name.local.push(pop_c);
        } else if is_valid_id_char(name.local.len(), pop_c) {
            name.local.push(pop_c);
        } else if pop_c == ':' && name.prefix.len() == 0 {
            name.prefix = name.local.clone();
            name.local.clear();
        } else if pop_c == '(' && tag_type == TagType::PointyTag && has_view == false {
            has_view = true;
        } else if pop_c == ')' && tag_type == TagType::PointyTag && has_view == true {
            name.view = name.local.clone();
            name.local.clear();
            has_view = false;
        } else {
            return Err(TokenizerError::IllegalCharMsg(
                src.get_pos(),
                pop_c,
                "valid id char".to_string(),
            ));
        }

        first = false;
        raw_name.push(src.pop());
    }

    if has_view {
        return Err(TokenizerError::MissingTerminator(src.get_pos(), ')'));
    }
    if name.local.len() == 0 && !(tag_type == TagType::PointyTag && open == '|' && close == '>') {
        return Err(TokenizerError::MissingLocalName(start_pos));
    }

    match tag_type {
        TagType::CurlyTag => return Ok(Token::CurlyTagStart(Span::new(), raw_name, name)),
        TagType::PointyTag => {
            return Ok(Token::PointyTag(Span::new(), raw_name, name, open, close))
        }
        _ => unreachable!(),
    }
}

fn parse_attr_name(src: &mut PeekReader, state: &mut State) -> Result<Token, TokenizerError> {
    let mut name = BasicName::new();
    let mut raw_name = "".to_string();
    let start_pos = src.get_pos();
    let mut first = true;

    loop {
        let (last_c, pop_c, next_c) = (src.peek(0), src.peek(1), src.peek(2));
        if !first && pop_c == '=' {
            raw_name.push(src.pop());
            break;
        }
        if first && pop_c == '!' {
            name.special = true;
            name.local.push(pop_c);
        } else if is_valid_id_char(name.local.len(), pop_c) {
            name.local.push(pop_c);
        } else if pop_c == ':' && name.prefix.len() == 0 {
            name.prefix = name.local.clone();
            name.local.clear();
        } else {
            return Err(TokenizerError::IllegalCharMsg(
                src.get_pos(),
                pop_c,
                "valid id char".to_string(),
            ));
        }

        first = false;
        raw_name.push(src.pop());
    }
    if name.local.len() == 0 {
        return Err(TokenizerError::MissingLocalName(start_pos));
    }

    return Ok(Token::AttributeName(Span::new(), raw_name, name));
}

fn parse_string_value(src: &mut PeekReader, state: &mut State) -> Result<Token, TokenizerError> {
    let (last_c, pop_c, next_c) = (src.peek(0), src.peek(1), src.peek(2));
    let mut raw_val = "".to_string();
    let mut val = "".to_string();
    let mut buf_unicode = "".to_string();
    let mut mode = TextEscapeState::Normal;

    // Check start char
    if pop_c != '"' {
        return Err(TokenizerError::IllegalChar2(
            src.get_pos(),
            pop_c,
            vec!['"'],
        ));
    }
    raw_val.push(src.pop());

    loop {
        let (last_c, pop_c, next_c) = (src.peek(0), src.peek(1), src.peek(2));
        println!("pop_c={:?} mode={:?}", pop_c, mode);
        if pop_c == '\0' {
            return Err(TokenizerError::IllegalChar2(
                src.get_pos(),
                pop_c,
                vec!['"'],
            ));
        }
        if mode == TextEscapeState::Normal {
            if pop_c == '"' {
                raw_val.push(src.pop());
                break;
            } else if pop_c == '\\' {
                mode = match next_c {
                    'u' => TextEscapeState::Unicode,
                    _ => TextEscapeState::Slash,
                };
                buf_unicode.clear();
            } else {
                val.push(pop_c);
            }
        } else if mode == TextEscapeState::Slash {
            let real_c = match pop_c {
                '"' => '"',
                '\\' => '\\',
                'a' => '\x07',
                'f' => '\x0C',
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                'v' => '\x0B',
                _ => '\0',
            };
            if real_c == '\0' {
                let s_err = format!("\\{}", next_c);
                return Err(TokenizerError::IllegalEscapeSequence(src.get_pos(), s_err));
            } else {
                val.push(real_c);
                mode = TextEscapeState::Normal;
            }
        } else if mode == TextEscapeState::Unicode {
            fn ret_uni_err(
                src: &PeekReader,
                buf_unicode: &String,
            ) -> Result<Token, TokenizerError> {
                let s_err = "\\u".to_string() + &buf_unicode;
                return Err(TokenizerError::IllegalEscapeSequence(src.get_pos(), s_err));
            }

            if buf_unicode.len() == 0 && pop_c == 'u' {
                // do nothing
            } else if pop_c == ';' {
                // finish
                let hex_val = match u32::from_str_radix(&buf_unicode, 16) {
                    Ok(x) => x,
                    _ => return ret_uni_err(src, &buf_unicode),
                };
                let real_c = match u32_to_char(hex_val) {
                    Some(c) => c,
                    _ => return ret_uni_err(src, &buf_unicode),
                };
                val.push(real_c);
                mode = TextEscapeState::Normal;
            } else if pop_c.is_digit(16) {
                buf_unicode.push(pop_c);
            } else {
                buf_unicode.push(pop_c);
                return ret_uni_err(src, &buf_unicode);
            }
        } else {
            unreachable!();
        }

        raw_val.push(src.pop());
    }

    return Ok(Token::StringValue(Span::new(), raw_val, val));
}

fn parse_numeric_value(src: &mut PeekReader, state: &mut State) -> Result<Token, TokenizerError> {
    let (last_c, pop_c, next_c) = (src.peek(0), src.peek(1), src.peek(2));
    unimplemented!()
}

// the other functions don't actually calculate the span, they just return a dummy value for it
fn parse_next_token(src: &mut PeekReader, state: &mut State) -> Result<Token, TokenizerError> {
    let (last_c, pop_c, next_c) = (src.peek(0), src.peek(1), src.peek(2));
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
#[path = "raw_tokenizer_test.rs"]
mod tests;

// #[cfg(test)]
// mod tests {
//     use crate::raw_tokenizer::*;

//     #[test]
//     fn test_1() {
//         let s = "";
//         let mut parser = RawTokenizer::new(Box::new(s.bytes()));
//         assert_eq!(parser.next(), None);

//         let s = "a";
//         let mut parser = RawTokenizer::new(Box::new(s.bytes()));
//         assert_eq!(
//             parser.next(),
//             Some(RawToken::InlineText(
//                 Span::new2(0, 1, 0, 1, 1, 1),
//                 "a".to_string()
//             ))
//         );
//         assert_eq!(parser.next(), None);

//         let s = "a{";
//         let mut parser = RawTokenizer::new(Box::new(s.bytes()));
//         assert_eq!(
//             parser.next(),
//             Some(RawToken::InlineText(
//                 Span::new2(0, 1, 0, 1, 1, 1),
//                 "a".to_string()
//             ))
//         );

//         let s = "hello world! {";
//         let mut parser = RawTokenizer::new(Box::new(s.bytes()));
//         assert_eq!(
//             parser.next(),
//             Some(RawToken::InlineText(
//                 Span::new2(0, 1, 0, 14, 1, 14),
//                 "hello world! {".to_string()
//             ))
//         );

//         let s = "hello > world!{ ";
//         let mut parser = RawTokenizer::new(Box::new(s.bytes()));
//         assert_eq!(
//             parser.next(),
//             Some(RawToken::InlineText(
//                 Span::new2(0, 1, 0, 14, 1, 14),
//                 "hello > world!".to_string()
//             ))
//         );

//         let s = "hello } world!{!";
//         let mut parser = RawTokenizer::new(Box::new(s.bytes()));
//         assert_eq!(
//             parser.next(),
//             Some(RawToken::InlineText(
//                 Span::new2(0, 1, 0, 14, 1, 14),
//                 "hello } world!".to_string()
//             ))
//         );

//         let s = "\\t } \\{\\s ";
//         let mut parser = RawTokenizer::new(Box::new(s.bytes()));
//         assert_eq!(
//             parser.next(),
//             Some(RawToken::InlineText(
//                 Span::new2(0, 1, 0, 10, 1, 10),
//                 "\\t } \\{\\s ".to_string()
//             ))
//         );
//         assert_eq!(parser.next(), None);
//     }

//     #[test]
//     fn test_2() {
//         let s = "abc {icon}{em; hi! }\n{em ; Hi!\\s} ";
//         let mut parser = RawTokenizer::new(Box::new(s.bytes()));
//         assert_eq!(
//             parser.next(),
//             Some(RawToken::InlineText(
//                 Span::new2(0, 1, 0, 4, 1, 4),
//                 "abc ".to_string()
//             ))
//         );
//         assert_eq!(
//             parser.next(),
//             Some(RawToken::CurlyTagStart(
//                 Span::new2(5, 1, 5, 10, 1, 10),
//                 "{icon".to_string()
//             ))
//         );
//         assert_eq!(
//             parser.next(),
//             Some(RawToken::CurlyTagEnd(Span::new2(10, 1, 10, 11, 1, 11)))
//         );
//         assert_eq!(
//             parser.next(),
//             Some(RawToken::CurlyTagStart(
//                 Span::new2(10, 1, 10, 14, 1, 14),
//                 "{em;".to_string()
//             ))
//         );
//         assert_eq!(
//             parser.next(),
//             Some(RawToken::InlineText(
//                 Span::new2(14, 1, 14, 19, 1, 19),
//                 " hi! ".to_string()
//             ))
//         );
//         assert_eq!(
//             parser.next(),
//             Some(RawToken::CurlyTagEnd(Span::new2(20, 1, 20, 21, 1, 21)))
//         );
//         assert_eq!(
//             parser.next(),
//             Some(RawToken::InlineText(
//                 Span::new2(20, 1, 20, 21, 2, 0),
//                 "\n".to_string()
//             ))
//         );
//         assert_eq!(
//             parser.next(),
//             Some(RawToken::CurlyTagStart(
//                 Span::new2(22, 2, 1, 27, 2, 6),
//                 "{em ;".to_string()
//             ))
//         );
//         assert_eq!(
//             parser.next(),
//             Some(RawToken::InlineText(
//                 Span::new2(26, 2, 5, 32, 2, 11),
//                 " Hi!\\s".to_string()
//             ))
//         );
//         assert_eq!(
//             parser.next(),
//             Some(RawToken::CurlyTagEnd(Span::new2(33, 2, 12, 34, 2, 13)))
//         );
//         assert_eq!(
//             parser.next(),
//             Some(RawToken::InlineText(
//                 Span::new2(33, 2, 12, 34, 2, 13),
//                 " ".to_string()
//             ))
//         );
//         assert_eq!(parser.next(), None);
//     }

//     #[test]
//     fn test_3() {
//         let s = "abc {icon bool_attr id=\"ab\\\"c\" num=1 val=.5; text   }{end id=\"e\"}";
//         let mut parser = RawTokenizer::new(Box::new(s.bytes()));
//         assert_eq!(
//             parser.next(),
//             Some(RawToken::InlineText(
//                 Span::new2(0, 1, 0, 4, 1, 4),
//                 "abc ".to_string()
//             ))
//         );
//         assert_eq!(
//             parser.next(),
//             Some(RawToken::CurlyTagStart(
//                 Span::new2(5, 1, 5, 11, 1, 11),
//                 "{icon ".to_string()
//             ))
//         );
//         assert_eq!(
//             parser.next(),
//             Some(RawToken::AttributeName(
//                 Span::new2(11, 1, 11, 20, 1, 20),
//                 "bool_attr".to_string()
//             ))
//         );
//         assert_eq!(
//             parser.next(),
//             Some(RawToken::AttributeName(
//                 Span::new2(20, 1, 20, 23, 1, 23),
//                 "id=".to_string()
//             ))
//         );
//         assert_eq!(
//             parser.next(),
//             Some(RawToken::StringValue(
//                 Span::new2(23, 1, 23, 30, 1, 30),
//                 "\"ab\\\"c\"".to_string()
//             ))
//         );
//         assert_eq!(
//             parser.next(),
//             Some(RawToken::AttributeName(
//                 Span::new2(30, 1, 30, 35, 1, 35),
//                 " num=".to_string()
//             ))
//         );
//         assert_eq!(
//             parser.next(),
//             Some(RawToken::NumericValue(
//                 Span::new2(35, 1, 35, 36, 1, 36),
//                 "1".to_string()
//             ))
//         );
//         assert_eq!(
//             parser.next(),
//             Some(RawToken::AttributeName(
//                 Span::new2(37, 1, 37, 42, 1, 42),
//                 " val=".to_string()
//             ))
//         );
//         assert_eq!(
//             parser.next(),
//             Some(RawToken::NumericValue(
//                 Span::new2(41, 1, 41, 43, 1, 43),
//                 ".5".to_string()
//             ))
//         );
//         assert_eq!(
//             parser.next(),
//             Some(RawToken::TextMarker(Span::new2(44, 1, 44, 45, 1, 45), ';'))
//         );
//         assert_eq!(
//             parser.next(),
//             Some(RawToken::InlineText(
//                 Span::new2(44, 1, 44, 52, 1, 52),
//                 " text   ".to_string()
//             ))
//         );
//         assert_eq!(
//             parser.next(),
//             Some(RawToken::CurlyTagEnd(Span::new2(53, 1, 53, 54, 1, 54)))
//         );
//         assert_eq!(
//             parser.next(),
//             Some(RawToken::CurlyTagStart(
//                 Span::new2(53, 1, 53, 58, 1, 58),
//                 "{end ".to_string()
//             ))
//         );
//         // println!("{:?}", parser);
//         assert_eq!(
//             parser.next(),
//             Some(RawToken::AttributeName(
//                 Span::new2(59, 1, 59, 62, 1, 62),
//                 "id=".to_string()
//             ))
//         );
//         assert_eq!(
//             parser.next(),
//             Some(RawToken::StringValue(
//                 Span::new2(61, 1, 61, 64, 1, 64),
//                 "\"e\"".to_string()
//             ))
//         );
//         assert_eq!(
//             parser.next(),
//             Some(RawToken::CurlyTagEnd(
//                 Span::new2(64, 1, 64, 65, 1, 65)
//             ))
//         );
//         assert_eq!(parser.next(), None);
//     }
// }

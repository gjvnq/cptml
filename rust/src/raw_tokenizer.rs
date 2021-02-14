//! This module/file contains low level stuff you probably should not use.

use crate::hacks::is_valid_id_char;
use crate::hacks::u32_to_char;

use crate::errors::ParserError;
use crate::peek_reader::PeekReader;

use crate::pos::Span;
use core::fmt::Debug;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BasicName {
    pub view: String,
    pub special: bool,
    pub prefix: String,
    pub local: String,
}

impl BasicName {
    pub fn new(view: &str, special: bool, prefix: &str, local: &str) -> BasicName {
        BasicName {
            view: view.to_string(),
            special: special,
            prefix: prefix.to_string(),
            local: local.to_string(),
        }
    }
    pub fn new_empty() -> BasicName {
        BasicName {
            view: "".to_string(),
            special: false,
            prefix: "".to_string(),
            local: "".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttrValue {
    String(String),
    Number(i64, u8),
}

// Self Note: when making a tree structure, the original "form" will always be printed if possible (use dirty bit?)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RawToken {
    // (span, raw, parsed)
    CodeBlock(Span, String, String),
    Whitespace(Span, String, String),
    TextMarker(Span, char),
    InlineText(Span, String, String),
    InlineMathText(Span, String, String),
    DisplayMathText(Span, String, String),
    AttributeName(Span, String, BasicName),
    AttributeValue(Span, String, AttrValue),
    CurlyTagStart(Span, String, BasicName),
    CurlyTagEnd(Span, char),
    PointyTagHead(Span, String, BasicName),
    PointyTagTail(Span, char),
}

impl RawToken {
    pub fn get_raw(&self) -> String {
        match self {
            RawToken::CodeBlock(_, raw, _) => raw.to_string(),
            RawToken::Whitespace(_, raw, _) => raw.to_string(),
            RawToken::TextMarker(_, raw_c) => raw_c.to_string(),
            RawToken::InlineText(_, raw, _) => raw.to_string(),
            RawToken::InlineMathText(_, raw, _) => raw.to_string(),
            RawToken::DisplayMathText(_, raw, _) => raw.to_string(),
            RawToken::AttributeName(_, raw, _) => raw.to_string(),
            RawToken::AttributeValue(_, raw, _) => raw.to_string(),
            RawToken::CurlyTagStart(_, raw, _) => raw.to_string(),
            RawToken::CurlyTagEnd(_, raw_c) => raw_c.to_string(),
            RawToken::PointyTagHead(_, raw, _) => raw.to_string(),
            RawToken::PointyTagTail(_, raw_c) => raw_c.to_string(),
        }
    }

    pub fn get_span(&self) -> Span {
        match self {
            RawToken::CodeBlock(span, _, _) => *span,
            RawToken::Whitespace(span, _, _) => *span,
            RawToken::TextMarker(span, _) => *span,
            RawToken::InlineText(span, _, _) => *span,
            RawToken::InlineMathText(span, _, _) => *span,
            RawToken::DisplayMathText(span, _, _) => *span,
            RawToken::AttributeName(span, _, _) => *span,
            RawToken::AttributeValue(span, _, _) => *span,
            RawToken::CurlyTagStart(span, _, _) => *span,
            RawToken::CurlyTagEnd(span, _) => *span,
            RawToken::PointyTagHead(span, _, _) => *span,
            RawToken::PointyTagTail(span, _) => *span,
        }
    }

    pub fn set_span(&mut self, new_span: Span) {
        if let RawToken::CodeBlock(ref mut span, _, _) = self {
            *span = new_span;
        } else if let RawToken::Whitespace(ref mut span, _, _) = self {
            *span = new_span;
        } else if let RawToken::TextMarker(ref mut span, _) = self {
            *span = new_span;
        } else if let RawToken::InlineText(ref mut span, _, _) = self {
            *span = new_span;
        } else if let RawToken::InlineMathText(ref mut span, _, _) = self {
            *span = new_span;
        } else if let RawToken::DisplayMathText(ref mut span, _, _) = self {
            *span = new_span;
        } else if let RawToken::AttributeName(ref mut span, _, _) = self {
            *span = new_span;
        } else if let RawToken::AttributeValue(ref mut span, _, _) = self {
            *span = new_span;
        } else if let RawToken::CurlyTagStart(ref mut span, _, _) = self {
            *span = new_span;
        } else if let RawToken::CurlyTagEnd(ref mut span, _) = self {
            *span = new_span;
        } else if let RawToken::PointyTagHead(ref mut span, _, _) = self {
            *span = new_span;
        } else if let RawToken::PointyTagTail(ref mut span, _) = self {
            *span = new_span;
        } else {
            unreachable!();
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Default,
    StartOfInput,
    CodeBlock,
    TextMarker,
    WhitespaceAttrName,
    Tag,
    InlineText,
    Math,
    AttributeName,
    NumericValue,
    StringValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct State {
    mode: Mode,
    inside_tag: TagType,
}

impl State {
    fn new() -> State {
        State {
            mode: Mode::StartOfInput,
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

const LIMIT_RETRYS: i32 = 10;

fn next_state(src: &mut PeekReader, state: &mut State) -> Option<ParserError> {
    let (pop_c, next_three) = (src.peek(1), src.peek_string(1, 3));
    let mut i = 0;

    loop {
        if i > LIMIT_RETRYS {
            panic!("next_state didn't stabilize, please file a bug report");
        }
        i += 1;
        state.mode = match state.mode {
            Mode::StartOfInput | Mode::Default => match pop_c {
                '{' | '}' | '<' | '|' => Mode::Tag,
                '$' => Mode::Math,
                _ if next_three == "```" => Mode::CodeBlock,
                _ => Mode::InlineText,
            },
            Mode::TextMarker => Mode::Default,
            Mode::InlineText => Mode::Default,
            Mode::Tag => match pop_c {
                ';' => Mode::TextMarker,
                '{' | '}' | '<' | '|' | '>' => Mode::Tag,
                _ if next_three == "```" => Mode::CodeBlock,
                pop_c if pop_c.is_whitespace() => match state.inside_tag {
                    TagType::NotTag => Mode::Default,
                    _ => Mode::WhitespaceAttrName,
                },
                _ => Mode::Default,
            },
            Mode::WhitespaceAttrName => match pop_c {
                    ';' => Mode::TextMarker,
                    '}' | '|' | '>' => Mode::Tag,
                    _ => Mode::AttributeName
                },
            Mode::AttributeName => match pop_c {
                '0'..='9' => Mode::NumericValue,
                '"' => Mode::StringValue,
                _ => {
                    return Some(ParserError::IllegalChar2(
                        src.get_pos(),
                        pop_c,
                        vec!['"', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9'],
                    ))
                }
            },
            Mode::NumericValue | Mode::StringValue => match pop_c {
                ';' => Mode::TextMarker,
                '}' | '>' | '|' => Mode::Tag,
                pop_c if pop_c.is_whitespace() => Mode::WhitespaceAttrName,
                _ => {
                    return Some(ParserError::IllegalChar2(
                        src.get_pos(),
                        pop_c,
                        vec![';', '}', '|', '>', ' ', '\n', '\t'],
                    ))
                }
            },
            Mode::CodeBlock => Mode::Default,
            Mode::Math => Mode::Default,
        };
        if state.mode != Mode::Default {
            break;
        }
    }

    return None;
}

fn parse_code_block(src: &mut PeekReader, state: &mut State) -> Result<RawToken, ParserError> {
    state.mode = Mode::CodeBlock;
    let mut start_num = 0;
    let mut finished_start = false;
    let mut counting = false;
    let mut counter = 0;
    let mut raw = String::new();
    let mut val = String::new();

    loop {
        let pop_c = src.peek(1);
        if !finished_start {
            if pop_c == '`' {
                start_num += 1;
            } else {
                finished_start = true;
                val.push(pop_c);
            }
        } else if counting {
            if pop_c == '`' {
                counter += 1;
            } else {
                if counter == start_num {
                    break;
                }
                // add "missing" backticks
                for _ in 0..counter {
                    val.push('`');
                }
                counting = false;
                counter = 0;
                val.push(pop_c);
            }
        } else {
            if pop_c == '`' {
                counting = true;
                counter = 1;
            } else {
                val.push(pop_c);
            }
        }
        raw.push(src.pop());
    }

    Ok(RawToken::CodeBlock(Span::new(), raw, val))
}

fn parse_text_marker(src: &mut PeekReader, state: &mut State) -> Result<RawToken, ParserError> {
    state.mode = Mode::TextMarker;
    let pop_c = src.peek(1);

    if pop_c == ';' {
        src.pop();
        return Ok(RawToken::TextMarker(Span::new(), pop_c));
    } else {
        return Err(ParserError::IllegalChar2(src.get_pos(), pop_c, vec![';']));
    }
}

fn parse_math(src: &mut PeekReader, state: &mut State) -> Result<RawToken, ParserError> {
    state.mode = Mode::Math;
    let (pop_c, next_c) = (src.peek(1), src.peek(2));
    let mut raw = String::new();
    let mut val = String::new();

    if pop_c == '$' && next_c == '$' {
        raw.push(src.pop());
        raw.push(src.pop());
    } else if pop_c == '$' {
        raw.push(src.pop());
    } else {
        return Err(ParserError::IllegalChar2(src.get_pos(), pop_c, vec!['$']));
    }
    state.mode = Mode::Math;

    let long_math = next_c == '$';

    loop {
        let (pop_c, next_c) = (src.peek(1), src.peek(2));
        raw.push(src.pop());
        if long_math && pop_c == '$' && next_c == '$' {
            raw.push(src.pop());
            break;
        }
        if !long_math && pop_c == '$' {
            break;
        }
        val.push(pop_c);
    }

    match long_math {
        true => Ok(RawToken::DisplayMathText(Span::new(), raw, val)),
        false => Ok(RawToken::InlineMathText(Span::new(), raw, val)),
    }
}

fn parse_whitespace(src: &mut PeekReader, state: &mut State) -> Result<RawToken, ParserError> {
    state.mode = Mode::WhitespaceAttrName;
    let mut ans = String::new();
    let mut has_break = false;

    loop {
        let pop_c = src.peek(1);
        if !pop_c.is_whitespace() {
            break;
        }
        if pop_c == '\n' {
            has_break = true;
        }
        ans.push(src.pop());
    }

    let val = match has_break {
        false => " ".to_string(),
        true => String::new(),
    };

    return Ok(RawToken::Whitespace(Span::new(), ans, val));
}

fn parse_inline_text(src: &mut PeekReader, state: &mut State) -> Result<RawToken, ParserError> {
    state.mode = Mode::InlineText;
    let mut text_escape = TextEscapeState::Normal;
    let mut ans_raw = String::new();
    let mut ans_parsed = String::new();
    let mut buf_unicode = String::new();
    let mut ws = WhitesapeMode::GotFirst;
    let mut last_vis = 0;

    loop {
        let (last_c, pop_c, next_c) = (src.peek(0), src.peek(1), src.peek(2));
        if pop_c == '\0' {
            break;
        }
        if text_escape == TextEscapeState::Normal {
            // Check if we need to change state
            let pop_special =
                pop_c == '{' || pop_c == '}' || pop_c == '<' || pop_c == '>' || pop_c == '|';
            if pop_special && (last_c != ' ' || next_c != ' ') {
                break;
            }
            ans_raw.push(pop_c);

            // Process escape sequences
            if pop_c == '\\' {
                text_escape = match next_c {
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
                if ans_parsed.len() > 0 {
                    ans_parsed.push(pop_c);
                }
            }
            if ws == WhitesapeMode::GotFirst {
                ans_parsed.push(pop_c);
                if !c_ws {
                    last_vis = ans_parsed.len();
                }
            }

            // Confirm step
            src.pop();
        } else if text_escape == TextEscapeState::Slash {
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
                return Err(ParserError::IllegalEscapeSequence(src.get_pos(), s_err));
            } else {
                ans_parsed.push(real_c);
                src.pop();
                text_escape = TextEscapeState::Normal;

                ws = WhitesapeMode::GotFirst;
                last_vis = ans_parsed.len();
            }
        } else if text_escape == TextEscapeState::Unicode {
            ans_raw.push(pop_c);

            fn ret_uni_err(
                src: &PeekReader,
                buf_unicode: &String,
            ) -> Result<RawToken, ParserError> {
                let s_err = "\\u".to_string() + &buf_unicode;
                return Err(ParserError::IllegalEscapeSequence(src.get_pos(), s_err));
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
                text_escape = TextEscapeState::Normal;

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

    return Ok(RawToken::InlineText(Span::new(), ans_raw, ans_parsed));
}

fn parse_tag(src: &mut PeekReader, state: &mut State) -> Result<RawToken, ParserError> {
    state.mode = Mode::Tag;

    let pop_c = src.peek(1);
    let mut name = BasicName::new_empty();
    let mut first = true;
    let mut raw_name = "".to_string();
    let mut has_view = false;
    let start_pos = src.get_pos();
    let open = pop_c;

    if pop_c == '}' {
        state.inside_tag = TagType::NotTag;
        return Ok(RawToken::CurlyTagEnd(Span::new(), src.pop()));
    }
    if pop_c == '>' {
        state.inside_tag = TagType::NotTag;
        return Ok(RawToken::PointyTagTail(Span::new(), src.pop()));
    }
    if pop_c == '|' && state.inside_tag == TagType::PointyTag {
        state.inside_tag = TagType::NotTag;
        return Ok(RawToken::PointyTagTail(Span::new(), src.pop()));
    }
    let ans_kind = match pop_c {
        '{' => RawToken::CurlyTagStart(Span::new(), String::new(), BasicName::new_empty()),
        '}' => RawToken::CurlyTagEnd(Span::new(), '}'),
        '<' => RawToken::PointyTagHead(Span::new(), String::new(), BasicName::new_empty()),
        '>' => RawToken::PointyTagTail(Span::new(), '>'),
        '|' => match state.inside_tag {
            TagType::PointyTag => RawToken::PointyTagTail(Span::new(), '|'),
            _ => RawToken::PointyTagHead(Span::new(), String::new(), BasicName::new_empty()),
        },
        _ => {
            return Err(ParserError::IllegalChar2(
                src.get_pos(),
                pop_c,
                vec!['<', '>', '{', '}', '|'],
            ))
        }
    };

    let tag_type = match pop_c {
        '<' | '|' => TagType::PointyTag,
        '{' | '}' => TagType::CurlyTag,
        _ => unreachable!(),
    };
    state.inside_tag = tag_type;
    raw_name.push(src.pop());

    loop {
        let pop_c = src.peek(1);
        if pop_c == '\0' || pop_c.is_whitespace() {
            break;
        }
        if tag_type == TagType::CurlyTag && (pop_c == ';' || pop_c == '}') {
            break;
        }
        if tag_type == TagType::PointyTag && (pop_c == '|' || pop_c == '>') {
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
            return Err(ParserError::IllegalCharMsg(
                src.get_pos(),
                pop_c,
                "valid id char".to_string(),
            ));
        }

        first = false;
        raw_name.push(src.pop());
    }

    if has_view {
        return Err(ParserError::MissingTerminator(src.get_pos(), ')'));
    }
    if name.local.len() == 0 && !(tag_type == TagType::PointyTag && open == '|') {
        return Err(ParserError::MissingLocalName(start_pos));
    }

    match ans_kind {
        RawToken::CurlyTagStart(_, _, _) => {
            return Ok(RawToken::CurlyTagStart(Span::new(), raw_name, name))
        }
        RawToken::PointyTagHead(_, _, _) => {
            return Ok(RawToken::PointyTagHead(Span::new(), raw_name, name))
        }
        RawToken::PointyTagTail(_, _) => return Ok(RawToken::PointyTagTail(Span::new(), open)),
        _ => panic!("hi"),
    }
}

fn parse_attr_name(src: &mut PeekReader, state: &mut State) -> Result<RawToken, ParserError> {
    state.mode = Mode::AttributeName;

    let mut name = BasicName::new_empty();
    let mut raw_name = "".to_string();
    let start_pos = src.get_pos();
    let mut first = true;

    loop {
        let pop_c = src.peek(1);
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
            return Err(ParserError::IllegalCharMsg(
                src.get_pos(),
                pop_c,
                "valid id char".to_string(),
            ));
        }

        first = false;
        raw_name.push(src.pop());
    }
    if name.local.len() == 0 {
        return Err(ParserError::MissingLocalName(start_pos));
    }

    return Ok(RawToken::AttributeName(Span::new(), raw_name, name));
}

fn parse_string_value(src: &mut PeekReader, state: &mut State) -> Result<RawToken, ParserError> {
    state.mode = Mode::StringValue;

    let pop_c = src.peek(1);
    let mut raw_val = "".to_string();
    let mut val = "".to_string();
    let mut buf_unicode = "".to_string();
    let mut mode = TextEscapeState::Normal;

    // Check start char
    if pop_c != '"' {
        return Err(ParserError::IllegalChar2(src.get_pos(), pop_c, vec!['"']));
    }
    raw_val.push(src.pop());

    loop {
        let (pop_c, next_c) = (src.peek(1), src.peek(2));
        if pop_c == '\0' {
            return Err(ParserError::IllegalChar2(src.get_pos(), pop_c, vec!['"']));
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
                return Err(ParserError::IllegalEscapeSequence(src.get_pos(), s_err));
            } else {
                val.push(real_c);
                mode = TextEscapeState::Normal;
            }
        } else if mode == TextEscapeState::Unicode {
            fn ret_uni_err(
                src: &PeekReader,
                buf_unicode: &String,
            ) -> Result<RawToken, ParserError> {
                let s_err = "\\u".to_string() + &buf_unicode;
                return Err(ParserError::IllegalEscapeSequence(src.get_pos(), s_err));
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

    return Ok(RawToken::AttributeValue(Span::new(), raw_val, AttrValue::String(val)));
}

fn parse_numeric_value(src: &mut PeekReader, state: &mut State) -> Result<RawToken, ParserError> {
    state.mode = Mode::NumericValue;

    let mut buf = "".to_string();
    let mut raw = "".to_string();
    let mut dot = false;
    let mut span = Span::new_from(src.get_pos());
    let mut places = 0;

    loop {
        let pop_c = src.peek(1);
        if pop_c.is_whitespace() || pop_c == '\0' || pop_c == '}' || pop_c == ';' || pop_c == '>' || pop_c == '|' {
            break;
        }
        if pop_c == '_' {
            // do nothing
        } else if pop_c.is_ascii_digit() {
            buf.push(pop_c);
            if dot {
                places += 1;
            }
        } else if dot == false && (pop_c == '.' || pop_c == ',') {
            dot = true;
        } else {
            return Err(ParserError::IllegalCharMsg(
                src.get_pos(),
                pop_c,
                "ASCII digit".to_string(),
            ));
        }

        raw.push(src.pop());
    }
    span.end = src.get_pos();

    match buf.parse::<i64>() {
        Ok(num) => return Ok(RawToken::AttributeValue(Span::new(), raw, AttrValue::Number(num, places))),
        _ => return Err(ParserError::IllegalNumber(span, raw)),
    };
}

fn parse_next_token(src: &mut PeekReader, state: &mut State) -> Result<RawToken, ParserError> {
    let mut i = 0;
    loop {
        if i >= LIMIT_RETRYS {
            panic!("parse_next_token didn't find a non-empty token, please file a bug report");
        }
        i += 1;

        if src.peek(1) == '\0' {
            return Err(ParserError::EndOfInput);
        }

        let mut span = Span::new();
        span.start = src.get_pos();
        match next_state(src, state) {
            None => {}
            Some(err) => return Err(err),
        };

        let func = match state.mode {
            Mode::StartOfInput => unreachable!(),
            Mode::Default => unreachable!(),
            Mode::InlineText => parse_inline_text,
            Mode::WhitespaceAttrName => parse_whitespace,
            Mode::CodeBlock => parse_code_block,
            Mode::TextMarker => parse_text_marker,
            Mode::Math => parse_math,
            Mode::Tag => parse_tag,
            Mode::AttributeName => parse_attr_name,
            Mode::StringValue => parse_string_value,
            Mode::NumericValue => parse_numeric_value,
        };
        let mut ans = match func(src, state) {
            Ok(v) => v,
            Err(err) => return Err(err),
        };

        // the span calculation is the resposability of this function, not the other ones
        span.end = src.get_pos();
        ans.set_span(span);

        if span.len() > 0 {
            return Ok(ans);
        }
    }
}

#[derive(Debug)]
pub struct RawTokenizer {
    src: Box<PeekReader>,
    state: State,
    reout: bool,
    last_output: Result<RawToken, ParserError>,
}

impl RawTokenizer {
    pub fn new(src: Box<PeekReader>) -> RawTokenizer {
        RawTokenizer {
            src: src,
            state: State::new(),
            reout: false,
            last_output: Err(ParserError::NotReadyYet),
        }
    }
    pub fn next(&mut self) -> Result<RawToken, ParserError> {
        if self.reout {
            self.reout = false;
            return self.last_output.clone();
        }
        self.last_output = parse_next_token(&mut self.src, &mut self.state);
        return self.last_output.clone();
    }
    pub fn unnext(&mut self) {
        self.reout = true;
    }
}

#[cfg(test)]
#[path = "raw_tokenizer_test.rs"]
mod tests;

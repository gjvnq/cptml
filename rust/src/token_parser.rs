use crate::chars::parse_special_char_escape;
use crate::chars::u32_to_char;
use crate::peek_reader::PeekReader;
use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    CurlyOpen,
    CurlyBreak, //;
    CurlyClose,
    PointyOpen,
    PointyBreak, //|
    PointyClose,
    TagName(BasicName),
    AttrName,
    AttrEquals(BasicName),
    AttrValue(AttrValue),
    Whitespace,
    Text(String),
    Comment,
    CodeBlock { lang: String, content: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    span: Span,
    raw: String,
    val: TokenKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenizerState {
    Normal,
    PointyTagHead(char),
    CurlyTagHead,
    CurlyTagEnd,
    CodeBlock,
    Comment,
}

#[derive(Debug)]
pub struct Tokenizer {
    src: Box<PeekReader>,
    state: TokenizerState,
    reout: bool,
    output: Result<Token, AnyError>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TextEscapeState {
    Normal,
    Slash,
    Unicode,
    Stop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WhitesapeMode {
    NewLine,
    GotFirst,
}

fn parse_text_normal(
    src: &mut PeekReader,
    raw: &mut String,
    mode: &mut TextEscapeState,
) -> Result<Option<char>, AnyError> {
    let (last_c, pop_c, next_c) = (src.peek(0)?, src.peek(1)?, src.peek(2)?);
    // Check if we need to change state
    let pop_special = pop_c == '{' || pop_c == '}' || pop_c == '<' || pop_c == '>' || pop_c == '|';
    if pop_special && (last_c != ' ' || next_c != ' ') {
        *mode = TextEscapeState::Stop;
        return Ok(None);
    }
    if pop_c == '/' && next_c == '*' {
        *mode = TextEscapeState::Stop;
        return Ok(None);
    }

    // Process escape sequences
    if pop_c == '\\' {
        *mode = match next_c {
            'u' => TextEscapeState::Unicode,
            _ => TextEscapeState::Slash,
        };
        return Ok(None);
    }
    raw.push(src.pop()?);
    return Ok(Some(pop_c));
}

fn parse_text_slash(
    src: &mut PeekReader,
    raw: &mut String,
    mode: &mut TextEscapeState,
) -> Result<Option<char>, AnyError> {
    let (last_c, pop_c) = (src.peek(0)?, src.peek(1)?);
    // Make conversions like "\{" -> "{"
    if let Some(real_c) = parse_special_char_escape(last_c, pop_c) {
        *mode = TextEscapeState::Normal;
        raw.push(src.pop()?);
        Ok(Some(real_c))
    } else {
        let s_err = format!("{}{}", last_c, pop_c);
        Err(AnyError::IllegalEscapeSequence(src.get_pos(), s_err))
    }
}

fn parse_text_unicode(
    src: &mut PeekReader,
    raw: &mut String,
    mode: &mut TextEscapeState,
) -> Result<Option<char>, AnyError> {
    let mut buf_unicode = String::new();
    let mut err_msg = "\\".to_string();
    loop {
        let pop_c = src.peek(1)?;
        err_msg.push(pop_c);
        raw.push(src.pop()?);

        if buf_unicode.len() == 0 && pop_c == 'u' {
            // Do nothing
        } else if pop_c == ';' {
            // Decode hex sequence
            *mode = TextEscapeState::Normal;
            let hex_val = match u32::from_str_radix(&buf_unicode, 16) {
                Ok(x) => x,
                _ => return Err(AnyError::IllegalEscapeSequence(src.get_pos(), err_msg)),
            };
            let real_c = match u32_to_char(hex_val) {
                Ok(c) => c,
                _ => return Err(AnyError::IllegalEscapeSequence(src.get_pos(), err_msg)),
            };
            return Ok(Some(real_c));
        } else if pop_c.is_digit(16) {
            // Add regular hex digit to buffer
            buf_unicode.push(pop_c);
        } else {
            // Invalid char
            buf_unicode.push(pop_c);
            return Err(AnyError::IllegalEscapeSequence(src.get_pos(), err_msg));
        }
    }
}

fn parse_text(src: &mut PeekReader, state: &mut TokenizerState) -> Result<Token, AnyError> {
    *state = TokenizerState::Normal;
    let mut mode = TextEscapeState::Normal;
    let mut raw = String::new();
    let mut ans_parsed = String::new();
    let mut ws = WhitesapeMode::GotFirst;
    let mut last_vis = 0;
    let start = src.get_pos();

    loop {
        if src.peek(1)? == '\0' {
            mode = TextEscapeState::Stop;
        }
        let val_c = match mode {
            TextEscapeState::Normal => parse_text_normal(src, &mut raw, &mut mode)?,
            TextEscapeState::Slash => parse_text_slash(src, &mut raw, &mut mode)?,
            TextEscapeState::Unicode => parse_text_unicode(src, &mut raw, &mut mode)?,
            TextEscapeState::Stop => break,
        };
        if let Some(val_c) = val_c {
            // Process whitespace relevance
            let c_ws = val_c == ' ' || val_c == '\t';
            if ws == WhitesapeMode::NewLine && !c_ws {
                ws = WhitesapeMode::GotFirst;
            }
            if val_c == '\n' {
                // At the end of the line, trim the strign to tha last "visible" char
                ans_parsed = ans_parsed[..last_vis].to_string();
                ws = WhitesapeMode::NewLine;
                if ans_parsed.len() > 0 {
                    ans_parsed.push(val_c);
                }
            }
            if ws == WhitesapeMode::GotFirst {
                ans_parsed.push(val_c);
                if !c_ws {
                    last_vis = ans_parsed.len();
                }
            }
        } else {
            // Do nothing
        }
    }

    // TO DO: set next state?
    let c = src.peek(1)?;
    *state = match c {
        '{' => TokenizerState::CurlyTagHead,
        '}' => TokenizerState::CurlyTagEnd,
        '<' | '|' => TokenizerState::PointyTagHead(c),
        '/' => TokenizerState::Comment,
        '`' => TokenizerState::CodeBlock,
        _ => {
            return Err(AnyError::FauxPanic(format!(
                "unhandled state transition: peek = {:?} old state = {:?}",
                c, state
            )))
        }
    };

    let end = src.get_pos();
    let span = Span::new_from_to(start, end);

    return Ok(Token {
        span: span,
        raw: raw,
        val: TokenKind::Text(ans_parsed),
    });
}

fn parse_next_token(
    mut src: &mut PeekReader,
    mut state: &mut TokenizerState,
) -> Result<Token, AnyError> {
    let start = src.get_pos();

    let ans = match state {
        TokenizerState::Normal => parse_text(&mut src, &mut state)?,
        _ => unreachable!(),
    };

    let end = src.get_pos();
    let span = Span::new_from_to(start, end);

    if span.len() != ans.raw.len()
        || ans.raw.len() != ans.span.len()
        || span.len() != ans.span.len()
    {
        Err(AnyError::FauxPanic(format!(
            "lengths don't match span={}, ans.raw={}, ans.span={}",
            span.len(),
            ans.raw.len(),
            ans.span.len()
        )))
    } else {
        Ok(ans)
    }
}

impl Tokenizer {
    pub fn new(src: Box<PeekReader>) -> Tokenizer {
        Tokenizer {
            src: src,
            state: TokenizerState::Normal,
            reout: false,
            output: Err(AnyError::NotReadyYet),
        }
    }
    pub fn next(&mut self) -> Result<Token, AnyError> {
        if self.reout {
            self.reout = false;
            return self.output.clone();
        }
        loop {
            self.output = parse_next_token(&mut self.src, &mut self.state);
            // Stop if we got a non empty token or if we got an error
            if self
                .output
                .as_ref()
                .map(|t| !t.span.is_empty())
                .unwrap_or(true)
            {
                break;
            }
        }
        return self.output.clone();
    }
    pub fn unnext(&mut self) {
        self.reout = true;
    }
}

#[cfg(test)]
#[path = "token_parser_test.rs"]
mod tests;

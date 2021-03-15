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
    pub(crate) span: Span,
    pub(crate) raw: String,
    pub(crate) val: TokenKind,
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

fn parse_next_token(
    mut src: &mut PeekReader,
    mut state: &mut TokenizerState,
) -> Result<Token, AnyError> {
    let start = src.get_pos();

    let ans = match state {
        TokenizerState::Normal => crate::token_parser_text::parse_text(&mut src, &mut state)?,
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

// #[cfg(test)]
// #[path = "token_parser_test.rs"]
// mod tests;

use crate::chars::parse_special_char_escape;
use crate::chars::u32_to_char;
use crate::prelude::*;
use crate::token_parser::*;

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
    raw.push(src.pop()?);
    if pop_c == '\\' {
        *mode = match next_c {
            'u' => TextEscapeState::Unicode,
            _ => TextEscapeState::Slash,
        };
        return Ok(None);
    }
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

pub(crate) fn parse_text(src: &mut PeekReader) -> Result<Token, AnyError> {
    let mut mode = TextEscapeState::Normal;
    let mut raw = String::new();
    let mut ans_parsed = String::new();
    let mut ws = WhitesapeMode::GotFirst;
    let mut last_vis = 0;
    let start = src.get_pos();

    loop {
        let c1 = src.peek(1)?;
        if c1 == '\0' {
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
            let c1_ws = c1 == ' ' || c1 == '\t';
            if ws == WhitesapeMode::NewLine && !c1_ws {
                ws = WhitesapeMode::GotFirst;
            }
            if c1 == '\n' {
                // At the end of the line, trim the string to tha last "visible" char
                ans_parsed = ans_parsed[..last_vis].to_string();
                ws = WhitesapeMode::NewLine;
                if ans_parsed.len() > 0 {
                    ans_parsed.push(c1);
                }
            }
            if ws == WhitesapeMode::GotFirst {
                ans_parsed.push(val_c);
                if !c1_ws {
                    last_vis = ans_parsed.len();
                }
            }
        } else {
            // Do nothing
        }
    }

    let end = src.get_pos();
    let span = Span::new_from_to(start, end);

    return Ok(Token {
        span: span,
        raw: raw,
        val: TokenKind::Text(ans_parsed),
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_text_slash_1() {
        let input_str = "\\t";
        let mut raw = String::new();
        let mut input = PeekReader::from_str(&input_str).unwrap();
        let mut mode = TextEscapeState::Slash;
        input.pop().unwrap();
        let ans = parse_text_slash(&mut input, &mut raw, &mut mode);
        assert_eq!(input_str[1..], raw);
        assert_eq!(ans, Ok(Some('\t')));
    }

    #[test]
    fn test_parse_text_slash_2() {
        let input_str = "\\{";
        let mut raw = String::new();
        let mut input = PeekReader::from_str(&input_str).unwrap();
        let mut mode = TextEscapeState::Slash;
        input.pop().unwrap();
        let ans = parse_text_slash(&mut input, &mut raw, &mut mode);
        assert_eq!(input_str[1..], raw);
        assert_eq!(ans, Ok(Some('{')));
    }

    #[test]
    fn test_parse_text_slash_3() {
        let input_str = "\\u";
        let mut raw = String::new();
        let mut input = PeekReader::from_str(&input_str).unwrap();
        let mut mode = TextEscapeState::Slash;
        input.pop().unwrap();
        let ans = parse_text_slash(&mut input, &mut raw, &mut mode);
        assert_eq!(
            ans,
            Err(AnyError::IllegalEscapeSequence(
                Position {
                    byte: 1,
                    line: 1,
                    col: 1
                },
                "\\u".to_string()
            ))
        );
    }

    #[test]
    fn test_parse_text_unicode_1() {
        let input_str = "\\u1F4DA;";
        let mut raw = String::new();
        let mut input = PeekReader::from_str(&input_str).unwrap();
        let mut mode = TextEscapeState::Unicode;
        input.pop().unwrap();
        let ans = parse_text_unicode(&mut input, &mut raw, &mut mode);
        assert_eq!(raw, input_str[1..]);
        assert_eq!(ans, Ok(Some('📚')));
    }

    #[test]
    fn test_parse_text_unicode_2() {
        let input_str = "\\u10437;";
        let mut raw = String::new();
        let mut input = PeekReader::from_str(&input_str).unwrap();
        let mut mode = TextEscapeState::Unicode;
        input.pop().unwrap();
        let ans = parse_text_unicode(&mut input, &mut raw, &mut mode);
        assert_eq!(raw, input_str[1..]);
        assert_eq!(ans, Ok(Some('𐐷')));
    }

    #[test]
    fn test_parse_text_unicode_3() {
        let input_str = "\\uD801;\\uDC37;";
        let mut raw = String::new();
        let mut input = PeekReader::from_str(&input_str).unwrap();
        let mut mode = TextEscapeState::Unicode;
        input.pop().unwrap();
        let ans = parse_text_unicode(&mut input, &mut raw, &mut mode);
        assert_eq!(
            ans,
            Err(AnyError::IllegalEscapeSequence(
                Position {
                    byte: 7,
                    line: 1,
                    col: 7
                },
                "\\uD801;".to_string()
            ))
        );
        assert_eq!(raw, input_str[1..7]);
    }

    #[test]
    fn test_parse_text_1() {
        let input_str = " hi { < | > } \n\t b/d*a {icon";
        let mut input = PeekReader::from_str(&input_str).unwrap();
        let ans = parse_text(&mut input).unwrap();
        assert_eq!(ans.raw, input_str[..23]);
        assert_eq!(ans.span, Span::new2(0, 1, 0, 23, 2, 8));
        assert_eq!(
            ans.val,
            TokenKind::Text(" hi { < | > }\nb/d*a ".to_string())
        );
    }

    #[test]
    fn test_parse_text_2() {
        let input_str = "abc/*comment";
        let mut input = PeekReader::from_str(&input_str).unwrap();
        let ans = parse_text(&mut input).unwrap();
        assert_eq!(ans.raw, input_str[..3]);
        assert_eq!(ans.span, Span::new2(0, 1, 0, 3, 1, 3));
        assert_eq!(ans.val, TokenKind::Text("abc".to_string()));
    }

    #[test]
    fn test_parse_text_3() {
        let input_str = "\n     \\s dasds \\t\t\t\n ";
        let mut input = PeekReader::from_str(&input_str).unwrap();
        let ans = parse_text(&mut input).unwrap();
        assert_eq!(ans.raw, input_str);
        assert_eq!(ans.span, Span::new2(0, 1, 0, 21, 3, 1));
        assert_eq!(ans.val, TokenKind::Text("  dasds \t\n".to_string()));
    }

    #[test]
    fn test_parse_text_4() {
        let input_str = "\n     \\s \n \\s dasds \\t\t\n";
        let mut input = PeekReader::from_str(&input_str).unwrap();
        let ans = parse_text(&mut input).unwrap();
        assert_eq!(ans.raw, input_str);
        assert_eq!(ans.span, Span::new2(0, 1, 0, 24, 4, 0));
        assert_eq!(ans.val, TokenKind::Text(" \n  dasds \t\n".to_string()));
    }
}

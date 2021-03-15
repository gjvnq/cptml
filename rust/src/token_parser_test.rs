// use crate::prelude::*;
use crate::token_parser::*;

fn quick_input(input: &'static str) -> PeekReader {
    PeekReader::new(Box::new(input.bytes())).unwrap()
}

#[test]
fn test_parse_text_slash_1() {
    let input_str = "\\t";
    let mut raw = String::new();
    let mut input = quick_input(&input_str);
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
    let mut input = quick_input(&input_str);
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
    let mut input = quick_input(&input_str);
    let mut mode = TextEscapeState::Slash;
    input.pop().unwrap();
    let ans = parse_text_slash(&mut input, &mut raw, &mut mode);
    assert_eq!(ans, Err(AnyError::IllegalEscapeSequence(Position { byte: 1, line: 1, col: 1 }, "\\u".to_string())));
}

#[test]
fn test_parse_text_unicode_1() {
    let input_str = "\\u1F4DA;";
    let mut raw = String::new();
    let mut input = quick_input(&input_str);
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
    let mut input = quick_input(&input_str);
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
    let mut input = quick_input(&input_str);
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
fn test_parse_text_normal_1() {
    let input_str = " hi { < | > } \n\t b/d*a {icon";
    let mut input = quick_input(&input_str);
    let mut state = TokenizerState::Normal;
    let ans = parse_text(&mut input, &mut state).unwrap();
    assert_eq!(state, TokenizerState::CurlyTagHead);
    assert_eq!(ans.raw, input_str[..23]);
    assert_eq!(ans.span, Span::new2(0, 1, 0, 23, 2, 8));
    assert_eq!(
        ans.val,
        TokenKind::Text(" hi { < | > }\nb/d*a ".to_string())
    );
}

#[test]
fn test_parse_text_normal_2() {
    let input_str = "abc/*comment";
    let mut input = quick_input(&input_str);
    let mut state = TokenizerState::Normal;
    let ans = parse_text(&mut input, &mut state).unwrap();
    assert_eq!(state, TokenizerState::Comment);
    assert_eq!(ans.raw, input_str[..3]);
    assert_eq!(ans.span, Span::new2(0, 1, 0, 3, 1, 3));
    assert_eq!(
        ans.val,
        TokenKind::Text("abc".to_string())
    );
}
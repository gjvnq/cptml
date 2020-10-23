use crate::raw_tokenizer::*;

fn quick_input(input: &'static str) -> PeekReader {
    PeekReader::new(Box::new(input.bytes()))
}

#[test]
fn parse_inline_text_1() {
    let mut input = quick_input("");
    let mut state = State {
        mode: Mode::StartOfInput,
        after_whitespace: None,
        text_escape: TextEscapeState::Normal,
        inside_tag: TagType::NotTag,
    };
    let ans = parse_inline_text(&mut input, &mut state);
    assert_eq!(
        ans,
        Ok(Token::InlineText(
            Span::new(),
            "".to_string(),
            "".to_string()
        ))
    );
}

#[test]
fn parse_inline_text_2() {
    let mut input = quick_input(" a bc  ");
    let mut state = State {
        mode: Mode::StartOfInput,
        after_whitespace: None,
        text_escape: TextEscapeState::Normal,
        inside_tag: TagType::NotTag,
    };
    let ans = parse_inline_text(&mut input, &mut state);
    assert_eq!(
        ans,
        Ok(Token::InlineText(
            Span::new(),
            " a bc  ".to_string(),
            "a bc".to_string()
        ))
    );
}

#[test]
fn parse_inline_text_3() {
    let mut input = quick_input(" \ta bc  \n\t z ");
    let mut state = State {
        mode: Mode::StartOfInput,
        after_whitespace: None,
        text_escape: TextEscapeState::Normal,
        inside_tag: TagType::NotTag,
    };
    let ans = parse_inline_text(&mut input, &mut state);
    assert_eq!(
        ans,
        Ok(Token::InlineText(
            Span::new(),
            " \ta bc  \n\t z ".to_string(),
            "a bc\nz".to_string()
        ))
    );
}

#[test]
fn parse_inline_text_4() {
    let mut input = quick_input(" \ta bc  \\n\n\\s\t z\\u1F4DA;   \\u0A;");
    let mut state = State {
        mode: Mode::StartOfInput,
        after_whitespace: None,
        text_escape: TextEscapeState::Normal,
        inside_tag: TagType::NotTag,
    };
    let ans = parse_inline_text(&mut input, &mut state);
    assert_eq!(
        ans,
        Ok(Token::InlineText(
            Span::new(),
            " \ta bc  \\n\n\\s\t z\\u1F4DA;   \\u0A;".to_string(),
            "a bc  \n\n \t z📚   \n".to_string()
        ))
    );
}

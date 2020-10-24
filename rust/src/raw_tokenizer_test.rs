use crate::raw_tokenizer::*;

fn quick_input(input: &'static str) -> PeekReader {
    PeekReader::new(Box::new(input.bytes()))
}

#[test]
fn parse_inline_text_1a() {
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
fn parse_inline_text_1b() {
    let mut input = quick_input("     \n");
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
            "     \n".to_string(),
            "\n".to_string()
        ))
    );
}

#[test]
fn parse_inline_text_2a() {
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
            " a bc  ".to_string()
        ))
    );
}

#[test]
fn parse_inline_text_2b() {
    let mut input = quick_input("\n a bc  \n");
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
            "\n a bc  \n".to_string(),
            "\na bc\n".to_string()
        ))
    );
}

#[test]
fn parse_inline_text_3() {
    let mut input = quick_input("\n \ta bc  \n\t z ");
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
            "\n \ta bc  \n\t z ".to_string(),
            "\na bc\nz ".to_string()
        ))
    );
}

#[test]
fn parse_inline_text_4() {
    let mut input = quick_input("\n   \\t   \\n\\u1F4DA;\\t");
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
            "\n   \\t   \\n\\u1F4DA;\\t".to_string(),
            "\n\t   \n📚\t".to_string()
        ))
    );
}

#[test]
fn parse_inline_text_5() {
    let mut input = quick_input("\n a { < | } > bc  \n");
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
            "\n a { < | } > bc  \n".to_string(),
            "\na { < | } > bc\n".to_string()
        ))
    );
}
#[test]
fn parse_inline_text_6() {
    let mut input = quick_input("\n a {bc  \n");
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
            "\n a ".to_string(),
            "\na ".to_string()
        ))
    );
}

use crate::raw_tokenizer::*;

fn quick_input(input: &'static str) -> PeekReader {
    PeekReader::new(Box::new(input.bytes()))
}

#[test]
fn parse_inline_text_1a() {
    let mut input = quick_input("");
    let mut state = State::new();
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
    let mut state = State::new();
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
    let mut state = State::new();
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
    let mut state = State::new();
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
    let mut state = State::new();
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
    let mut state = State::new();
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
    let mut state = State::new();
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
    let mut state = State::new();
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

#[test]
fn parse_tag_start_1() {
    let mut input = quick_input("{icon");
    let mut state = State::new();
    let ans = parse_tag_start(&mut input, &mut state);
    assert_eq!(
        ans,
        Ok(Token::CurlyTagStart(
            Span::new(),
            "{icon".to_string(),
            BasicName {
                view: "".to_string(),
                special: false,
                prefix: "".to_string(),
                local: "icon".to_string()
            }
        ))
    );
}

#[test]
fn parse_tag_start_2() {
    let mut input = quick_input("{icon}");
    let mut state = State::new();
    let ans = parse_tag_start(&mut input, &mut state);
    assert_eq!(
        ans,
        Ok(Token::CurlyTagStart(
            Span::new(),
            "{icon".to_string(),
            BasicName {
                view: "".to_string(),
                special: false,
                prefix: "".to_string(),
                local: "icon".to_string()
            }
        ))
    );
}

#[test]
fn parse_tag_start_3() {
    let mut input = quick_input("{!icon;");
    let mut state = State::new();
    let ans = parse_tag_start(&mut input, &mut state);
    assert_eq!(
        ans,
        Ok(Token::CurlyTagStart(
            Span::new(),
            "{!icon".to_string(),
            BasicName {
                view: "".to_string(),
                special: true,
                prefix: "".to_string(),
                local: "!icon".to_string()
            }
        ))
    );
}

#[test]
fn parse_tag_start_4() {
    let mut input = quick_input("<ns:icon|");
    let mut state = State::new();
    let ans = parse_tag_start(&mut input, &mut state);
    assert_eq!(
        ans,
        Ok(Token::PointyTagStart(
            Span::new(),
            "<ns:icon".to_string(),
            BasicName {
                view: "".to_string(),
                special: false,
                prefix: "ns".to_string(),
                local: "icon".to_string()
            }
        ))
    );
}

#[test]
fn parse_tag_start_5() {
    let mut input = quick_input("<!ns:icon|");
    let mut state = State::new();
    let ans = parse_tag_start(&mut input, &mut state);
    assert_eq!(
        ans,
        Ok(Token::PointyTagStart(
            Span::new(),
            "<!ns:icon".to_string(),
            BasicName {
                view: "".to_string(),
                special: true,
                prefix: "!ns".to_string(),
                local: "icon".to_string()
            }
        ))
    );
}

#[test]
fn parse_tag_start_6() {
    let mut input = quick_input("<(t)tei:line|");
    let mut state = State::new();
    let ans = parse_tag_start(&mut input, &mut state);
    assert_eq!(
        ans,
        Ok(Token::PointyTagStart(
            Span::new(),
            "<(t)tei:line".to_string(),
            BasicName {
                view: "t".to_string(),
                special: false,
                prefix: "tei".to_string(),
                local: "line".to_string()
            }
        ))
    );
}

#[test]
fn parse_tag_start_7() {
    let mut input = quick_input("<(t)tei:line:a|");
    let mut state = State::new();
    let ans = parse_tag_start(&mut input, &mut state);
    assert_eq!(
        ans,
        Err(TokenizerError::IllegalCharMsg(
            Position::new2(12, 1, 12),
            ':',
            "alphanumeric".to_string()
        ))
    );
}

#[test]
fn parse_tag_start_8() {
    let mut input = quick_input("<(ttei:line|");
    let mut state = State::new();
    let ans = parse_tag_start(&mut input, &mut state);
    assert_eq!(
        ans,
        Err(TokenizerError::MissingTerminator(
            Position::new2(11, 1, 11),
            ')'
        ))
    );
}

#[test]
fn parse_tag_start_9() {
    let mut input = quick_input("<ns:>");
    let mut state = State::new();
    let ans = parse_tag_start(&mut input, &mut state);
    assert_eq!(
        ans,
        Err(TokenizerError::MissingLocalName(Position::new2(0, 1, 0),))
    );
}

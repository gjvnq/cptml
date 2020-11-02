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
            "".to_string()
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
            "a bc\n".to_string()
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
            "a bc\nz ".to_string()
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
            "\t   \n📚\t".to_string()
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
            "a { < | } > bc\n".to_string()
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
            "a ".to_string()
        ))
    );
}

#[test]
fn parse_tag_1() {
    let mut input = quick_input("{icon");
    let mut state = State::new();
    let ans = parse_tag(&mut input, &mut state);
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
fn parse_tag_2() {
    let mut input = quick_input("{icon}");
    let mut state = State::new();
    let ans = parse_tag(&mut input, &mut state);
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
fn parse_tag_3() {
    let mut input = quick_input("{!icon;");
    let mut state = State::new();
    let ans = parse_tag(&mut input, &mut state);
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
fn parse_tag_4() {
    let mut input = quick_input("<ns:icon|");
    let mut state = State::new();
    let ans = parse_tag(&mut input, &mut state);
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
    let ans = parse_tag(&mut input, &mut state);
    assert_eq!(
        ans,
        Ok(Token::TextMarker(
            Span::new(),
            '|'
        ))
    );
}

#[test]
fn parse_tag_5() {
    let mut input = quick_input("<!ns:icon|");
    let mut state = State::new();
    let ans = parse_tag(&mut input, &mut state);
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
fn parse_tag_6() {
    let mut input = quick_input("<(t)tei:line|");
    let mut state = State::new();
    let ans = parse_tag(&mut input, &mut state);
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
fn parse_tag_7() {
    let mut input = quick_input("<(t)tei:line:a|");
    let mut state = State::new();
    let ans = parse_tag(&mut input, &mut state);
    assert_eq!(
        ans,
        Err(TokenizerError::IllegalCharMsg(
            Position::new2(12, 1, 12),
            ':',
            "valid id char".to_string()
        ))
    );
}

#[test]
fn parse_tag_8() {
    let mut input = quick_input("<(ttei:line|");
    let mut state = State::new();
    let ans = parse_tag(&mut input, &mut state);
    assert_eq!(
        ans,
        Err(TokenizerError::MissingTerminator(
            Position::new2(11, 1, 11),
            ')'
        ))
    );
}

#[test]
fn parse_tag_9() {
    let mut input = quick_input("<ns:>");
    let mut state = State::new();
    let ans = parse_tag(&mut input, &mut state);
    assert_eq!(
        ans,
        Err(TokenizerError::MissingLocalName(Position::new2(0, 1, 0),))
    );
}

#[test]
fn parse_tag_10() {
    let mut input = quick_input("|(t)tei:line>");
    let mut state = State::new();
    let ans = parse_tag(&mut input, &mut state);
    assert_eq!(
        ans,
        Ok(Token::PointyTagStart(
            Span::new(),
            "|(t)tei:line>".to_string(),
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
fn parse_tag_11() {
    let mut input = quick_input("|(t)>");
    let mut state = State::new();
    let ans = parse_tag(&mut input, &mut state);
    assert_eq!(
        ans,
        Ok(Token::PointyTagStart(
            Span::new(),
            "|(t)>".to_string(),
            BasicName {
                view: "t".to_string(),
                special: false,
                prefix: "".to_string(),
                local: "".to_string()
            }
        ))
    );
}

#[test]
fn parse_tag_12() {
    let mut input = quick_input("|>");
    let mut state = State::new();
    let ans = parse_tag(&mut input, &mut state);
    assert_eq!(
        ans,
        Ok(Token::PointyTagStart(
            Span::new(),
            "|>".to_string(),
            BasicName {
                view: "".to_string(),
                special: false,
                prefix: "".to_string(),
                local: "".to_string()
            }
        ))
    );
}

#[test]
fn parse_tag_13() {
    let mut input = quick_input("}");
    let mut state = State::new();
    let ans = parse_tag(&mut input, &mut state);
    assert_eq!(ans, Ok(Token::CurlyTagEnd(Span::new(), '}')));
}

#[test]
fn parse_attr_name_1() {
    let mut input = quick_input("attr=");
    let mut state = State::new();
    let ans = parse_attr_name(&mut input, &mut state);
    assert_eq!(
        ans,
        Ok(Token::AttributeName(
            Span::new(),
            "attr=".to_string(),
            BasicName {
                view: "".to_string(),
                special: false,
                prefix: "".to_string(),
                local: "attr".to_string()
            }
        ))
    );
}

#[test]
fn parse_attr_name_2() {
    let mut input = quick_input("!id=");
    let mut state = State::new();
    let ans = parse_attr_name(&mut input, &mut state);
    assert_eq!(
        ans,
        Ok(Token::AttributeName(
            Span::new(),
            "!id=".to_string(),
            BasicName {
                view: "".to_string(),
                special: true,
                prefix: "".to_string(),
                local: "!id".to_string()
            }
        ))
    );
}

#[test]
fn parse_attr_name_3() {
    let mut input = quick_input("ns1:attr_val=");
    let mut state = State::new();
    let ans = parse_attr_name(&mut input, &mut state);
    assert_eq!(
        ans,
        Ok(Token::AttributeName(
            Span::new(),
            "ns1:attr_val=".to_string(),
            BasicName {
                view: "".to_string(),
                special: false,
                prefix: "ns1".to_string(),
                local: "attr_val".to_string()
            }
        ))
    );
}

#[test]
fn parse_attr_name_4() {
    let mut input = quick_input("ns1:1attr_val=");
    let mut state = State::new();
    let ans = parse_attr_name(&mut input, &mut state);
    assert_eq!(
        ans,
        Err(TokenizerError::IllegalCharMsg(
            Position::new2(4, 1, 4),
            '1',
            "valid id char".to_string()
        ))
    );
}

#[test]
fn parse_string_value_1() {
    let mut input = quick_input("\"\"");
    let mut state = State::new();
    let ans = parse_string_value(&mut input, &mut state);
    assert_eq!(
        ans,
        Ok(Token::StringValue(
            Span::new(),
            "\"\"".to_string(),
            "".to_string()
        ))
    );
}

#[test]
fn parse_string_value_2() {
    let mut input = quick_input("\"abc\"");
    let mut state = State::new();
    let ans = parse_string_value(&mut input, &mut state);
    assert_eq!(
        ans,
        Ok(Token::StringValue(
            Span::new(),
            "\"abc\"".to_string(),
            "abc".to_string()
        ))
    );
}

#[test]
fn parse_string_value_3() {
    let mut input = quick_input("\"\\\"\"");
    let mut state = State::new();
    let ans = parse_string_value(&mut input, &mut state);
    assert_eq!(
        ans,
        Ok(Token::StringValue(
            Span::new(),
            "\"\\\"\"".to_string(),
            "\"".to_string()
        ))
    );
}

#[test]
fn parse_string_value_4() {
    let mut input = quick_input("\"\\u222B;\"");
    let mut state = State::new();
    let ans = parse_string_value(&mut input, &mut state);
    assert_eq!(
        ans,
        Ok(Token::StringValue(
            Span::new(),
            "\"\\u222B;\"".to_string(),
            "∫".to_string()
        ))
    );
}

#[test]
fn parse_numeric_value_1() {
    let mut input = quick_input("1_000");
    let mut state = State::new();
    let ans = parse_numeric_value(&mut input, &mut state);
    assert_eq!(
        ans,
        Ok(Token::NumericValue(
            Span::new(),
            "1_000".to_string(),
            Number::Integer(1000)
        ))
    );
}

#[test]
fn parse_numeric_value_2() {
    let mut input = quick_input("1_000.3__4");
    let mut state = State::new();
    let ans = parse_numeric_value(&mut input, &mut state);
    assert_eq!(
        ans,
        Ok(Token::NumericValue(
            Span::new(),
            "1_000.3__4".to_string(),
            Number::Float(1000.34)
        ))
    );
}

#[test]
fn parse_numeric_value_3() {
    let mut input = quick_input("_");
    let mut state = State::new();
    let ans = parse_numeric_value(&mut input, &mut state);
    assert_eq!(
        ans,
        Err(TokenizerError::IllegalNumber(
            Span::new2(0, 1, 0, 1, 1, 1),
            "_".to_string()
        ))
    );
}

#[test]
fn parse_whitespace_1() {
    let mut input = quick_input("   ");
    let mut state = State::new();
    let ans = parse_whitespace(&mut input, &mut state);
    assert_eq!(
        ans,
        Ok(Token::Whitespace(
            Span::new(),
            "   ".to_string(),
            " ".to_string()
        ))
    );
}

#[test]
fn parse_whitespace_2() {
    let mut input = quick_input(" \n  ");
    let mut state = State::new();
    let ans = parse_whitespace(&mut input, &mut state);
    assert_eq!(
        ans,
        Ok(Token::Whitespace(
            Span::new(),
            " \n  ".to_string(),
            "".to_string()
        ))
    );
}

#[test]
fn parse_next_token_1() {
    let mut input = quick_input("{emph;\nhi }");
    let mut state = State::new();
    let ans = parse_next_token(&mut input, &mut state, 5);
    assert_eq!(
        ans,
        Ok(Token::CurlyTagStart(
            Span::new2(0, 1, 0, 5, 1, 5),
            "{emph".to_string(),
            BasicName {
                view: "".to_string(),
                special: false,
                prefix: "".to_string(),
                local: "emph".to_string()
            }
        ))
    );
    let ans = parse_next_token(&mut input, &mut state, 5);
    assert_eq!(
        ans,
        Ok(Token::TextMarker(Span::new2(5, 1, 5, 6, 1, 6), ';'))
    );
    let ans = parse_next_token(&mut input, &mut state, 5);
    assert_eq!(
        ans,
        Ok(Token::InlineText(
            Span::new2(6, 1, 6, 10, 2, 3),
            "\nhi ".to_string(),
            "hi ".to_string()
        ))
    );
    let ans = parse_next_token(&mut input, &mut state, 5);
    assert_eq!(
        ans,
        Ok(Token::CurlyTagEnd(Span::new2(10, 2, 3, 11, 2, 4), '}'))
    );
    let ans = parse_next_token(&mut input, &mut state, 5);
    assert_eq!(ans, Err(TokenizerError::EndOfInput));
}

#[test]
fn parse_next_token_2() {
    let mut input = quick_input("{emph \t\n!id=\"elem\" num=3._14;\nhi }");
    let mut state = State::new();
    let ans = parse_next_token(&mut input, &mut state, 5);
    assert_eq!(
        ans,
        Ok(Token::CurlyTagStart(
            Span::new2(0, 1, 0, 5, 1, 5),
            "{emph".to_string(),
            BasicName {
                view: "".to_string(),
                special: false,
                prefix: "".to_string(),
                local: "emph".to_string()
            }
        ))
    );
    let ans = parse_next_token(&mut input, &mut state, 5);
    assert_eq!(
        ans,
        Ok(Token::Whitespace(
            Span::new2(5, 1, 5, 8, 2, 0),
            " \t\n".to_string(),
            "".to_string()
        ))
    );
    let ans = parse_next_token(&mut input, &mut state, 5);
    assert_eq!(
        ans,
        Ok(Token::AttributeName(
            Span::new2(8, 2, 0, 12, 2, 4),
            "!id=".to_string(),
            BasicName {
                view: "".to_string(),
                special: true,
                prefix: "".to_string(),
                local: "!id".to_string()
            }
        ))
    );
    let ans = parse_next_token(&mut input, &mut state, 5);
    assert_eq!(
        ans,
        Ok(Token::StringValue(
            Span::new2(12, 2, 4, 18, 2, 10),
            "\"elem\"".to_string(),
            "elem".to_string(),
        ))
    );
    let ans = parse_next_token(&mut input, &mut state, 5);
    assert_eq!(
        ans,
        Ok(Token::Whitespace(
            Span::new2(18, 2, 10, 19, 2, 11),
            " ".to_string(),
            " ".to_string()
        ))
    );
    let ans = parse_next_token(&mut input, &mut state, 5);
    assert_eq!(
        ans,
        Ok(Token::AttributeName(
            Span::new2(19, 2, 11, 23, 2, 15),
            "num=".to_string(),
            BasicName {
                view: "".to_string(),
                special: false,
                prefix: "".to_string(),
                local: "num".to_string()
            }
        ))
    );
    let ans = parse_next_token(&mut input, &mut state, 5);
    assert_eq!(
        ans,
        Ok(Token::NumericValue(
            Span::new2(23, 2, 15, 28, 2, 20),
            "3._14".to_string(),
            Number::Float(3.14)
        ))
    );

    let ans = parse_next_token(&mut input, &mut state, 5);
    assert_eq!(
        ans,
        Ok(Token::TextMarker(Span::new2(28, 2, 20, 29, 2, 21), ';'))
    );
    let ans = parse_next_token(&mut input, &mut state, 5);
    assert_eq!(
        ans,
        Ok(Token::InlineText(
            Span::new2(29, 2, 21, 33, 3, 3),
            "\nhi ".to_string(),
            "hi ".to_string()
        ))
    );
    let ans = parse_next_token(&mut input, &mut state, 5);
    assert_eq!(
        ans,
        Ok(Token::CurlyTagEnd(Span::new2(33, 3, 3, 34, 3, 4), '}'))
    );
    let ans = parse_next_token(&mut input, &mut state, 5);
    assert_eq!(ans, Err(TokenizerError::EndOfInput));
}

#[test]
fn parse_next_token_3() {
    let mut input = quick_input("<emph \t\n!id=\"elem\" num=3._14|\nhi");
    let mut state = State::new();
    let ans = parse_next_token(&mut input, &mut state, 5);
    assert_eq!(
        ans,
        Ok(Token::PointyTagStart(
            Span::new2(0, 1, 0, 5, 1, 5),
            "<emph".to_string(),
            BasicName {
                view: "".to_string(),
                special: false,
                prefix: "".to_string(),
                local: "emph".to_string()
            }
        ))
    );
    let ans = parse_next_token(&mut input, &mut state, 5);
    assert_eq!(
        ans,
        Ok(Token::Whitespace(
            Span::new2(5, 1, 5, 8, 2, 0),
            " \t\n".to_string(),
            "".to_string()
        ))
    );
    let ans = parse_next_token(&mut input, &mut state, 5);
    assert_eq!(
        ans,
        Ok(Token::AttributeName(
            Span::new2(8, 2, 0, 12, 2, 4),
            "!id=".to_string(),
            BasicName {
                view: "".to_string(),
                special: true,
                prefix: "".to_string(),
                local: "!id".to_string()
            }
        ))
    );
    let ans = parse_next_token(&mut input, &mut state, 5);
    assert_eq!(
        ans,
        Ok(Token::StringValue(
            Span::new2(12, 2, 4, 18, 2, 10),
            "\"elem\"".to_string(),
            "elem".to_string(),
        ))
    );
    let ans = parse_next_token(&mut input, &mut state, 5);
    assert_eq!(
        ans,
        Ok(Token::Whitespace(
            Span::new2(18, 2, 10, 19, 2, 11),
            " ".to_string(),
            " ".to_string()
        ))
    );
    let ans = parse_next_token(&mut input, &mut state, 5);
    assert_eq!(
        ans,
        Ok(Token::AttributeName(
            Span::new2(19, 2, 11, 23, 2, 15),
            "num=".to_string(),
            BasicName {
                view: "".to_string(),
                special: false,
                prefix: "".to_string(),
                local: "num".to_string()
            }
        ))
    );
    let ans = parse_next_token(&mut input, &mut state, 5);
    assert_eq!(
        ans,
        Ok(Token::NumericValue(
            Span::new2(23, 2, 15, 28, 2, 20),
            "3._14".to_string(),
            Number::Float(3.14)
        ))
    );

    let ans = parse_next_token(&mut input, &mut state, 5);
    assert_eq!(
        ans,
        Ok(Token::TextMarker(Span::new2(28, 2, 20, 29, 2, 21), '|'))
    );
    let ans = parse_next_token(&mut input, &mut state, 5);
    assert_eq!(
        ans,
        Ok(Token::InlineText(
            Span::new2(29, 2, 21, 33, 3, 3),
            "\nhi ".to_string(),
            "hi ".to_string()
        ))
    );
    let ans = parse_next_token(&mut input, &mut state, 5);
    assert_eq!(
        ans,
        Ok(Token::CurlyTagEnd(Span::new2(33, 3, 3, 34, 3, 4), '}'))
    );
    let ans = parse_next_token(&mut input, &mut state, 5);
    assert_eq!(ans, Err(TokenizerError::EndOfInput));
}

// #[test]
// fn parse_next_token_3() {
//     let mut input = quick_input("{poem;
//   <(t)line|<(g)sentence|I, by attorney, bless thee from thy mother,|(t)line>
//   <(t)line|Who prays continually for Richmond's good.|(g)sentence>|(t)line>
//   <(t)line|<(g)sentence|So much for that.|(g)><(g)sentence|—The silent hours steal on,|(t)>
//   <(t)line|And flaky darkness breaks within the east.|(g)>|(t)>
// }
// ");
//     let mut state = State::new();
//     let mut ans = parse_next_token(&mut input, &mut state, 5).unwrap();
//     ans.set_span(Span::new());
//     assert_eq!(
//         ans,
//         Token::CurlyTagStart(
//             Span::new2(0, 1, 0, 0, 1, 0),
//             "{poem".to_string(),
//             BasicName {
//                 view: "".to_string(),
//                 special: false,
//                 prefix: "".to_string(),
//                 local: "poem".to_string()
//             }
//         )
//     );
//     let mut ans = parse_next_token(&mut input, &mut state, 5).unwrap();
//     ans.set_span(Span::new());
//     assert_eq!(
//         ans,
//         Token::TextMarker(
//             Span::new2(0, 1, 0, 0, 1, 0),
//             ';'
//         )
//     );
//     let mut ans = parse_next_token(&mut input, &mut state, 5).unwrap();
//     ans.set_span(Span::new());
//     assert_eq!(
//         ans,
//         Token::InlineText(
//             Span::new2(0, 1, 0, 0, 1, 0),
//             "\n  ".to_string(),
//             "".to_string()
//         )
//     );
//     let mut ans = parse_next_token(&mut input, &mut state, 5).unwrap();
//     ans.set_span(Span::new());
//     assert_eq!(
//         ans,
//         Token::PointyTag(
//             Span::new2(0, 1, 0, 0, 1, 0),
//             "<(t)line|".to_string(),
//             BasicName {
//                 view: "t".to_string(),
//                 special: false,
//                 prefix: "".to_string(),
//                 local: "line".to_string()
//             },
//             '<', '|'
//         )
//     );
//     let mut ans = parse_next_token(&mut input, &mut state, 5).unwrap();
//     ans.set_span(Span::new());
//     assert_eq!(
//         ans,
//         Token::PointyTag(
//             Span::new2(0, 1, 0, 0, 1, 0),
//             "<(g)sentence|".to_string(),
//             BasicName {
//                 view: "g".to_string(),
//                 special: false,
//                 prefix: "".to_string(),
//                 local: "sentence".to_string()
//             },
//             '<', '|'
//         )
//     );
//     let mut ans = parse_next_token(&mut input, &mut state, 5).unwrap();
//     ans.set_span(Span::new());
//     assert_eq!(
//         ans,
//         Token::InlineText(
//             Span::new2(0, 1, 0, 0, 1, 0),
//             "I, by attorney, bless thee from thy mother,".to_string(),
//             "I, by attorney, bless thee from thy mother,".to_string(),
//         )
//     );
//     let mut ans = parse_next_token(&mut input, &mut state, 5).unwrap();
//     ans.set_span(Span::new());
//     assert_eq!(
//         ans,
//         Token::PointyTag(
//             Span::new2(0, 1, 0, 0, 1, 0),
//             "|(t)line>".to_string(),
//             BasicName {
//                 view: "t".to_string(),
//                 special: false,
//                 prefix: "".to_string(),
//                 local: "line".to_string()
//             },
//             '|', '>'
//         )
//     );
//     let mut ans = parse_next_token(&mut input, &mut state, 5).unwrap();
//     ans.set_span(Span::new());
//     assert_eq!(
//         ans,
//         Token::InlineText(
//             Span::new2(0, 1, 0, 0, 1, 0),
//             "\n  ".to_string(),
//             "".to_string()
//         )
//     );
//     let mut ans = parse_next_token(&mut input, &mut state, 5).unwrap();
//     ans.set_span(Span::new());
//     assert_eq!(
//         ans,
//         Token::PointyTag(
//             Span::new2(0, 1, 0, 0, 1, 0),
//             "<(t)line|".to_string(),
//             BasicName {
//                 view: "t".to_string(),
//                 special: false,
//                 prefix: "".to_string(),
//                 local: "line".to_string()
//             },
//             '<', '|'
//         )
//     );
// }

use crate::basic_parser::*;

fn quick_input(input: &'static str) -> Box<PeekReader> {
    Box::new(PeekReader::new(Box::new(input.bytes())))
}

#[test]
fn test_no_input() {
    let input = quick_input("");
    let mut parser = BasicParser::new(input);
    let ans = parser.next();
    assert_eq!(ans, Err(ParserError::EndOfInput));
}

#[test]
fn test_text_1() {
    let input = quick_input(" \t hi!  \n   a");
    let mut parser = BasicParser::new(input);
    let ans = parser.next();
    assert_eq!(
        ans,
        Ok(Tag {
            name: BasicName::new("", true, "", "text"),
            kind: TagType::Virtual,
            raw: " \t hi!  \n   a".to_string(),
            span: Span::new2(0, 1, 0, 13, 2, 4),
            attrs: vec![Attr {
                name: BasicName::new("", true, "", "val"),
                val: AttrValue::String(" \t hi!\na".to_string()),
            }]
        })
    );
    let ans = parser.next();
    assert_eq!(ans, Err(ParserError::EndOfInput));
}

#[test]
fn test_tag_1() {
    let input = quick_input("{icon \n }");
    let mut parser = BasicParser::new(input);
    let ans = parser.next();
    assert_eq!(
        ans,
        Ok(Tag {
            name: BasicName::new("", false, "", "icon"),
            kind: TagType::CurlyFull,
            span: Span::new2(0, 1, 0, 9, 2, 2),
            raw: "{icon \n }".to_string(),
            attrs: vec![]
        })
    );
    let ans = parser.next();
    assert_eq!(ans, Err(ParserError::EndOfInput));
}

#[test]
fn test_tag_2() {
    let input = quick_input("{icon !id=\"hi\\t\" \n n=3014\t}");
    let mut parser = BasicParser::new(input);
    let ans = parser.next();
    assert_eq!(
        ans,
        Ok(Tag {
            name: BasicName::new("", false, "", "icon"),
            kind: TagType::CurlyFull,
            span: Span::new2(0, 1, 0, 27, 2, 9),
            raw: "{icon !id=\"hi\\t\" \n n=3014\t}".to_string(),
            attrs: vec![Attr{
                name: BasicName::new("", true, "", "!id"),
                val: AttrValue::String("hi\t".to_string())
            },Attr{
                name: BasicName::new("", false, "", "n"),
                val: AttrValue::Integer(3014)
            }]
        })
    );
    let ans = parser.next();
    assert_eq!(ans, Err(ParserError::EndOfInput));
}

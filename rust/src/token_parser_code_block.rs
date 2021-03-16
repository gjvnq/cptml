use crate::prelude::*;
use crate::token_parser::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CodeBlockMode {
    Counting,
    ReadingLang,
    ReadingContent,
    Stop,
}

pub(crate) fn parse_code_block(src: &mut PeekReader) -> Result<Token, AnyError> {
    let mut raw = String::new();
    let mut lang = String::new();
    let mut content = String::new();
    let mut mode = CodeBlockMode::Counting;
    let mut ticks_start = 0;
    let mut ticks_counter = 0;
    let mut cycle = false;

    let start = src.get_pos();

    loop {
        let c1 = src.peek(1)?;
        match mode {
            CodeBlockMode::Stop => break,
            CodeBlockMode::Counting => match c1 {
                '`' => ticks_counter += 1,
                _ => {
                    if ticks_start == 0 {
                        ticks_start = ticks_counter;
                        mode = CodeBlockMode::ReadingLang;
                        cycle = true;
                    } else if ticks_counter == ticks_start {
                        mode = CodeBlockMode::Stop;
                    } else if ticks_counter < ticks_start {
                        mode = CodeBlockMode::ReadingContent;
                        cycle = true;
                        for _ in 0..ticks_counter {
                            content.push('`')
                        }
                    } else {
                        for _ in 0..(ticks_counter - ticks_start) {
                            content.push('`')
                        }
                    }

                    if c1 == '\0' {
                        break;
                    }
                }
            },
            CodeBlockMode::ReadingLang => match c1 {
                '\0' => {
                    mode = CodeBlockMode::Stop;
                    cycle = true
                }
                ' ' | '\n' | '\t' => mode = CodeBlockMode::ReadingContent,
                '`' => {
                    mode = CodeBlockMode::Counting;
                    ticks_counter = 0;
                    cycle = true;
                }
                _ => lang.push(c1),
            },
            CodeBlockMode::ReadingContent => match c1 {
                '\0' => {
                    mode = CodeBlockMode::Stop;
                    cycle = true
                }
                '`' => {
                    mode = CodeBlockMode::Counting;
                    ticks_counter = 0;
                    cycle = true;
                }
                _ => content.push(c1),
            },
        }
        if cycle {
            cycle = false
        } else {
            raw.push(src.pop()?);
        }
    }

    let end = src.get_pos();
    let span = Span::new_from_to(start, end);

    if content.len() == 0 {
        std::mem::swap(&mut lang, &mut content)
    }

    return Ok(Token {
        span: span,
        raw: raw,
        val: TokenKind::CodeBlock {
            lang: lang,
            content: content,
        },
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_code_block_1() {
        let input_str = "```a```";
        let mut input = PeekReader::from_str(&input_str).unwrap();
        let ans = parse_code_block(&mut input);
        assert_eq!(
            ans,
            Ok(Token {
                span: Span::new2(0, 1, 0, 7, 1, 7),
                raw: input_str.to_string(),
                val: TokenKind::CodeBlock {
                    lang: "".to_string(),
                    content: "a".to_string()
                }
            })
        );
    }

    #[test]
    fn test_parse_code_block_2a() {
        let input_str = "``` a b```";
        let mut input = PeekReader::from_str(&input_str).unwrap();
        let ans = parse_code_block(&mut input);
        assert_eq!(
            ans,
            Ok(Token {
                span: Span::new2(0, 1, 0, 10, 1, 10),
                raw: input_str.to_string(),
                val: TokenKind::CodeBlock {
                    lang: "".to_string(),
                    content: "a b".to_string()
                }
            })
        );
    }

    #[test]
    fn test_parse_code_block_2b() {
        let input_str = "```\na b```";
        let mut input = PeekReader::from_str(&input_str).unwrap();
        let ans = parse_code_block(&mut input);
        assert_eq!(
            ans,
            Ok(Token {
                span: Span::new2(0, 1, 0, 10, 2, 6),
                raw: input_str.to_string(),
                val: TokenKind::CodeBlock {
                    lang: "".to_string(),
                    content: "a b".to_string()
                }
            })
        );
    }

    #[test]
    fn test_parse_code_block_3() {
        let input_str = "```js function f();```";
        let mut input = PeekReader::from_str(&input_str).unwrap();
        let ans = parse_code_block(&mut input);
        assert_eq!(
            ans,
            Ok(Token {
                span: Span::new2(0, 1, 0, 22, 1, 22),
                raw: input_str.to_string(),
                val: TokenKind::CodeBlock {
                    lang: "js".to_string(),
                    content: "function f();".to_string()
                }
            })
        );
    }

    #[test]
    fn test_parse_code_block_4() {
        let input_str = "```js my text `` a ```";
        let mut input = PeekReader::from_str(&input_str).unwrap();
        let ans = parse_code_block(&mut input);
        assert_eq!(
            ans,
            Ok(Token {
                span: Span::new2(0, 1, 0, 22, 1, 22),
                raw: input_str.to_string(),
                val: TokenKind::CodeBlock {
                    lang: "js".to_string(),
                    content: "my text `` a ".to_string()
                }
            })
        );
    }

    #[test]
    fn test_parse_code_block_5() {
        let input_str = "```js my text ```````";
        let mut input = PeekReader::from_str(&input_str).unwrap();
        let ans = parse_code_block(&mut input);
        assert_eq!(
            ans,
            Ok(Token {
                span: Span::new2(0, 1, 0, 21, 1, 21),
                raw: input_str.to_string(),
                val: TokenKind::CodeBlock {
                    lang: "js".to_string(),
                    content: "my text ````".to_string()
                }
            })
        );
    }
}

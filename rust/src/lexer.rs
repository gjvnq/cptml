use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete;
use nom::character::complete::{alphanumeric1, char, multispace0, satisfy};
use nom::combinator::{map, peek, recognize};
use nom::multi::many0;
use nom::sequence::{pair, separated_pair};
use nom::IResult;
use unicode_xid::UnicodeXID;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct IdFullName<'a> {
    namespace: &'a str,
    localname: &'a str,
}

fn valid_xid_start(ch: char) -> bool {
    return UnicodeXID::is_xid_start(ch);
}

fn valid_xid_continue(ch: char) -> bool {
    return ch == '-' || UnicodeXID::is_xid_continue(ch);
}

// Parses <XID_START> <XID_CONTINUE>*
pub fn xid_name(input: &str) -> nom::IResult<&str, &str> {
    let mut input_chars = input.char_indices();

    match input_chars.next() {
        Some((_, ch)) => {
            if !valid_xid_start(ch) {
                // Using ErrorKind::Alpha is a bit wrong but it's good enough for now. Unfortunately there's no ErrorKind::XidStart
                return Err(nom::Err::Error(nom::error::Error::new(
                    input,
                    nom::error::ErrorKind::Alpha,
                )));
            }
        }
        None => {
            return Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Eof,
            )));
        }
    }

    let mut last_pos = 0;
    loop {
        match input_chars.next() {
            Some((pos, ch)) => {
                if !valid_xid_continue(ch) {
                    last_pos = pos;
                    break;
                }
            }
            None => {
                last_pos = input.len();
                break;
            }
        }
    }

    return Ok((&input[last_pos..], &input[..last_pos]));
}

// E.g. "!id", "!cptml"
pub fn idfullname_special(input: &str) -> nom::IResult<&str, (&str, &str)> {
    pair(recognize(char('!')), xid_name)(input)
}

// E.g. "namespace:name", "namespace:color"
pub fn idfullname_regular(input: &str) -> nom::IResult<&str, (&str, &str)> {
    separated_pair(xid_name, char(':'), xid_name)(input)
}

// E.g. "name", "color"
pub fn idfullname_local(input: &str) -> nom::IResult<&str, (&str, &str)> {
    map(xid_name, |s: &str| ("", s))(input)
}

pub fn idfullname(input: &str) -> nom::IResult<&str, IdFullName> {
    let (input, (namespace, localname)) =
        alt((idfullname_special, idfullname_regular, idfullname_local))(input)?;
    Ok((
        input,
        IdFullName {
            namespace: namespace,
            localname: localname,
        },
    ))
}

#[derive(Debug, Clone, PartialEq)]
pub enum TagAttrValue {
    Integer(i64),
    // Float(f64),
    // String(str),
    Boolean(bool),
}

pub fn parse_bool_true(input: &str) -> nom::IResult<&str, bool> {
    let (input, _) = tag("true")(input)?;
    Ok((input, true))
}

pub fn parse_bool_false(input: &str) -> nom::IResult<&str, bool> {
    let (input, _) = tag("false")(input)?;
    Ok((input, false))
}

pub fn tag_args_bool(input: &str) -> nom::IResult<&str, TagAttrValue> {
    let (input, flag) = alt((parse_bool_true, parse_bool_false))(input)?;
    Ok((input, TagAttrValue::Boolean(flag)))
}

// TODO: support underscores as thousands separator
pub fn integer_hex(input: &str) -> nom::IResult<&str, i64> {
    let (input, _) = tag("0x")(input)?;
    let (input, val) = complete::hex_digit1(input)?;
    Ok((
        input,
        i64::from_str_radix(val, 16).expect("valid hexadecimal integer"),
    ))
}

// TODO: support underscores as thousands separator
pub fn integer_dec(input: &str) -> nom::IResult<&str, i64> {
    let (input, val) = complete::i64(input)?;
    Ok((input, val))
}

pub fn tag_args_integer(input: &str) -> nom::IResult<&str, TagAttrValue> {
    let (input, val) = alt((integer_hex, integer_dec))(input)?;
    Ok((input, TagAttrValue::Integer(val)))
}

pub fn tag_args_pair<'a>(
    input: &'a str,
) -> nom::IResult<&'a str, (&'a str, IdFullName<'a>, TagAttrValue)> {
    let (input, whitespace) = multispace0(input)?;
    let (input, name) = idfullname(input)?;
    let (input, _) = char('=')(input)?;
    let (input, val) = alt((tag_args_bool, tag_args_integer))(input)?;
    Ok((input, (whitespace, name, val)))
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct CurlyTagStart<'a> {
    element: IdFullName<'a>,
    args: Vec<(&'a str, IdFullName<'a>, TagAttrValue)>,
    whitespace: &'a str,
}

pub fn curly_tag_start<'a>(input: &'a str) -> nom::IResult<&'a str, CurlyTagStart<'a>> {
    let (input, _) = recognize(char('{'))(input)?;
    let (input, element) = idfullname(input)?;
    let (input, args) = many0(tag_args_pair)(input)?;
    let (input, whitespace) = multispace0(input)?;
    let (input, _) = recognize(char(';'))(input)?;

    Ok((
        input,
        CurlyTagStart {
            element: element,
            args: args,
            whitespace: whitespace,
        },
    ))
}

#[cfg(test)]
mod tests {
    use crate::lexer::*;
    use nom::error::ErrorKind::{Alpha, Char, Digit, Eof, Tag};
    use nom::Err::Error as NomError;

    #[test]
    fn test_curly_tag_start() {
        assert_eq!(
            curly_tag_start(""),
            Err(nom::Err::Error(nom::error::Error {
                input: "",
                code: Char
            }))
        );
        assert_eq!(
            curly_tag_start("{"),
            Err(nom::Err::Error(nom::error::Error {
                input: "",
                code: Eof
            }))
        );
        assert_eq!(
            curly_tag_start("{span; "),
            Ok((
                " ",
                CurlyTagStart {
                    element: IdFullName {
                        namespace: "",
                        localname: "span"
                    },
                    args: vec![],
                    whitespace: "",
                }
            ))
        );
        assert_eq!(
            curly_tag_start("{!cptml\t; "),
            Ok((
                " ",
                CurlyTagStart {
                    element: IdFullName {
                        namespace: "!",
                        localname: "cptml"
                    },
                    args: vec![],
                    whitespace: "\t",
                }
            ))
        );
        assert_eq!(
            curly_tag_start("{tei:span ;"),
            Ok((
                "",
                CurlyTagStart {
                    element: IdFullName {
                        namespace: "tei",
                        localname: "span"
                    },
                    args: vec![],
                    whitespace: " ",
                }
            ))
        );
        assert_eq!(
            curly_tag_start("{tei:span !id=4 html:show=false ;"),
            Ok((
                "",
                CurlyTagStart {
                    element: IdFullName {
                        namespace: "tei",
                        localname: "span"
                    },
                    args: vec![
                        (
                            " ",
                            IdFullName {
                                namespace: "!",
                                localname: "id"
                            },
                            TagAttrValue::Integer(4)
                        ),
                        (
                            " ",
                            IdFullName {
                                namespace: "html",
                                localname: "show"
                            },
                            TagAttrValue::Boolean(false)
                        )
                    ],
                    whitespace: " ",
                }
            ))
        );
    }

    #[test]
    fn test_tag_integer() {
        assert_eq!(
            tag_args_integer(""),
            Err(nom::Err::Error(nom::error::Error {
                input: "",
                code: Digit
            }))
        );
        assert_eq!(tag_args_integer("0"), Ok(("", TagAttrValue::Integer(0))));
        assert_eq!(
            tag_args_integer("874938432809"),
            Ok(("", TagAttrValue::Integer(874938432809)))
        );
        assert_eq!(
            tag_args_integer("-34343432"),
            Ok(("", TagAttrValue::Integer(-34343432)))
        );
        assert_eq!(tag_args_integer("0xA"), Ok(("", TagAttrValue::Integer(10))));
    }

    #[test]
    fn test_tag_bool() {
        assert_eq!(
            tag_args_bool(""),
            Err(nom::Err::Error(nom::error::Error {
                input: "",
                code: Tag
            }))
        );
        assert_eq!(tag_args_bool("true"), Ok(("", TagAttrValue::Boolean(true))));
        assert_eq!(
            tag_args_bool("false"),
            Ok(("", TagAttrValue::Boolean(false)))
        );
        assert_eq!(
            tag_args_bool("t"),
            Err(nom::Err::Error(nom::error::Error {
                input: "t",
                code: Tag
            }))
        );
        assert_eq!(
            tag_args_bool("f"),
            Err(nom::Err::Error(nom::error::Error {
                input: "f",
                code: Tag
            }))
        );
    }

    #[test]
    fn test_xid_name() {
        assert_eq!(
            xid_name(""),
            Err(nom::Err::Error(nom::error::Error {
                input: "",
                code: Eof
            }))
        );
        assert_eq!(
            xid_name("_"),
            Err(nom::Err::Error(nom::error::Error {
                input: "_",
                code: Alpha
            }))
        );
        assert_eq!(
            xid_name("-"),
            Err(nom::Err::Error(nom::error::Error {
                input: "-",
                code: Alpha
            }))
        );
        assert_eq!(xid_name("a"), Ok(("", "a")));
        assert_eq!(xid_name("my-tag"), Ok(("", "my-tag")));
        assert_eq!(xid_name("my_tag"), Ok(("", "my_tag")));
        assert_eq!(xid_name("ただの-名前"), Ok(("", "ただの-名前")));
    }

    #[test]
    fn test_idfullname_special() {
        assert_eq!(idfullname_special("!cptml"), Ok(("", ("!", "cptml"))));
        assert_eq!(idfullname_special("!href"), Ok(("", ("!", "href"))));
        assert_eq!(idfullname_special("!名前"), Ok(("", ("!", "名前"))));
        assert_eq!(
            idfullname_special("cptml"),
            Err(nom::Err::Error(nom::error::Error {
                input: "cptml",
                code: Char
            }))
        );
    }
    #[test]
    fn test_idfullname_regular() {
        assert_eq!(idfullname_regular("ns1:img"), Ok(("", ("ns1", "img"))));
        assert_eq!(idfullname_regular("ns2:span"), Ok(("", ("ns2", "span"))));
        assert_eq!(idfullname_regular("ns3:名前"), Ok(("", ("ns3", "名前"))));
        assert_eq!(
            idfullname_regular("img"),
            Err(nom::Err::Error(nom::error::Error {
                input: "",
                code: Char
            }))
        );
    }
    #[test]
    fn test_idfullname_local() {
        assert_eq!(idfullname_local("img"), Ok(("", ("", "img"))));
        assert_eq!(idfullname_local("span"), Ok(("", ("", "span"))));
        assert_eq!(idfullname_local("名前"), Ok(("", ("", "名前"))));
        assert_eq!(idfullname_local("ns:名前"), Ok((":名前", ("", "ns"))));
    }
}

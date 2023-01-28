use nom::IResult;
use nom::character::complete::{char, alphanumeric1, satisfy};
use nom::combinator::{recognize, map, peek};
use nom::sequence::{pair, separated_pair};
use nom::branch::{alt};
use unicode_xid::UnicodeXID;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct IdFullName<'a> {
    namespace: &'a str,
    localname: &'a str,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CurlyTagStart<'a> {
    src: &'a str,
    name: IdFullName<'a>,
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
        Some((_, ch)) => if !valid_xid_start(ch) {
            // Using ErrorKind::Alpha is a bit wrong but it's good enough for now. Unfortunately there's no ErrorKind::XidStart
            return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Alpha)));
        },
        None => {return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Eof)));}
    }

    let mut last_pos = 0;
    loop {
        match input_chars.next() {
            Some((pos, ch)) => if !valid_xid_continue(ch) {
                last_pos = pos;
                break
            },
            None => {
                last_pos = input.len();
                break
            }
        }
    }

    return Ok((
        &input[last_pos..],
        &input[..last_pos]
    ))
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
    let (input, (namespace, localname)) = alt((idfullname_special, idfullname_regular, idfullname_local))(input)?;
    Ok((input, IdFullName{
        namespace: namespace,
        localname: localname
    }))
}

pub fn curly_tag_start(input: &str) -> nom::IResult<&str, IdFullName> {
    let (input, _) = recognize(char('{'))(input)?;
    idfullname(input)
}

#[cfg(test)]
mod tests {
    use crate::lexer::*;
    use nom::error::ErrorKind::{Alpha, Char, Eof};
    use nom::Err::Error as NomError;

    #[test]
    fn test_curly_tag_start() {
        assert_eq!(curly_tag_start(""), Err(nom::Err::Error(nom::error::Error { input: "", code: Eof })));
        assert_eq!(curly_tag_start("{"), Err(nom::Err::Error(nom::error::Error { input: "{", code: Eof })));
        assert_eq!(curly_tag_start("{span "), Ok((" ", IdFullName{
            namespace:  "",
            localname: "span"
        })));
    }

    #[test]
    fn test_xid_name() {
        assert_eq!(xid_name(""), Err(nom::Err::Error(nom::error::Error { input: "", code: Eof })));
        assert_eq!(xid_name("_"), Err(nom::Err::Error(nom::error::Error { input: "_", code: Alpha })));
        assert_eq!(xid_name("-"), Err(nom::Err::Error(nom::error::Error { input: "-", code: Alpha })));
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
        assert_eq!(idfullname_special("cptml"), Err(nom::Err::Error(nom::error::Error { input: "cptml", code: Char })));
    }
    #[test]
    fn test_idfullname_regular() {
        assert_eq!(idfullname_regular("ns1:img"), Ok(("", ("ns1", "img"))));
        assert_eq!(idfullname_regular("ns2:span"), Ok(("", ("ns2", "span"))));
        assert_eq!(idfullname_regular("ns3:名前"), Ok(("", ("ns3", "名前"))));
        assert_eq!(idfullname_regular("img"), Err(nom::Err::Error(nom::error::Error { input: "", code: Char })));
    }
    #[test]
    fn test_idfullname_local() {
        assert_eq!(idfullname_local("img"), Ok(("", ("", "img"))));
        assert_eq!(idfullname_local("span"), Ok(("", ("", "span"))));
        assert_eq!(idfullname_local("名前"), Ok(("", ("", "名前"))));
        assert_eq!(idfullname_local("ns:名前"), Ok((":名前", ("", "ns"))));
    }
}
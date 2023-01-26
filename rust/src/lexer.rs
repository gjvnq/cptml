use nom::IResult;
use nom::character::complete::{char, alphanumeric1, satisfy};
use nom::combinator::{recognize, map, peek};
use nom::sequence::{pair, separated_pair};
use nom::branch::{alt};
use unicode_xid::UnicodeXID;

#[derive(Debug, Clone, PartialEq)]
pub struct IdFullName<'a> {
    namespace: &'a str,
    localname: &'a str,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CurlyTagStart<'a> {
    src: &'a str,
    name: IdFullName<'a>,
}


// Parses <XID_START> <XID_CONTINUE>*
pub fn xid_name(input: &str) -> nom::IResult<&str, &str> {
    let mut input_chars = input.char_indices();

    match input_chars.next() {
        Some((_, ch)) => if !UnicodeXID::is_xid_start(ch) {
            // Using ErrorKind::Alpha is a bit wrong but it's good enough for now. Unfortunately there's no ErrorKind::XidStart
            return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Alpha)));
        },
        None => {return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Eof)));}
    }

    let mut last_pos = 0;
    loop {
        match input_chars.next() {
            Some((pos, ch)) => if !UnicodeXID::is_xid_continue(ch) {
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

#[cfg(test)]
mod tests {
    use crate::lexer::*;

    #[test]
    fn idfullname_test() {
        println!("{:?}", xid_name(""));
        println!("{:?}", xid_name(" "));
        println!("{:?}", xid_name("çà_ "));
        println!("{:?}", xid_name("çà_-a"));

        println!("{:?}", idfullname("!id"));
        println!("{:?}", idfullname("name"));
        println!("{:?}", idfullname("hello:world"));
        println!("{:?}", idfullname("helláo:worlçd"));
        println!("{:?}", idfullname(":hello:world"));
        println!("{:?}", idfullname("!hello:world"));
    }
}
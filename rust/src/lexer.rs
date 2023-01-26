use nom::IResult;
use nom::character::complete::{char, alphanumeric1, satisfy};
use nom::combinator::{recognize, map, peek};
use nom::sequence::{pair, separated_pair};
use nom::branch::{alt};
use unicode_xid::UnicodeXID;

#[derive(Debug, Clone, PartialEq)]
pub struct TagName {
    namespace: String,
    localname: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CurlyTagStart {
    src: String,
    name: TagName,
}


pub fn xid_name(input: &str) -> nom::IResult<&str, &str> {
    let mut input_chars = input.char_indices();

    match input_chars.next() {
        Some((_, ch)) => if !UnicodeXID::is_xid_start(ch) {
            panic!("not xid_start")
        },
        None => panic!("input too short")
    }

    let mut last_pos = 0;
    loop {
        match input_chars.next() {
            Some((pos, ch)) => match UnicodeXID::is_xid_continue(ch) {
                true => {last_pos = pos},
                false => {last_pos = pos; break}
            },
            None => {last_pos = input.len(); break}
        }
    }

    return Ok((
        &input[last_pos..],
        &input[..last_pos]
    ))
}

pub fn tagname_special(input: &str) -> nom::IResult<&str, (&str, &str)> {
    pair(recognize(char('!')), xid_name)(input)
}

pub fn tagname_regular(input: &str) -> nom::IResult<&str, (&str, &str)> {
    separated_pair(xid_name, char(':'), xid_name)(input)
}

pub fn tagname_local(input: &str) -> nom::IResult<&str, (&str, &str)> {
    map(xid_name, |s: &str| ("", s))(input)
}

pub fn tagname(input: &str) -> nom::IResult<&str, TagName> {
	let (input, (namespace, localname)) = alt((tagname_special, tagname_regular, tagname_local))(input)?;
    Ok((input, TagName{
        namespace: namespace.to_string(),
        localname: localname.to_string()
    }))
}

#[cfg(test)]
mod tests {
    use crate::lexer::*;

    #[test]
    fn tagname_test() {
        println!("{:?}", xid_name("çà_ "));
        println!("{:?}", xid_name("çà_-a"));

        println!("{:?}", tagname("!id"));
        println!("{:?}", tagname("name"));
        println!("{:?}", tagname("hello:world"));
        println!("{:?}", tagname("helláo:worlçd"));
        println!("{:?}", tagname(":hello:world"));
        // println!("{:?}", tagname_special(":hello:world"));
    }
}
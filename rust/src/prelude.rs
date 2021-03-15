pub use crate::chars::CharErrorEnum;
pub use crate::peek_reader::PeekReader;
pub use crate::pos::Position;
pub use crate::pos::Span;

pub trait ByteReader: core::fmt::Debug + Iterator<Item = u8> {}
impl ByteReader for std::str::Bytes<'_> {}

pub type CResult<T> = Result<T, AnyError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnyError {
    CharError(CharErrorEnum),
    IllegalEscapeSequence(Position, String),
    OutOfRange(isize, isize, isize),
    WrongAttrType(WrongAttrTypeEnum),
    EndOfInput,
    FauxPanic(String),
    // IllegalChar(Position, char),
    // IllegalChar2(Position, char, Vec<char>),
    // IllegalCharMsg(Position, char, String),
    // MissingTerminator(Position, char),
    // MissingLocalName(Position),
    // IllegalEscapeSequence(Position, String),
    // IllegalNumber(Span, String),
    // NotAttributeValue(Span, String),
    NotReadyYet,
}

impl std::convert::From<CharErrorEnum> for AnyError {
    fn from(err: CharErrorEnum) -> Self {
        AnyError::CharError(err)
    }
}

impl std::convert::From<WrongAttrTypeEnum> for AnyError {
    fn from(err: WrongAttrTypeEnum) -> Self {
        AnyError::WrongAttrType(err)
    }
}

fn non_empty_or_none(val: Option<&str>) -> Option<String> {
    match val {
        Some(val) => match val.trim().len() {
            0 => None,
            _ => Some(val.to_string()),
        },
        None => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BasicName {
    pub view: Option<String>,
    pub prefix: Option<String>,
    pub local: String,
    pub raw: Option<String>,
}

impl BasicName {
    pub fn is_empty(&self) -> bool {
        return self.view.is_none()
            && self.prefix.is_none()
            && self.local.len() == 0
            && self.raw.is_none();
    }
    pub fn new(
        view: Option<&str>,
        prefix: Option<&str>,
        local: &str,
        raw: Option<&str>,
    ) -> BasicName {
        BasicName {
            view: non_empty_or_none(view),
            prefix: non_empty_or_none(prefix),
            local: local.trim().to_string(),
            raw: raw.map(|s| s.to_string()),
        }
    }
    pub fn new_empty() -> BasicName {
        BasicName {
            view: None,
            prefix: None,
            local: "".to_string(),
            raw: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WrongAttrTypeEnum {
    NotANumber(AttrValue),
    NotAString(AttrValue),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttrValue {
    NumVal { num: i64, places: u16 },
    StringVal { parsed: String },
}

impl std::convert::TryFrom<AttrValue> for bool {
    type Error = WrongAttrTypeEnum;
    fn try_from(val: AttrValue) -> Result<bool, WrongAttrTypeEnum> {
        if let AttrValue::NumVal { num, .. } = val {
            Ok(num != 0)
        } else {
            Err(WrongAttrTypeEnum::NotANumber(val))
        }
    }
}

impl std::convert::TryFrom<AttrValue> for i64 {
    type Error = WrongAttrTypeEnum;
    fn try_from(val: AttrValue) -> Result<i64, WrongAttrTypeEnum> {
        if let AttrValue::NumVal { num, places, .. } = val {
            let mut ans = num;
            for _ in 0..places {
                ans /= 10;
            }
            Ok(ans)
        } else {
            Err(WrongAttrTypeEnum::NotANumber(val))
        }
    }
}

impl std::convert::TryFrom<AttrValue> for f64 {
    type Error = WrongAttrTypeEnum;
    fn try_from(val: AttrValue) -> Result<f64, WrongAttrTypeEnum> {
        if let AttrValue::NumVal { num, places, .. } = val {
            let mut ans = num as f64;
            for _ in 0..places {
                ans /= 10.0;
            }
            Ok(ans)
        } else {
            Err(WrongAttrTypeEnum::NotANumber(val))
        }
    }
}

impl std::convert::TryFrom<AttrValue> for String {
    type Error = WrongAttrTypeEnum;
    fn try_from(val: AttrValue) -> Result<String, WrongAttrTypeEnum> {
        if let AttrValue::StringVal { parsed: ans, .. } = val {
            Ok(ans)
        } else {
            Err(WrongAttrTypeEnum::NotAString(val))
        }
    }
}

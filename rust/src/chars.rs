// use crate::prelude::*;
use unicode_xid::UnicodeXID;

#[inline]
pub fn is_reserved_char(c: char) -> bool {
    return c == '<' || c == '>' || c == '{' || c == '}' || c == '\\' || c == '|'
}

#[inline]
fn is_id_other(c: char) -> bool {
    return c == '_' || c == '-' || c == '$'
}

#[inline]
pub fn is_id_start(c: char) -> bool {
    return is_id_other(c) || UnicodeXID::is_xid_start(c);
}

#[inline]
pub fn is_id_continue(c: char) -> bool {
    return is_id_other(c) || UnicodeXID::is_xid_continue(c);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CharErrorEnum {
    InvalidFirstByte(u8),
    ScalarTooLarge(u32),
    SurrogateInUtf8(u32),
    SliceTooShort(usize, Vec<u8>)
}

use CharErrorEnum::{InvalidFirstByte, ScalarTooLarge, SurrogateInUtf8, SliceTooShort};

pub fn u32_to_char(val: u32) -> Result<char,CharErrorEnum> {
    if val > 0x10ffff {
        return Err(ScalarTooLarge(val));
    }
    if 0xD800 <= val && val <= 0xDFFF {
        return Err(SurrogateInUtf8(val));
    }

    unsafe { Ok(std::mem::transmute(val)) }
}

pub fn bytes_to_char(v: &[u8]) -> Result<(char, usize), CharErrorEnum> {
    let mut ans: u32;
    let size: usize;

    if v.len() < 1 {
        return Err(SliceTooShort(1, v.to_vec()));
    }

    let v0: u32 = v[0].into();

    if v0 < 0x80 {
        ans = v0.into();
        size = 1;
    } else if (v0 & 0xe0) == 0xc0 {
        if v.len() < 2 {
            return Err(SliceTooShort(2, v.to_vec()));
        }
        let v1: u32 = v[1].into();
        ans = (v0 & 0x1f) << 6;
        ans |= (v1 & 0x3f) << 0;
        size = 2;
    } else if (v0 & 0xf0) == 0xe0 {
        if v.len() < 3 {
            return Err(SliceTooShort(3, v.to_vec()));
        }
        let v1: u32 = v[1].into();
        let v2: u32 = v[2].into();
        ans = (v0 & 0x0f) << 12;
        ans |= (v1 & 0x3f) << 6;
        ans |= (v2 & 0x3f) << 0;
        size = 3;
    } else if (v0 & 0xf8) == 0xf0 && (v0 <= 0xf4) {
        if v.len() < 4 {
            return Err(SliceTooShort(4, v.to_vec()));
        }
        let v1: u32 = v[1].into();
        let v2: u32 = v[2].into();
        let v3: u32 = v[3].into();
        ans = (v0 & 0x07) << 18;
        ans |= (v1 & 0x3f) << 12;
        ans |= (v2 & 0x3f) << 6;
        ans |= (v3 & 0x3f) << 0;
        size = 4;
    } else {
        return Err(InvalidFirstByte(v[0]));
    }

    if ans > 0x10ffff {
        return Err(ScalarTooLarge(ans));
    }
    if 0xD800 <= ans && ans <= 0xDFFF {
        return Err(SurrogateInUtf8(ans));
    }

    unsafe { Ok((std::mem::transmute(ans), size)) }
}

pub fn char_size(v0: u8) -> Result<usize, CharErrorEnum> {
    if v0 < 0x80 {
        Ok(1)
    } else if (v0 & 0xe0) == 0xc0 {
        Ok(2)
    } else if (v0 & 0xf0) == 0xe0 {
        Ok(3)
    } else if (v0 & 0xf8) == 0xf0 && (v0 <= 0xf4) {
        Ok(4)
    } else {
        Err(InvalidFirstByte(v0))
    }
}

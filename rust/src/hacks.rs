use core::fmt::Debug;

pub trait ByteReader: Debug + Iterator<Item = u8> {}

impl ByteReader for std::str::Bytes<'_> {}

pub fn is_valid_id_first_char(c: char) -> bool {
    c.is_alphabetic()
}

pub fn is_valid_id_next_char(c: char) -> bool {
    c == '_' || c.is_alphanumeric()
}

pub fn bytes_to_char(v: &[u8]) -> (char, usize) {
    let mut ans: u32;
    let size: usize;
    let v0: u32 = v[0].into();

    if v0 < 0x80 {
        ans = v0.into();
        size = 1;
    } else if (v0 & 0xe0) == 0xc0 {
        let v1: u32 = v[1].into();
        ans = (v0 & 0x1f) << 6;
        ans |= (v1 & 0x3f) << 0;
        size = 2;
    } else if (v0 & 0xf0) == 0xe0 {
        let v1: u32 = v[1].into();
        let v2: u32 = v[2].into();
        ans = (v0 & 0x0f) << 12;
        ans |= (v1 & 0x3f) << 6;
        ans |= (v2 & 0x3f) << 0;
        size = 3;
    } else if (v0 & 0xf8) == 0xf0 && (v0 <= 0xf4) {
        let v1: u32 = v[1].into();
        let v2: u32 = v[2].into();
        let v3: u32 = v[3].into();
        ans = (v0 & 0x07) << 18;
        ans |= (v1 & 0x3f) << 12;
        ans |= (v2 & 0x3f) << 6;
        ans |= (v3 & 0x3f) << 0;
        size = 4;
    } else {
        panic!("invalid UTF-8 'first' byte: {:?}", v0);
    }

    if ans > 0x10ffff {
        panic!("invalid Unicode scalar (too large): {:?}", ans);
    }
    if 0xD800 <= ans && ans <= 0xDFFF {
        panic!("invalid Unicode scalar (surrogate in UTF-8): {:?}", ans);
    }

    unsafe { (std::mem::transmute(ans), size) }
}

pub fn char_size(v0: u8) -> usize {
    if v0 < 0x80 {
        1
    } else if (v0 & 0xe0) == 0xc0 {
        2
    } else if (v0 & 0xf0) == 0xe0 {
        3
    } else if (v0 & 0xf8) == 0xf0 && (v0 <= 0xf4) {
        4
    } else {
        panic!("invalid UTF-8 'first' byte: {:?}", v0);
    }
}

pub fn read_single_char(v: &[u8]) -> (char, usize) {
	let mut ans : u32;
	let size : usize;

	// I always read four bytes on purpose: it makes it easier to catch bugs earlier
	let v0 : u32 = v[0].into();
	let v1 : u32 = v[1].into();
	let v2 : u32 = v[2].into();
	let v3 : u32 = v[3].into();

	if v0 < 0x80 {
		ans = v0.into();
		size = 1;
	} else if (v0 & 0xe0) == 0xc0 {
		ans =  (v0 & 0x1f) <<  6;
		ans |= (v1 & 0x3f) <<  0;
		size = 2;
	} else if (v0 & 0xf0) == 0xe0 {
		ans =  (v0 & 0x0f) << 12;
		ans |= (v1 & 0x3f) <<  6;
		ans |= (v2 & 0x3f) <<  0;
		size = 3;
	} else if (v0 & 0xf8) == 0xf0 && (v0 <= 0xf4) {
		ans =  (v0 & 0x07) << 18;
		ans |= (v1 & 0x3f) << 12;
		ans |= (v2 & 0x3f) <<  6;
		ans |= (v3 & 0x3f) <<  0;
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

	unsafe {
		(std::mem::transmute(ans), size)
	}
}
use crate::chars::{bytes_to_char, char_size};
use crate::prelude::*;
use core::fmt::Debug;

const READ_BUF_SIZE: usize = 64;
const PEEK_RESERVE: usize = 4;
const PEEK_RESERVE_I: isize = PEEK_RESERVE as isize;
const READ_LIMIT: usize = READ_BUF_SIZE - PEEK_RESERVE;

#[derive(Debug)]
pub struct PeekReader {
    buf: [char; READ_BUF_SIZE],
    buf_pos: usize,
    pos: Position,
    src: Box<dyn ByteReader>,
    eof: bool,
}

impl PeekReader {
    pub fn new(reader: Box<dyn ByteReader>) -> CResult<Self> {
        let mut ans = PeekReader {
            buf: ['\0'; READ_BUF_SIZE],
            buf_pos: 0,
            eof: false,
            pos: Position::new(),
            src: reader,
        };
        ans.read_cycle()?;
        Ok(ans)
    }

    pub fn from_str(input: &'static str) -> CResult<PeekReader> {
        PeekReader::new(Box::new(input.bytes()))
    }

    pub fn get_pos(&self) -> Position {
        self.pos
    }

    pub fn peek_string(&self, from: isize, to: isize) -> CResult<String> {
        let mut ans = String::new();
        for i in from..(to + 1) {
            ans.push(self.peek(i)?);
        }
        Ok(ans)
    }

    // 0 is the element you have just popped.
    pub fn peek(&self, dist: isize) -> CResult<char> {
        if !(-PEEK_RESERVE_I + 1 <= dist && dist <= PEEK_RESERVE_I) {
            return Err(AnyError::OutOfRange(
                dist,
                -PEEK_RESERVE_I + 1,
                PEEK_RESERVE_I,
            ));
        }
        let tmp = (self.buf_pos) as isize + dist - 1;
        if tmp < 0 || tmp >= (self.buf.len() as isize) {
            return Err(AnyError::OutOfRange(tmp, 0, (self.buf.len() - 1) as isize));
        }
        Ok(self.buf[tmp as usize])
    }

    pub fn pop(&mut self) -> CResult<char> {
        if self.buf_pos > READ_LIMIT {
            self.read_cycle()?;
        }
        let ans = self.buf[self.buf_pos];
        self.buf_pos += 1;
        self.pos.step(ans);
        Ok(ans)
    }

    fn read_cycle(&mut self) -> CResult<()> {
        // Copy chars we had reserved for peeking to the begining of the buffer
        if self.pos.byte != 0 {
            let mut i = 0;
            let mut j = self.buf_pos - PEEK_RESERVE;
            while j < self.buf.len() {
                self.buf[i] = self.buf[j];
                i += 1;
                j += 1;
            }
        }

        let mut i = match self.pos.byte {
            0 => PEEK_RESERVE as usize,
            _ => self.buf.len() - self.buf_pos + PEEK_RESERVE,
        };
        self.buf_pos = PEEK_RESERVE as usize;
        let mut buf: [u8; 4] = [0; 4];
        while i < self.buf.len() {
            // Read a single byte
            let res = self.src.next();
            if res.is_none() {
                self.eof = true;
                buf[0] = b'\0';
            } else {
                buf[0] = res.unwrap();
            }
            // Find out how many bytes we will need to read for this character
            let s = char_size(buf[0])?;
            for j in 1..s {
                // Read more bytes if necessary (code ponts above 007F)
                let res = match self.src.next() {
                    Some(val) => val,
                    _ => {
                        return Err(AnyError::CharError(CharErrorEnum::SliceTooShort(
                            s,
                            buf.to_vec(),
                        )))
                    }
                };
                buf[j] = res;
            }
            // Decode the character and save it
            self.buf[i] = bytes_to_char(&buf)?.0;
            i += 1;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::peek_reader::*;

    #[test]
    fn it_works() {
        let s = "hello world!§abcdefghijklmnopqrstuvwxyz ABCDEFGHIJKLMNOPQRSTUVWXYZ\n1§ªº冬";
        let mut parser = PeekReader::new(Box::new(s.bytes())).unwrap();
        assert_eq!(Ok('\0'), parser.peek(-3));
        assert_eq!(Ok('\0'), parser.peek(-2));
        assert_eq!(Ok('\0'), parser.peek(-1));
        assert_eq!(Ok('\0'), parser.peek(0));
        assert_eq!(Ok('h'), parser.peek(1));
        assert_eq!(Ok('h'), parser.pop());
        assert_eq!(Ok('e'), parser.pop());
        assert_eq!(Ok('l'), parser.pop());
        assert_eq!(Ok('l'), parser.pop());
        assert_eq!(Ok('o'), parser.pop());
        assert_eq!(Ok(' '), parser.pop());
        assert_eq!(Ok('w'), parser.pop());
        assert_eq!(Ok('o'), parser.pop());
        assert_eq!(Ok('r'), parser.pop());
        assert_eq!(Ok('l'), parser.pop());
        assert_eq!(Ok('d'), parser.pop());
        assert_eq!(Ok('!'), parser.pop());
        assert_eq!(Ok('§'), parser.pop());
        for c in "abcdefghijklmnopqrstuvwxyz ABCDEFGHIJKLMNOPQRSTUVWXYZ".chars() {
            assert_eq!(Ok(c), parser.pop());
        }
        assert_eq!(Ok('\n'), parser.pop());
        assert_eq!(Ok('X'), parser.peek(-3));
        assert_eq!(Ok('Y'), parser.peek(-2));
        assert_eq!(Ok('Z'), parser.peek(-1));
        assert_eq!(Ok('\n'), parser.peek(0));
        assert_eq!(Ok('1'), parser.peek(1));
        assert_eq!(Ok('§'), parser.peek(2));
        assert_eq!(Ok('ª'), parser.peek(3));
        assert_eq!(Ok('º'), parser.peek(4));
        assert_eq!(Ok('1'), parser.pop());
        assert_eq!(Ok('§'), parser.pop());
        assert_eq!(Ok('ª'), parser.pop());
        assert_eq!(Ok('º'), parser.pop());
        assert_eq!(Ok('冬'), parser.pop());
        assert_eq!(Ok('\0'), parser.pop());
        assert_eq!(Ok('\0'), parser.pop());
        assert_eq!(Ok('\0'), parser.pop());
    }
}

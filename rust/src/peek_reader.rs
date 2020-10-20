use crate::hacks::{bytes_to_char, char_size};
use crate::pos::Position;
use crate::hacks::ByteReader;
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
    pub fn new(reader: Box<dyn ByteReader>) -> Self {
        let mut ans = PeekReader {
            buf: ['\0'; READ_BUF_SIZE],
            buf_pos: 1,
            eof: false,
            pos: Position::new(),
            src: reader,
        };
        ans.read_cycle();
        ans
    }

    pub fn get_pos(&self) -> Position {
        self.pos
    }

    pub fn peek(&mut self, dist: isize) -> char {
        if !(-PEEK_RESERVE_I <= dist && dist <= PEEK_RESERVE_I) {
            panic!("PeekReader::peek(n={}), n must be between {} and {}", dist, -PEEK_RESERVE_I, PEEK_RESERVE);
        }
        let tmp = (self.buf_pos) as isize + dist;
        if tmp < 0 {
            panic!("Something went horribly wrong: dist={} buf_pos={} tmp={} {:?}", dist, self.buf_pos, tmp, self);
        }
        self.buf[tmp as usize]
    }

    pub fn pop(&mut self) -> char {
        if self.buf_pos > READ_LIMIT {
            self.read_cycle()
        }
        let ans = self.buf[self.buf_pos];
        self.buf_pos += 1;
        self.pos.step(ans);
        ans
    }

    fn read_cycle(&mut self) {
        // Copy chars we had reserved for peeking to the begining of the buffer
        if self.pos.byte != 0 {
            let mut i = 0;
            let mut j = self.buf_pos - PEEK_RESERVE;
            println!("j = {}", j);
            while j < self.buf.len() {
                println!("{} <- {}", self.buf[i], self.buf[j]);
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
            let s = char_size(buf[0]);
            for j in 1..s {
                // Read more bytes if necessary (code ponts above 007F)
                let res = self.src.next();
                buf[j] = res.unwrap();
            }
            // Decode the character and save it
            self.buf[i] = bytes_to_char(&buf).0;
            i += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::peek_reader::*;

    #[test]
    fn it_works() {
        let s = "hello world!§abcdefghijklmnopqrstuvwxyz ABCDEFGHIJKLMNOPQRSTUVWXYZ\n1§ªº冬";
        let mut parser = PeekReader::new(Box::new(s.bytes()));
        assert_eq!('\0', parser.peek(-1));
        assert_eq!('h', parser.peek(0));
        assert_eq!('e', parser.peek(1));
        assert_eq!('h', parser.pop());
        assert_eq!('e', parser.pop());
        assert_eq!('l', parser.pop());
        assert_eq!('l', parser.pop());
        assert_eq!('o', parser.pop());
        assert_eq!(' ', parser.pop());
        assert_eq!('w', parser.pop());
        assert_eq!('o', parser.pop());
        assert_eq!('r', parser.pop());
        assert_eq!('l', parser.pop());
        assert_eq!('d', parser.pop());
        assert_eq!('!', parser.pop());
        assert_eq!('§', parser.pop());
        for c in "abcdefghijklmnopqrstuvwxyz ABCDEFGHIJKLMNOPQRSTUVWXYZ".chars() {
            assert_eq!(c, parser.pop());
        }
            // println!("{:?}", parser);
        assert_eq!('\n', parser.pop());
        assert_eq!('X', parser.peek(-4));
        assert_eq!('Y', parser.peek(-3));
        assert_eq!('Z', parser.peek(-2));
        assert_eq!('\n', parser.peek(-1));
        assert_eq!('1', parser.peek(0));
        assert_eq!('§', parser.peek(1));
        assert_eq!('ª', parser.peek(2));
        assert_eq!('º', parser.peek(3));
        assert_eq!('冬', parser.peek(4));
        assert_eq!('1', parser.pop());
        assert_eq!('§', parser.pop());
        assert_eq!('ª', parser.pop());
        assert_eq!('º', parser.pop());
        assert_eq!('冬', parser.pop());
        assert_eq!('\0', parser.pop());
        assert_eq!('\0', parser.pop());
        assert_eq!('\0', parser.pop());
    }
}

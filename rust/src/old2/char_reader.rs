const DEFAULT_PEEKING_WINDOW: usize = 3;

#[inline]
pub fn is_char_new_line(c: char) -> bool {
    // CR (U+000D) is not included here to avoid trouble with the CR+LF sequence
    return c == '\u{000A}' || c == '\u{000B}' || c == '\u{000C}' || c == '\u{0085}' || c == '\u{2028}' || c == '\u{2029}'
}


#[derive(Debug,Clone,Copy)]
pub struct CharPos {
    byte: usize,
    line: usize,
    col: usize,
}

impl CharPos {
    pub fn new(byte: usize, line: usize, col: usize) -> CharPos {
        CharPos{
            byte: byte,
            line: line,
            col: col
        }
    }

    pub fn byte(&self) -> usize {
        self.byte
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn col(&self) -> usize {
        self.col
    }

    pub fn walk_opt(&self, c: Option<char>) -> CharPos {
        match c {
            Some(c) => self.walk(c),
            None => *self
        }
    }

    pub fn walk(&self, c: char) -> CharPos {
        let byte = self.byte + c.len_utf8();
        if is_char_new_line(c) {
            CharPos {
                byte: byte,
                line: self.line+1,
                col: 0,
            }
        } else {
            CharPos {
                byte: byte,
                line: self.line,
                col: self.col+1,
            }
        }
    }
}

#[derive(Debug,Clone,Copy)]
pub struct CharAndPos {
    char: char,
    pos: CharPos,
}

impl CharAndPos {
    pub fn new(char: char, pos: CharPos) -> CharAndPos {
        CharAndPos{
            char: char,
            pos: pos,
        }
    }

    pub fn char(&self) -> char {
        self.char
    }

    pub fn pos(&self) -> CharPos {
        self.pos
    }
}

#[derive(Debug)]
// To make this reader peekable, it works by reading a few chars beyond what it reports,
// thus giving the impression of a peekable reader. We don't use Peekable<T> because it's too limited.
pub struct CharReader<'a> {
    src_str: &'a str,
    src_iter: std::str::Chars<'a>,
    real_pos: CharPos,
    fake_pos: CharPos,
    buffer: Vec<Option<char>>,
}

impl CharReader<'_> {
    pub fn new<'a>(src_str: &'a str) -> CharReader<'a> {
        CharReader::new2(src_str, DEFAULT_PEEKING_WINDOW)
    }

    pub fn new2<'a>(src_str: &'a str, buffer_size: usize) -> CharReader<'a> {
        let mut ans = CharReader {
            src_str: src_str,
            src_iter: src_str.chars(),
            real_pos: CharPos::new(0, 0, 0),
            fake_pos: CharPos::new(0, 0, 0),
            buffer: Vec::with_capacity(buffer_size)
        };
        // Initialize buffer for peeking
        while ans.buffer.len() < buffer_size {
            let c = ans.src_iter.next();
            ans.buffer.push(c);
            ans.real_pos = ans.real_pos.walk_opt(c);
        };
        return ans
    }

    fn real_next(&mut self) {
        // Move elements so that the last position becomes available
        for i in 0..self.buffer.len()-1 {
            self.buffer[i] = self.buffer[i+1];
        }
        // Actually read the next char into the last position
        let c = self.src_iter.next();
        let last_pos = self.buffer.len()-1;
        self.buffer[last_pos] = c;
        self.real_pos = self.real_pos.walk_opt(c);
    }

    pub fn next(&mut self) -> Option<CharAndPos> {
        let c = self.buffer[0];
        self.fake_pos = self.fake_pos.walk_opt(c);
        self.real_next();
        match c {
            Some(c) => Some(CharAndPos::new(c, self.fake_pos)),
            None => None
        }
    }

    pub fn peek(&self, n: usize) -> Option<CharAndPos> {
        if n > self.buffer.len() {
            panic!("attempted to peek more than buffered (wanted={}, max={})", n, self.buffer.len());
        }

        let mut pos = self.fake_pos;
        for i in 0..n {
            pos = pos.walk_opt(self.buffer[i]);
        }
        match self.buffer[n] {
            Some(c) => Some(CharAndPos::new(c, pos)),
            None => None
        }
    }

    pub fn byte(&self) -> usize {
        self.fake_pos.byte()
    }

    pub fn line(&self) -> usize {
        self.fake_pos.line()
    }

    pub fn col(&self) -> usize {
        self.fake_pos.col()
    }
}
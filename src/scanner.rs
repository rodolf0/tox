use std::io;

static WHITE: &'static str = " \n\r\t";

// A Scanner reads chars into a growing window,
// once enough chars have been read to take action
// a string can be extracted and the window collapsed
pub struct Scanner<R: io::Reader> {
    rdr: io::BufferedReader<R>,
    buf: Vec<char>,
    pub pos: isize //TODO: don't make this public
}


impl Scanner<io::MemReader> {
    // Build a MathLexer reading from a string
    pub fn from_str(e: &str) -> Scanner<io::MemReader> {
        let b = io::MemReader::new(e.as_bytes().to_vec());
        Scanner::new(b)
    }
}

impl<R: io::Reader> Scanner<R> {
    // Build Scanner from generic reader
    pub fn new(r: R) -> Scanner<R> {
        Scanner{
            rdr: io::BufferedReader::new(r),
            buf: Vec::new(),
            pos: -1
        }
    }

    //Read the next char
    pub fn next(&mut self) -> Option<char> {
        self.pos += 1;
        let pos = self.pos as usize;
        // reached end of buffer, fetch more chars
        if pos >= self.buf.len() {
            match self.rdr.read_char() {
                Ok(c) => self.buf.push(c),
                Err(ref e) if e.kind == io::EndOfFile => {
                    self.pos = self.buf.len() as isize;
                    return None;
                },
                Err(e) => panic!("Scanner::next failed: {}", e)
            }
        }
        self.curr()
    }

    // Current char the scanner is on
    pub fn curr(&self) -> Option<char> {
        if self.pos < 0 {
            return None;
        }
        let pos = self.pos as usize;
        if pos >= self.buf.len() {
            return None;
        }
        Some(self.buf[pos])
    }

    // Read the prev char
    pub fn prev(&mut self) -> Option<char> {
        if self.pos >= 0 {
            self.pos -= 1;
        }
        return self.curr();
    }

    // Take a look at the next char without advancing
    pub fn peek(&mut self) -> Option<char> {
        let backtrack = self.pos;
        let peeked = self.next();
        self.pos = backtrack;
        peeked
    }

    // Check if the scanner reached EOF
    pub fn eof(&self) -> bool {
        self.pos as usize >= self.buf.len()
    }

    // Take a peep at what the scanner is currently holding
    pub fn view(&self) -> &[char] {
        let n = self.pos as usize + 1;
        self.buf.slice_to(n)
    }

    // Extract the current buffer and reset the scanner
    pub fn extract(&mut self) -> String {
        let ret = self.view().iter().cloned().collect();
        self.ignore();
        return ret;
    }

    // Ignore current view
    pub fn ignore(&mut self) {
        if self.pos >= 0 {
            let n = self.pos as usize + 1;
            self.buf = self.buf.slice_from(n).to_vec();
        }
        self.pos = -1;
    }

}

impl<R: io::Reader> Scanner<R> {
    // Advance the scanner only if the next char is in the 'any' set
    // after accept returns true self.curr() should return the matched char
    pub fn accept(&mut self, any: &str) -> Option<char> {
        match self.peek() {
            None => None,
            Some(next) => {
                if let Some(idx) = any.find(next) {
                    assert!(self.next().is_some());
                    return Some(any.char_at(idx));
                }
                None
            }
        }
    }

    // Skip over the 'over' set, return if the scanner was advanced
    // after skip a call to self.curr() will return the last matching char
    pub fn skip(&mut self, over: &str) -> bool {
        let mut advanced = false;
        while self.accept(over).is_some() {
            advanced = true;
        }
        return advanced;
    }

    // Advance the scanner until we find a char in the 'any' set or
    // we reach EOF. Returns true if the scanner was advanced.
    // After until a call to self.curr() returns the last non-matching char
    pub fn until(&mut self, any: &str) -> bool {
        let mut advanced = false;
        while let Some(next) = self.peek() {
            if any.find(next).is_some() {
                break;
            }
            assert!(self.next().is_some());
            advanced = true;
        }
        return advanced;
    }

    // Advance until WHITE or EOF
    pub fn until_ws(&mut self) -> bool { self.until(WHITE) }

    // Skip over white-space
    pub fn skip_ws(&mut self) -> bool { self.skip(WHITE) }

    // After skipping over-white space, drop the current window
    pub fn ignore_ws(&mut self) {
        self.skip_ws();
        self.ignore();
    }
}


#[cfg(test)]
mod test {
    #[test]
    fn test_extremes() {
        let mut s = super::Scanner::from_str("just a test buffer@");
        assert_eq!(s.prev(), None);
        assert_eq!(s.next(), Some('j'));
        assert_eq!(s.prev(), None);
        while s.next() != Some('@') {}
        assert_eq!(s.curr(), Some('@'));
        assert_eq!(s.prev(), Some('r'));
        assert_eq!(s.prev(), Some('e'));
        assert_eq!(s.next(), Some('r'));
        assert_eq!(s.next(), Some('@'));
        assert_eq!(s.next(), None);
        assert!(s.eof());
    }

    #[test]
    fn test_extract() {
        let mut s = super::Scanner::from_str("just a test buffer@");
        for _ in range(0, 4) { assert!(s.next().is_some()); }
        assert_eq!(s.extract().as_slice(), "just");
        assert_eq!(s.peek(), Some(' '));
        assert_eq!(s.prev(), None);
    }

    #[test]
    fn test_accept() {
        let mut s = super::Scanner::from_str("heey  you!");
        assert!(!s.skip_ws());
        assert_eq!(s.prev(), None);
        assert_eq!(s.accept("he"), Some('h'));
        assert_eq!(s.curr(), Some('h'));
        assert_eq!(s.accept("he"), Some('e'));
        assert_eq!(s.curr(), Some('e'));
        assert_eq!(s.accept("hye"), Some('e'));
        assert_eq!(s.accept("e"), None);
        assert_eq!(s.accept("hey"), Some('y'));
        assert!(s.skip_ws());
        assert!(!s.skip_ws());
        assert_eq!(s.curr(), Some(' '));
        assert_eq!(s.peek(), Some('y'));
        assert_eq!(s.next(), Some('y'));
        assert_eq!(s.next(), Some('o'));
    }

    #[test]
    fn test_skips() {
        let mut s = super::Scanner::from_str("heey  you!");
        assert_eq!(s.accept("h"), Some('h'));
        assert!(s.skip("hey"));
        assert!(!s.skip("hey"));
        assert_eq!(s.curr(), Some('y'));
        assert!(s.until("!"));
        assert!(!s.until("!"));
        assert_eq!(s.accept("!"), Some('!'));
        assert_eq!(s.next(), None);
        assert_eq!(s.curr(), None);
        assert!(s.eof());
    }
}

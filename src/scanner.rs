use std::io;

static WHITE: &'static str = " \n\r\t";

// A Scanner reads chars into a growing window,
// once enough chars have been read to take action
// a string can be extracted and the window collapsed
pub struct Scanner<R: io::Reader> {
    rdr: io::BufferedReader<R>,
    buf: Vec<char>,
    pub pos: int //TODO: don't make this public
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
        let pos = self.pos as uint;
        if pos >= self.buf.len() {
            match self.rdr.read_char() {
                Err(ref e) if e.kind == io::EndOfFile => {
                    self.pos = self.buf.len() as int;
                    return None;
                },
                Err(e) => fail!("Scanner::next failed: {}", e),
                Ok(c) => self.buf.push(c)
            }
        }
        self.curr()
    }

    // Current char the scanner is on
    pub fn curr(&self) -> Option<char> {
        if self.pos < 0 {
            return None;
        }
        let pos = self.pos as uint;
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
        self.pos as uint >= self.buf.len()
    }

    // Take a peep at what the scanner is currently holding
    pub fn view(&self) -> &[char] {
        let n = self.pos as uint + 1;
        self.buf.slice_to(n)
    }

    // Extract the current buffer and reset the scanner
    pub fn extract(&mut self) -> String {
        let ret = String::from_chars(self.view());
        self.ignore();
        return ret;
    }

    // Ignore current view
    pub fn ignore(&mut self) {
        if self.pos >= 0 {
            let n = self.pos as uint + 1;
            self.buf = self.buf.slice_from(n).to_vec();
        }
        self.pos = -1;
    }

}

impl<R: io::Reader> Scanner<R> {
    // Advance the scanner only if the next char is in the 'any' set
    // after accept returns true self.curr() should return the matched char
    pub fn accept(&mut self, any: &str) -> bool {
        match self.peek() {
            None => return false,
            Some(next) => {
                if any.find(next).is_some() {
                    assert!(self.next().is_some());
                    return true;
                }
                return false;
            }
        }
    }

    // Skip over the 'over' set, return if the scanner was advanced
    // after skip a call to self.curr() will return the last matching char
    pub fn skip(&mut self, over: &str) -> bool {
        let mut advanced = false;
        while self.accept(over) {
            advanced = true;
        }
        return advanced;
    }

    // Skip over white-space
    pub fn skip_ws(&mut self) -> bool {
      self.skip(WHITE)
    }

    // After skipping over-white space, drop the current window
    pub fn ignore_ws(&mut self) {
        self.skip_ws();
        self.ignore();
    }
}


#[cfg(test)]
mod test {
    use std::io;

    #[test]
    fn test_extremes() {
        let b = io::MemReader::new(b"just a test buffer@".to_vec());
        let mut s = super::Scanner::new(b);
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
        let b = io::MemReader::new(b"just a test buffer@".to_vec());
        let mut s = super::Scanner::new(b);
        for _ in range(0u, 4) { assert!(s.next().is_some()); }
        assert_eq!(s.extract().as_slice(), "just");
        assert_eq!(s.peek(), Some(' '));
        assert_eq!(s.prev(), None);
    }

    #[test]
    fn test_accept() {
        let b = io::MemReader::new(b"heey  you!".to_vec());
        let mut s = super::Scanner::new(b);
        assert!(!s.skip_ws());
        assert_eq!(s.prev(), None);
        assert!(s.accept("h"));
        assert_eq!(s.curr(), Some('h'));
        assert!(s.accept("e"));
        assert_eq!(s.curr(), Some('e'));
        assert!(s.accept("e"));
        assert!(!s.accept("e"));
        assert!(s.accept("y"));
        assert!(s.skip_ws());
        assert!(!s.skip_ws());
        assert_eq!(s.curr(), Some(' '));
        assert_eq!(s.peek(), Some('y'));
        assert_eq!(s.next(), Some('y'));
        assert_eq!(s.next(), Some('o'));
    }

    #[test]
    fn test_skips() {
        let b = io::MemReader::new(b"heey  you!".to_vec());
        let mut s = super::Scanner::new(b);
        assert!(s.accept("h"));
        assert!(s.skip("hey"));
        assert!(!s.skip("hey"));
        assert_eq!(s.curr(), Some('y'));
        assert!(!s.skip("you"));
        assert!(s.skip(" oy"));
        assert_eq!(s.next(), Some('u'));
        assert!(s.accept("!"));
        assert_eq!(s.next(), None);
        assert_eq!(s.curr(), None);
        assert!(s.eof());
    }
}

use std::io;

static WHITE: &'static str = " \n\r\t";

#[deriving(Show, PartialEq)]
pub enum ScannerErr {
    BOF,
    EOF,
    Other(io::IoError)
}

pub type ScannerResult<T> = Result<T, ScannerErr>;

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
    pub fn next(&mut self) -> ScannerResult<char> {
        self.pos += 1;
        let pos = self.pos as uint;
        if pos >= self.buf.len() {
            match self.rdr.read_char() {
                Err(ref e) if e.kind == io::EndOfFile => {
                    self.pos = self.buf.len() as int;
                    return Err(EOF);
                },
                Err(e) => return Err(Other(e)),
                Ok(c) => self.buf.push(c)
            }
        }
        self.curr()
    }

    // Current char the scanner is on
    pub fn curr(&self) -> ScannerResult<char> {
        if self.pos < 0 {
            return Err(BOF);
        }
        let pos = self.pos as uint;
        if pos >= self.buf.len() {
            return Err(EOF);
        }
        Ok(self.buf[pos])
    }

    // Read the prev char
    pub fn prev(&mut self) -> ScannerResult<char> {
        if self.pos >= 0 {
            self.pos -= 1;
        }
        return self.curr();
    }

    // Take a look at the next char without advancing
    pub fn peek(&mut self) -> ScannerResult<char> {
        let pos = self.pos;
        let ret = self.next();
        self.pos = pos;
        return ret;
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
    pub fn accept(&mut self, any: &str) -> ScannerResult<bool> {
        let next = try!(self.peek());
        if any.find(next).is_some() {
            assert!(self.next().is_ok());
            return Ok(true);
        }
        Ok(false)
    }

    // Skip over the 'over' set, return if the scanner was advanced
    // after skip a call to self.curr() will return the last matching char
    pub fn skip(&mut self, over: &str) -> ScannerResult<bool> {
        let mut advanced = false;
        loop {
            match self.accept(over) {
                Ok(true) => advanced = true,
                Ok(false) => return Ok(advanced),
                Err(EOF) if advanced => return Ok(true),
                Err(e) => return Err(e)
            }
        }
    }

    // Advance until a char in the 'find' set, return if advanced
    // after until a call to self.curr() should return the last non-matching char
    pub fn until(&mut self, any: &str) -> ScannerResult<bool> {
        let mut advanced = false;
        loop {
            match self.peek() {
                Ok(next) => {
                    if any.find(next).is_some() {
                        return Ok(advanced);
                    }
                    assert!(self.next().is_ok());
                    advanced = true;
                },
                Err(EOF) if advanced => return Ok(true),
                Err(e) => return Err(e)
            }
        }
    }

    pub fn skip_ws(&mut self) -> ScannerResult<bool> { self.skip(WHITE) }
    pub fn until_ws(&mut self) -> ScannerResult<bool> { self.until(WHITE) }
}



#[cfg(test)]
mod test {
    use std::io;

    #[test]
    fn test_extremes() {
        let b = io::MemReader::new(b"just a test buffer@".to_vec());
        let mut s = super::Scanner::new(b);

        assert_eq!(s.prev(), Err(super::BOF));
        while s.next() != Ok('@') {}
        assert_eq!(s.curr(), Ok('@'));
        assert_eq!(s.next(), Err(super::EOF));
    }

    #[test]
    fn test_extract() {
        let b = io::MemReader::new(b"just a test buffer@".to_vec());
        let mut s = super::Scanner::new(b);

        for _ in range(0u, 4) {
            assert!(s.next().is_ok());
        }
        assert_eq!(s.extract(), "just".to_string());
        assert_eq!(s.peek(), Ok(' '));
        assert_eq!(s.prev(), Err(super::BOF));
    }

    #[test]
    fn test_accept() {
        let b = io::MemReader::new(b"heey  you!".to_vec());
        let mut s = super::Scanner::new(b);

        assert_eq!(s.skip_ws(), Ok(false));
        assert_eq!(s.accept("h"), Ok(true));
        assert_eq!(s.accept("e"), Ok(true));
        assert_eq!(s.accept("e"), Ok(true));
        assert_eq!(s.accept("e"), Ok(false));
        assert_eq!(s.accept("y"), Ok(true));
        assert_eq!(s.skip_ws(), Ok(true));
        assert_eq!(s.skip_ws(), Ok(false));
        assert_eq!(s.curr(), Ok(' '));
        assert_eq!(s.peek(), Ok('y'));
        assert_eq!(s.until("uoy"), Ok(false)); // won't advance because next is 'y'
        assert_eq!(s.until("uo"), Ok(true));   // advance up to prev-'o'
        assert_eq!(s.curr(), Ok('y'));
        assert_eq!(s.until("!"), Ok(true));   // advance up to prev-'u'
        assert_eq!(s.curr(), Ok('u'));
        assert_eq!(s.accept("!"), Ok(true));

        assert_eq!(s.skip_ws(), Err(super::EOF));
        assert_eq!(s.until_ws(), Err(super::EOF));
    }

    #[test]
    fn test_skips() {
        let b = io::MemReader::new(b"heey  you!".to_vec());
        let mut s = super::Scanner::new(b);
        assert_eq!(s.accept("h"), Ok(true));
        assert_eq!(s.until("@"), Ok(true));
        assert_eq!(s.until("@"), Err(super::EOF));

        let b = io::MemReader::new(b"heey  you!".to_vec());
        let mut s = super::Scanner::new(b);
        assert_eq!(s.accept("h"), Ok(true));
        assert_eq!(s.skip("heyou !"), Ok(true));
        assert_eq!(s.skip("heyou !"), Err(super::EOF));
    }
}

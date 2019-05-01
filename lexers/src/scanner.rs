#![deny(warnings)]

pub struct Scanner<I: Iterator>
where
    I::Item: Clone,
{
    src: I,
    buf: Vec<I::Item>,
    pos: isize,
}

// Scanners are Iterators
impl<I> Iterator for Scanner<I>
where
    I: Iterator,
    I::Item: Clone,
{
    type Item = I::Item;
    fn next(&mut self) -> Option<Self::Item> {
        self.pos += 1;
        // Check if we need to fill the buffer
        let lacking = self.pos - (self.buf.len() as isize) + 1;
        if lacking > 0 {
            self.buf.extend(self.src.by_ref().take(lacking as usize));
        }
        // limit the buffer position to the buffer length at most
        self.pos = std::cmp::min(self.pos, self.buf.len() as isize);
        self.current()
    }
}

impl<I> Scanner<I>
where
    I: Iterator,
    I::Item: Clone,
{
    pub fn new(source: I) -> Scanner<I> {
        Scanner {
            src: source,
            buf: Vec::new(),
            pos: -1,
        }
    }

    // Allows getting current buffer position to backtrack
    pub fn buffer_pos(&self) -> isize {
        self.pos
    }

    // Reset buffer position, normally used for backtracking
    // If position is out of bounds set_buffer_pos returns false
    pub fn set_buffer_pos(&mut self, pos: isize) -> bool {
        if pos < -1 || pos > (self.buf.len() as isize) {
            return false;
        }
        self.pos = pos;
        true
    }

    // Returns the current token on which the scanner is positioned
    pub fn current(&self) -> Option<I::Item> {
        let pos = self.pos as usize;
        if self.pos < 0 || pos >= self.buf.len() {
            return None;
        }
        Some(self.buf[pos].clone())
    }

    // Steps the scanner back and returns the token at that position
    pub fn prev(&mut self) -> Option<I::Item> {
        if self.pos >= 0 {
            self.pos -= 1;
        }
        self.current()
    }

    // Returns the token ahead without actually advancing the scanner
    pub fn peek(&mut self) -> Option<I::Item> {
        let backtrack = self.pos;
        let peeked = self.next();
        self.pos = backtrack;
        peeked
    }

    // Returns the previous token without actually backtracking the scanner
    pub fn peek_prev(&mut self) -> Option<I::Item> {
        let backtrack = self.pos;
        let peeked = self.prev();
        self.pos = backtrack;
        peeked
    }

    // Returns a view of the current underlying buffer
    pub fn view(&self) -> &[I::Item] {
        let n = (self.pos + 1) as usize;
        &self.buf[..n]
    }

    // Consumes the buffer into a new token (which can be ignored)
    pub fn extract(&mut self) -> Vec<I::Item> {
        // Check where to shift buffer
        let split_point = std::cmp::min(self.pos + 1, self.buf.len() as isize);
        assert!(split_point >= 0);
        // Reset buffer cursor
        self.pos = -1;
        // Split buffer and keep the remainder
        let mut remaining = self.buf.split_off(split_point as usize);
        std::mem::swap(&mut self.buf, &mut remaining);
        remaining
    }
}

impl<I> Scanner<I>
where
    I: Iterator,
    I::Item: Clone + PartialEq,
{
    // Advance the scanner only if the next char is the expected one
    // self.current() will return the matched char if accept matched
    pub fn accept(&mut self, what: &I::Item) -> Option<I::Item> {
        let backtrack = self.buffer_pos();
        if let Some(next) = self.next() {
            if &next == what {
                return Some(next);
            }
        }
        self.set_buffer_pos(backtrack);
        None
    }

    // Advance the scanner only if the next char is in the 'any' set,
    // self.current() will return the matched char if accept matched any
    pub fn accept_any(&mut self, any: &[I::Item]) -> Option<I::Item> {
        let backtrack = self.buffer_pos();
        if let Some(next) = self.next() {
            if any.contains(&next) {
                return Some(next);
            }
        }
        self.set_buffer_pos(backtrack);
        None
    }

    // Skip over the 'over' set, result is if the scanner was advanced,
    // self.current() will return the last matching char
    pub fn skip_all(&mut self, over: &[I::Item]) -> bool {
        let mut advanced = false;
        while self.accept_any(over).is_some() {
            advanced = true;
        }
        advanced
    }

    // Find an element in the 'any' set or EOF, return if the scanner advanced,
    // self.current() returns the last non-matching char
    pub fn until_any(&mut self, any: &[I::Item]) -> bool {
        let mut advanced = false;
        while let Some(next) = self.peek() {
            if any.contains(&next) {
                break;
            }
            self.next();
            advanced = true;
        }
        advanced
    }
}

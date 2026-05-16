pub struct Scanner<I: Iterator>
where
    I::Item: Clone,
{
    src: I,
    buf: Vec<I::Item>,
    matched_len: usize,
}

// Scanners are Iterators
impl<I> Iterator for Scanner<I>
where
    I: Iterator,
    I::Item: Clone,
{
    type Item = I::Item;
    fn next(&mut self) -> Option<Self::Item> {
        self.matched_len += 1;
        // Check if we need to fill the buffer
        if self.matched_len > self.buf.len() {
            if let Some(item) = self.src.next() {
                self.buf.push(item);
            }
        }
        // limit the buffer position to the buffer length at most
        self.matched_len = std::cmp::min(self.matched_len, self.buf.len() + 1);
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
            matched_len: 0,
        }
    }

    // Allows getting current buffer position to backtrack
    pub fn buffer_pos(&self) -> usize {
        self.matched_len
    }

    // Reset buffer position, normally used for backtracking
    // If position is out of bounds set_buffer_pos returns false
    pub fn set_buffer_pos(&mut self, pos: usize) -> bool {
        if pos > self.buf.len() + 1 {
            return false;
        }
        self.matched_len = pos;
        true
    }

    // Returns the current token on which the scanner is positioned
    pub fn current(&self) -> Option<I::Item> {
        if self.matched_len == 0 || self.matched_len > self.buf.len() {
            return None;
        }
        Some(self.buf[self.matched_len - 1].clone())
    }

    // Steps the scanner back and returns the token at that position
    pub fn prev(&mut self) -> Option<I::Item> {
        if self.matched_len > 0 {
            self.matched_len -= 1;
        }
        self.current()
    }

    // Returns the token ahead without actually advancing the scanner
    pub fn peek(&mut self) -> Option<I::Item> {
        if self.matched_len == self.buf.len() {
            if let Some(item) = self.src.next() {
                self.buf.push(item);
            }
        }
        if self.matched_len < self.buf.len() {
            Some(self.buf[self.matched_len].clone())
        } else {
            None
        }
    }

    // Returns the previous token without actually backtracking the scanner
    pub fn peek_prev(&mut self) -> Option<I::Item> {
        if self.matched_len <= 1 {
            None
        } else {
            Some(self.buf[self.matched_len - 2].clone())
        }
    }

    // Returns a view of the current underlying buffer
    pub fn view(&self) -> &[I::Item] {
        let n = std::cmp::min(self.matched_len, self.buf.len());
        &self.buf[..n]
    }

    // Consumes the buffer into a new token (which can be ignored)
    pub fn extract(&mut self) -> Vec<I::Item> {
        let end = std::cmp::min(self.matched_len, self.buf.len());
        self.matched_len = 0;
        self.buf.drain(..end).collect()
    }
}

pub trait TokenMatcher<Item> {
    fn matches(&mut self, item: &Item) -> bool;
}

impl<Item, F> TokenMatcher<Item> for F
where
    F: FnMut(&Item) -> bool,
{
    fn matches(&mut self, item: &Item) -> bool {
        self(item)
    }
}

impl<Item: PartialEq> TokenMatcher<Item> for &[Item] {
    fn matches(&mut self, item: &Item) -> bool {
        self.contains(item)
    }
}

impl<Item: PartialEq, const N: usize> TokenMatcher<Item> for &[Item; N] {
    fn matches(&mut self, item: &Item) -> bool {
        self.contains(item)
    }
}

impl TokenMatcher<char> for char {
    fn matches(&mut self, item: &char) -> bool {
        *self == *item
    }
}

impl<I> Scanner<I>
where
    I: Iterator,
    I::Item: Clone + PartialEq,
{
    // Advance the scanner only if the next char is the expected one
    // self.current() will return the matched char if accept matched
    pub fn accept(&mut self, mut matcher: impl TokenMatcher<I::Item>) -> Option<I::Item> {
        let backtrack = self.buffer_pos();
        if let Some(next) = self.next() {
            if matcher.matches(&next) {
                return Some(next);
            }
        }
        self.set_buffer_pos(backtrack);
        None
    }

    // Advance the scanner only if a full match for items form 'what'.
    // self.current() will return the last item from 'what'
    pub fn accept_all(&mut self, what: impl Iterator<Item = I::Item>) -> bool {
        let backtrack = self.buffer_pos();
        for item in what {
            if let Some(next) = self.next() {
                if next != item {
                    self.set_buffer_pos(backtrack);
                    return false;
                }
            } else {
                self.set_buffer_pos(backtrack);
                return false;
            }
        }
        true
    }

    // Skip over the matching elements, result is if the scanner was advanced,
    // self.current() will return the last matching char
    pub fn ignore(&mut self, mut matcher: impl TokenMatcher<I::Item>) -> bool {
        let mut advanced = false;
        while let Some(next) = self.peek() {
            if matcher.matches(&next) {
                self.next();
                advanced = true;
            } else {
                break;
            }
        }
        advanced
    }

    // Find an element that matches or EOF, return if the scanner advanced,
    // self.current() returns the last non-matching char
    pub fn until(&mut self, mut matcher: impl TokenMatcher<I::Item>) -> bool {
        let mut advanced = false;
        while let Some(next) = self.peek() {
            if matcher.matches(&next) {
                break;
            }
            self.next();
            advanced = true;
        }
        advanced
    }
}

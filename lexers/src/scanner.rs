pub struct Scanner<I: Iterator>
where
    I::Item: Clone,
{
    src: I,
    buf: Vec<I::Item>,
    matched_len: usize,
}

pub trait Scan: Iterator + Sized {
    fn scanner(self) -> Scanner<Self>
    where
        Self::Item: Clone;
}

impl<I: Iterator> Scan for I
where
    I::Item: Clone,
{
    fn scanner(self) -> Scanner<Self> {
        Scanner::new(self)
    }
}

// Scanners are Iterators
impl<I> Iterator for Scanner<I>
where
    I: Iterator,
    I::Item: Clone,
{
    type Item = I::Item;
    fn next(&mut self) -> Option<Self::Item> {
        self.advance().cloned()
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Checkpoint(usize);

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

    // Returns the token ahead without actually advancing the scanner
    pub fn peek(&mut self) -> Option<&I::Item> {
        if self.matched_len >= self.buf.len() {
            if let Some(item) = self.src.next() {
                self.buf.push(item);
            } else {
                return None;
            }
        }
        Some(&self.buf[self.matched_len])
    }

    pub fn advance(&mut self) -> Option<&I::Item> {
        if self.matched_len >= self.buf.len() {
            if let Some(item) = self.src.next() {
                self.buf.push(item);
            } else {
                self.matched_len = std::cmp::min(self.matched_len + 1, self.buf.len() + 1);
                return None;
            }
        }
        self.matched_len += 1;
        Some(&self.buf[self.matched_len - 1])
    }

    // Returns the current token on which the scanner is positioned
    pub fn current(&self) -> Option<&I::Item> {
        if self.matched_len == 0 || self.matched_len > self.buf.len() {
            return None;
        }
        Some(&self.buf[self.matched_len - 1])
    }

    // Save a checkpoint for backtracking
    pub fn checkpoint(&self) -> Checkpoint {
        Checkpoint(self.matched_len)
    }

    // Restore scanner state from a checkpoint.
    // Panics if the checkpoint is out of bounds for this scanner's buffer.
    pub fn restore(&mut self, checkpoint: Checkpoint) {
        assert!(
            checkpoint.0 <= self.buf.len() + 1,
            "Checkpoint out of bounds: {} > {}",
            checkpoint.0,
            self.buf.len() + 1
        );
        self.matched_len = checkpoint.0;
    }

    // Steps the scanner back and returns the token at that position
    pub fn prev(&mut self) -> Option<&I::Item> {
        if self.matched_len > 1 && self.matched_len <= self.buf.len() + 1 {
            self.matched_len -= 1;
            Some(&self.buf[self.matched_len - 1])
        } else {
            self.matched_len = 0;
            None
        }
    }

    // Returns the previous token without actually backtracking the scanner
    pub fn peek_prev(&mut self) -> Option<&I::Item> {
        if self.matched_len <= 1 {
            None
        } else {
            Some(&self.buf[self.matched_len - 2])
        }
    }

    // Removes and returns all consumed items from the buffer, resetting the cursor
    pub fn lift(&mut self) -> impl Iterator<Item = I::Item> + '_ {
        let end = std::cmp::min(self.matched_len, self.buf.len());
        self.matched_len = 0;
        self.buf.drain(..end)
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
    // Advance the scanner only if the next item matches
    pub fn accept(&mut self, mut matcher: impl TokenMatcher<I::Item>) -> Option<&I::Item> {
        if self.matched_len >= self.buf.len() {
            if let Some(item) = self.src.next() {
                self.buf.push(item);
            } else {
                return None;
            }
        }
        if matcher.matches(&self.buf[self.matched_len]) {
            self.matched_len += 1;
            Some(&self.buf[self.matched_len - 1])
        } else {
            None
        }
    }

    // Advance the scanner as long as matcher keeps matching
    pub fn accept_while(&mut self, mut matcher: impl TokenMatcher<I::Item>) -> &[I::Item] {
        let start = self.matched_len;
        let mut lookahead = self.matched_len;
        loop {
            if lookahead >= self.buf.len() {
                if let Some(item) = self.src.next() {
                    self.buf.push(item);
                } else {
                    break;
                }
            }
            if !matcher.matches(&self.buf[lookahead]) {
                break;
            }
            lookahead += 1;
        }
        self.matched_len = lookahead;
        &self.buf[start..lookahead]
    }

    // Advance the scanner only if the full sequence is found.
    // On a partial match the scanner is reset to its original position.
    pub fn accept_seq(&mut self, what: impl Iterator<Item = I::Item>) -> bool {
        let mut lookahead = self.matched_len;
        for item in what {
            if lookahead >= self.buf.len() {
                if let Some(next_item) = self.src.next() {
                    self.buf.push(next_item);
                } else {
                    return false;
                }
            }
            if self.buf[lookahead] != item {
                return false;
            }
            lookahead += 1;
        }
        self.matched_len = lookahead;
        true
    }

    pub fn peek_seq(&mut self, what: impl Iterator<Item = I::Item>) -> bool {
        let mut lookahead = self.matched_len;
        for item in what {
            if lookahead >= self.buf.len() {
                if let Some(next_item) = self.src.next() {
                    self.buf.push(next_item);
                } else {
                    return false;
                }
            }
            if self.buf[lookahead] != item {
                return false;
            }
            lookahead += 1;
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::Scan;

    #[test]
    fn advance() {
        let mut s = "ab".chars().scanner();
        assert_eq!(s.advance(), Some(&'a'));
        assert_eq!(s.advance(), Some(&'b'));
        assert_eq!(s.advance(), None);
    }

    #[test]
    fn checkpoint() {
        let mut s = "checkpoint".chars().scanner();
        assert_eq!(s.next(), Some('c'));
        let cp = s.checkpoint();
        assert_eq!(s.next(), Some('h'));
        s.restore(cp);
        assert_eq!(s.next(), Some('h'));
        assert_eq!(s.next(), Some('e'));
        s.restore(cp);
        assert_eq!(s.next(), Some('h'));
    }

    #[test]
    fn current() {
        let mut s = "a".chars().scanner();
        assert_eq!(s.current(), None);
        assert_eq!(s.next(), Some('a'));
        assert_eq!(s.current(), Some(&'a'));
        assert_eq!(s.next(), None);
        assert_eq!(s.current(), None);
    }

    #[test]
    fn prev() {
        let mut s = "abc".chars().scanner();
        assert_eq!(s.prev(), None);
        assert_eq!(s.advance(), Some(&'a'));
        assert_eq!(s.prev(), None);
        assert_eq!(s.advance(), Some(&'a'));
        assert_eq!(s.advance(), Some(&'b'));
        assert_eq!(s.advance(), Some(&'c'));
        assert_eq!(s.prev(), Some(&'b'));
        assert_eq!(s.prev(), Some(&'a'));
        assert_eq!(s.prev(), None);
    }

    #[test]
    fn peek() {
        let mut s = "a".chars().scanner();
        assert_eq!(s.peek(), Some(&'a'));
        assert_eq!(s.advance(), Some(&'a'));
        assert_eq!(s.peek(), None);
    }

    #[test]
    fn peek_prev() {
        let mut s = "ab".chars().scanner();
        assert_eq!(s.peek_prev(), None);
        assert_eq!(s.advance(), Some(&'a'));
        assert_eq!(s.peek_prev(), None);
        assert_eq!(s.advance(), Some(&'b'));
        assert_eq!(s.peek_prev(), Some(&'a'));
        assert_eq!(s.advance(), None);
        assert_eq!(s.peek_prev(), Some(&'b'));
    }

    #[test]
    fn lift() {
        let mut s = "abc".chars().scanner();
        assert_eq!(s.advance(), Some(&'a'));
        assert_eq!(s.advance(), Some(&'b'));
        assert_eq!(s.lift().collect::<String>(), "ab");
        assert_eq!(s.current(), None);
        assert_eq!(s.advance(), Some(&'c'));
        let _ = s.lift();
        assert_eq!(s.current(), None);
        assert_eq!(s.next(), None);
    }

    #[test]
    fn accept() {
        let mut s = "abcdef".chars().scanner();
        assert_eq!(s.accept(|c: &char| *c == 'a'), Some(&'a'));
        assert_eq!(s.accept('b'), Some(&'b'));
        assert_eq!(s.accept(&['x', 'c']), Some(&'c'));
        assert_eq!(s.accept(&['x', 'c']), None);
        assert_eq!(s.accept(|c: &char| *c == 'd'), Some(&'d'));
        assert_eq!(s.accept('e'), Some(&'e'));
        assert_eq!(s.accept(vec!['x', 'f'].as_slice()), Some(&'f'));
    }

    #[test]
    fn accept_while() {
        let mut s = "abcdef".chars().scanner();
        assert_eq!(s.accept_while(&['c', 'a', 'b']), &['a', 'b', 'c']);
        assert_eq!(s.accept_while('x'), &[]);
        assert_eq!(s.accept_while(|c: &char| "bcde".contains(*c)), &['d', 'e']);
        assert_eq!(s.accept_while(vec!['x', 'f'].as_slice()), &['f']);
        assert_eq!(s.accept_while(&['a', 'b', 'c', 'd', 'e', 'f', 'g']), &[]);
    }

    #[test]
    fn accept_sequence() {
        let mut s = "abcdefghi".chars().scanner();
        assert_eq!(s.accept_seq("abc".chars()), true);
        assert_eq!(s.accept_seq("dex".chars()), false);
        assert_eq!(s.accept_seq("def".chars()), true);
        assert_eq!(s.accept_seq("ghi".chars()), true);
    }

    #[test]
    fn peek_sequence() {
        let mut s = "abcdef".chars().scanner();
        assert_eq!(s.peek_seq("abc".chars()), true);
        assert_eq!(s.accept_seq("abc".chars()), true);
        assert_eq!(s.peek_seq("def".chars()), true);
        assert_eq!(s.peek_seq("dex".chars()), false);
        assert_eq!(s.accept_seq("def".chars()), true);
        assert_eq!(s.peek_seq("".chars()), true);
    }
}

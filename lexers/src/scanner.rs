pub struct Scanner<I: Iterator>
where
    I::Item: Clone,
{
    src: I,
    buf: Vec<I::Item>,
    matched_len: usize,
}

// A convinince trait to create scanners from any iterator
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
                self.matched_len = self.buf.len() + 1; // EOF
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

    // Returns a view of the current underlying buffer
    pub fn view(&self) -> &[I::Item] {
        &self.buf[..self.matched_len()]
    }

    pub fn view_from(&self, cp: Checkpoint) -> &[I::Item] {
        &self.buf[cp.0..self.matched_len()]
    }

    pub fn matched_len(&self) -> usize {
        std::cmp::min(self.matched_len, self.buf.len())
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
    pub fn accept_while(&mut self, mut matcher: impl TokenMatcher<I::Item>) -> Option<&[I::Item]> {
        let start = self.matched_len();
        let mut lookahead = start;
        loop {
            if lookahead >= self.buf.len() {
                if let Some(item) = self.src.next() {
                    self.buf.push(item);
                } else {
                    self.matched_len = self.buf.len() + 1; // mark EOF
                    if lookahead > start {
                        return Some(&self.buf[start..lookahead]);
                    }
                    return None;
                }
            }
            if !matcher.matches(&self.buf[lookahead]) {
                break;
            }
            lookahead += 1;
        }
        self.matched_len = lookahead;
        if lookahead > start {
            return Some(&self.buf[start..lookahead]);
        }
        None
    }

    // Advance the scanner only if the full sequence is found.
    // On a partial match the scanner is reset to its original position.
    pub fn accept_seq(&mut self, what: impl Iterator<Item = I::Item>) -> Option<&[I::Item]> {
        self._peek_seq(what, true)
    }

    pub fn peek_seq(&mut self, what: impl Iterator<Item = I::Item>) -> Option<&[I::Item]> {
        self._peek_seq(what, false)
    }

    fn _peek_seq(
        &mut self,
        what: impl Iterator<Item = I::Item>,
        advance: bool,
    ) -> Option<&[I::Item]> {
        let start = self.matched_len();
        let mut lookahead = start;
        for item in what {
            if lookahead >= self.buf.len() {
                if let Some(next_item) = self.src.next() {
                    self.buf.push(next_item);
                } else {
                    // full seq not matched, shouldn't change matched_len
                    return None;
                }
            }
            if self.buf[lookahead] != item {
                return None;
            }
            lookahead += 1;
        }
        if advance {
            self.matched_len = lookahead;
        }
        if lookahead > start {
            return Some(&self.buf[start..lookahead]);
        }
        None
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
        assert_eq!(s.advance(), None); // None at EOF
    }

    #[test]
    fn checkpoint() {
        let mut s = "checkpoint".chars().scanner();
        assert_eq!(s.next(), Some('c'));
        let cp = s.checkpoint(); // Save position at 'h'
        assert_eq!(s.next(), Some('h'));
        s.restore(cp); // Backtrack to saved position
        assert_eq!(s.next(), Some('h')); // Verify successful backtrack
        assert_eq!(s.next(), Some('e'));
        s.restore(cp);
        assert_eq!(s.next(), Some('h'));
    }

    #[test]
    fn current() {
        let mut s = "a".chars().scanner();
        assert_eq!(s.current(), None); // None before first advance
        assert_eq!(s.next(), Some('a'));
        assert_eq!(s.current(), Some(&'a')); // Matches last advanced item
        assert_eq!(s.next(), None);
        assert_eq!(s.current(), None); // None at EOF
    }

    #[test]
    fn prev() {
        let mut s = "abc".chars().scanner();
        assert_eq!(s.prev(), None); // Cannot step back initially
        assert_eq!(s.advance(), Some(&'a'));
        assert_eq!(s.prev(), None); // Cannot step before start
        assert_eq!(s.advance(), Some(&'a'));
        assert_eq!(s.advance(), Some(&'b'));
        assert_eq!(s.advance(), Some(&'c'));
        assert_eq!(s.prev(), Some(&'b'));
        assert_eq!(s.prev(), Some(&'a'));
        assert_eq!(s.prev(), None); // Reached beginning of stream
    }

    #[test]
    fn peek() {
        let mut s = "a".chars().scanner();
        assert_eq!(s.peek(), Some(&'a')); // Peek without advancing
        assert_eq!(s.advance(), Some(&'a'));
        assert_eq!(s.peek(), None); // None at EOF
    }

    #[test]
    fn peek_prev() {
        let mut s = "ab".chars().scanner();
        assert_eq!(s.peek_prev(), None); // None before first advance
        assert_eq!(s.advance(), Some(&'a'));
        assert_eq!(s.peek_prev(), None); // None if only 1 advanced
        assert_eq!(s.advance(), Some(&'b'));
        assert_eq!(s.peek_prev(), Some(&'a'));
        assert_eq!(s.advance(), None); // Advance to EOF
        assert_eq!(s.peek_prev(), Some(&'b')); // Last valid item
    }

    #[test]
    fn lift_drains_buffer() {
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
        assert_eq!(s.accept(&['x', 'c']), None); // Fails, doesn't advance
        assert_eq!(s.accept(|c: &char| *c == 'd'), Some(&'d'));
        assert_eq!(s.accept('e'), Some(&'e'));
        assert_eq!(s.accept(vec!['x', 'f'].as_slice()), Some(&'f'));
    }

    #[test]
    fn accept_while() {
        let mut s = "abcdef".chars().scanner();
        assert_eq!(s.accept_while(&['c', 'a', 'b']), Some(&['a', 'b', 'c'][..]));
        assert_eq!(s.accept_while('x'), None); // Fails, doesn't advance
        assert_eq!(
            s.accept_while(|c: &char| "bcde".contains(*c)),
            Some(&['d', 'e'][..])
        );
        assert_eq!(s.accept_while(vec!['x', 'f'].as_slice()), Some(&['f'][..]));
        assert_eq!(s.accept_while(&['a', 'b', 'c', 'd', 'e', 'f', 'g']), None); // None at EOF
    }

    #[test]
    fn methods_fail_at_eof() {
        let mut s = "a".chars().scanner();
        assert_eq!(s.advance(), Some(&'a'));
        assert_eq!(s.advance(), None);
        assert_eq!(s.accept_while(&['a']), None);
        assert_eq!(s.accept_seq("a".chars()), None);
        assert_eq!(s.peek_seq("a".chars()), None);
    }

    #[test]
    fn accept_sequence() {
        let mut s = "abcdefghi".chars().scanner();
        assert_eq!(s.accept_seq("abc".chars()), Some(&['a', 'b', 'c'][..]));
        assert_eq!(s.accept_seq("dex".chars()), None); // Fails partial match, reverts
        assert_eq!(s.accept_seq("def".chars()), Some(&['d', 'e', 'f'][..]));
        assert_eq!(s.accept_seq("ghi".chars()), Some(&['g', 'h', 'i'][..]));
    }

    #[test]
    fn peek_sequence() {
        let mut s = "abcdef".chars().scanner();
        assert_eq!(s.peek_seq("abc".chars()), Some(&['a', 'b', 'c'][..])); // Matches, doesn't advance
        assert_eq!(s.accept_seq("abc".chars()), Some(&['a', 'b', 'c'][..]));
        assert_eq!(s.peek_seq("def".chars()), Some(&['d', 'e', 'f'][..]));
        assert_eq!(s.peek_seq("dex".chars()), None);
        assert_eq!(s.accept_seq("def".chars()), Some(&['d', 'e', 'f'][..]));
        assert_eq!(s.peek_seq("".chars()), None);
    }

    #[test]
    fn accept_while_until_eof() {
        let mut s = "aaa".chars().scanner();
        // Match all characters until the stream ends
        assert_eq!(s.accept_while('a'), Some(&['a', 'a', 'a'][..]));
        // Next attempt should hit EOF and return None
        assert_eq!(s.accept_while('a'), None);
        assert_eq!(s.current(), None);
    }

    #[test]
    fn accept_while_across_buffer_boundary() {
        let mut s = "abcde".chars().scanner();
        // Force buffer to populate partially
        assert_eq!(s.advance(), Some(&'a'));
        assert_eq!(s.advance(), Some(&'b'));
        // accept_while should read 'c', 'd' from new source, 'e' fails match
        assert_eq!(
            s.accept_while(|c: &char| "cd".contains(*c)),
            Some(&['c', 'd'][..])
        );
        assert_eq!(s.current(), Some(&'d'));
        assert_eq!(s.advance(), Some(&'e'));
    }

    #[test]
    fn accept_while_at_eof() {
        let mut s = "".chars().scanner();
        // Stream is empty, immediate EOF
        assert_eq!(s.accept_while('a'), None);
        assert_eq!(s.current(), None);
    }

    #[test]
    fn accept_while_after_eof() {
        let mut s = "a".chars().scanner();
        assert_eq!(s.advance(), Some(&'a'));
        assert_eq!(s.advance(), None); // Hit EOF
        let cp = s.checkpoint();
        // Calling accept_while after EOF should return None
        assert_eq!(s.accept_while('b'), None);
        // Ensure state wasn't messed up
        assert_eq!(s.current(), None);
        assert_eq!(s.advance(), None);
        // Restore and verify we can't accept_while backwards
        s.restore(cp);
        assert_eq!(s.accept_while('a'), None);
    }

    #[test]
    fn peek_seq_hitting_eof_does_not_advance() {
        let mut s = "a".chars().scanner();
        assert_eq!(s.advance(), Some(&'a'));
        // Peek for a sequence longer than what's remaining.
        // This will hit EOF inside _peek_seq.
        assert_eq!(s.peek_seq("bc".chars()), None);
        // BUG: Since it was just a peek, the current token should still be 'a'.
        // But the current implementation overwrites matched_len with EOF.
        assert_eq!(s.current(), Some(&'a'));
    }

    #[test]
    fn accept_seq_partial_match_hitting_eof_does_not_advance() {
        let mut s = "ab".chars().scanner();
        assert_eq!(s.advance(), Some(&'a'));
        // Try to accept "bc". It will match 'b' (forcing a read),
        // then hit EOF looking for 'c'.
        assert_eq!(s.accept_seq("bc".chars()), None);
        // BUG: accept_seq should cleanly revert on a partial match.
        // It matched 'b' but failed on 'c'. The position should still be 'a'.
        assert_eq!(s.current(), Some(&'a'));
    }

    #[test]
    fn peek_seq_exact_eof() {
        let mut s = "ab".chars().scanner();
        // Peek exact remaining stream.
        assert_eq!(s.peek_seq("ab".chars()), Some(&['a', 'b'][..]));
        // Should not have advanced.
        assert_eq!(s.current(), None);
        assert_eq!(s.advance(), Some(&'a'));
    }

    #[test]
    fn accept_at_eof() {
        let mut s = "a".chars().scanner();
        assert_eq!(s.advance(), Some(&'a'));
        assert_eq!(s.advance(), None);
        assert_eq!(s.accept('b'), None);
        assert_eq!(s.current(), None);
    }

    #[test]
    fn accept_fails_match() {
        let mut s = "ab".chars().scanner();
        assert_eq!(s.advance(), Some(&'a'));
        assert_eq!(s.accept('c'), None);
        assert_eq!(s.current(), Some(&'a'));
    }

    #[test]
    fn restore_at_eof() {
        let mut s = "a".chars().scanner();
        assert_eq!(s.advance(), Some(&'a'));
        assert_eq!(s.advance(), None);
        let cp = s.checkpoint();
        s.restore(cp);
        assert_eq!(s.current(), None);
        assert_eq!(s.advance(), None);
    }

    #[test]
    fn lift_at_eof() {
        let mut s = "a".chars().scanner();
        assert_eq!(s.advance(), Some(&'a'));
        assert_eq!(s.advance(), None);
        assert_eq!(s.lift().collect::<String>(), "a");
        assert_eq!(s.current(), None);
    }

    #[test]
    fn lift_empty() {
        let mut s = "a".chars().scanner();
        assert_eq!(s.advance(), Some(&'a'));
        assert_eq!(s.lift().collect::<String>(), "a");
        assert_eq!(s.lift().collect::<String>(), "");
        assert_eq!(s.lift().collect::<String>(), "");
    }

    #[test]
    fn view_and_view_from() {
        let mut s = "abc".chars().scanner();
        assert_eq!(s.advance(), Some(&'a'));
        let cp = s.checkpoint();
        assert_eq!(s.advance(), Some(&'b'));
        assert_eq!(s.view(), &['a', 'b'][..]);
        assert_eq!(s.view_from(cp), &['b'][..]);
        assert_eq!(s.advance(), Some(&'c'));
        assert_eq!(s.view(), &['a', 'b', 'c'][..]);
        assert_eq!(s.view_from(cp), &['b', 'c'][..]);
    }
}

#![deny(warnings)]

pub struct Scanner<I: Iterator> where I::Item: Clone {
    src: I,
    buf: Vec<I::Item>,
    pos: isize,
}

impl<I> Iterator for Scanner<I> where I: Iterator, I::Item: Clone {
    type Item = I::Item;
    fn next(&mut self) -> Option<Self::Item> {
        self.pos += 1;
        self.prep_buffer();
        let blen = self.buf.len() as isize;
        if self.pos >= blen {
            self.pos = blen;
        }
        self.curr()
    }
}

impl<I> Scanner<I> where I: Iterator, I::Item: Clone {
    pub fn new(source: I) -> Scanner<I> {
        Scanner{src: source, buf: Vec::new(), pos: -1}
    }

    pub fn pos(&self) -> isize { self.pos }

    pub fn set_pos(&mut self, pos: isize) -> bool {
        if pos < -1 || pos > (self.buf.len() as isize) {
            return false;
        }
        self.pos = pos;
        true
    }

    pub fn curr(&self) -> Option<I::Item> {
        let pos = self.pos as usize;
        if self.pos < 0 || pos >= self.buf.len() {
            return None;
        }
        Some(self.buf[pos].clone())
    }

    // try to get enough elements in the buffer for self.pos
    fn prep_buffer(&mut self) {
        while self.pos >= (self.buf.len() as isize) {
            if let Some(tok) = self.src.next() {
                self.buf.push(tok);
            } else {
                break;
            }
        }
    }

    pub fn prev(&mut self) -> Option<I::Item> {
        if self.pos >= 0 { self.pos -= 1; }
        self.curr()
    }

    pub fn peek(&mut self) -> Option<I::Item> {
        let backtrack = self.pos;
        let peeked = self.next();
        self.pos = backtrack;
        peeked
    }

    pub fn peek_prev(&mut self) -> Option<I::Item> {
        let backtrack = self.pos;
        let peeked = self.prev();
        self.pos = backtrack;
        peeked
    }

    pub fn view(&self) -> &[I::Item] {
        let n = (self.pos + 1) as usize;
        &self.buf[..n]
    }

    pub fn ignore(&mut self) {
        if self.pos >= 0 {
            let n = (self.pos + 1) as usize;
            self.buf = if self.buf.len() > n {
                self.buf[n..].to_vec()
            } else {
                Vec::new()
            }
        }
        self.pos = -1;
    }

    pub fn extract(&mut self) -> Vec<I::Item> {
        let tokens = self.view().to_vec();
        self.ignore();
        tokens
    }
}


impl<I> Scanner<I> where I: Iterator, I::Item: Clone + PartialEq {
    pub fn accept(&mut self, what: &I::Item) -> Option<I::Item> {
        let backtrack = self.pos();
        if let Some(next) = self.next() {
            if &next == what { return Some(next); }
        }
        self.set_pos(backtrack);
        None
    }

    // Advance the scanner only if the next char is in the 'any' set,
    // self.curr() will return the matched char if accept matched any
    pub fn accept_any(&mut self, any: &[I::Item]) -> Option<I::Item> {
        let backtrack = self.pos();
        if let Some(next) = self.next() {
            if any.contains(&next) { return Some(next); }
        }
        self.set_pos(backtrack);
        None
    }

    // Skip over the 'over' set, result is if the scanner was advanced,
    // after skip a call to self.curr() will return the last matching char
    pub fn skip_all(&mut self, over: &[I::Item]) -> bool {
        let mut advanced = false;
        while self.accept_any(over).is_some() { advanced = true; }
        advanced
    }

    // Find an element in the 'any' set or EOF, return if the scanner advanced,
    // After until a call to self.curr() returns the last non-matching char
    pub fn until_any(&mut self, any: &[I::Item]) -> bool {
        let mut advanced = false;
        while let Some(next) = self.peek() {
            if any.contains(&next) { break; }
            self.next();
            advanced = true;
        }
        advanced
    }
}

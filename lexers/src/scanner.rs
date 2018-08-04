#![deny(warnings)]

use std::collections::HashSet;
use std::hash::Hash;


pub struct Scanner<T: Clone> {
    src: Option<Box<Iterator<Item=T>>>,
    buf: Vec<T>,
    pos: isize,
}

impl<T: Clone> Iterator for Scanner<T> {
    type Item = T;
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

///////////////////////////////////////////////////////////////////////////////

impl<T: Clone> Scanner<T> {
    pub fn new(source: Box<Iterator<Item=T>>) -> Scanner<T> {
        Scanner{src: Some(source), buf: Vec::new(), pos: -1}
    }

    pub fn from_buf<V: IntoIterator<Item=T>>(source: V) -> Scanner<T> {
        use std::iter::FromIterator;
        Scanner{src: None, buf: Vec::from_iter(source.into_iter()), pos: -1}
    }

    pub fn pos(&self) -> isize { self.pos }

    pub fn set_pos(&mut self, pos: isize) -> bool {
        if pos < -1 || pos > (self.buf.len() as isize) {
            return false;
        }
        self.pos = pos;
        true
    }

    pub fn curr(&self) -> Option<T> {
        let pos = self.pos as usize;
        if self.pos < 0 || pos >= self.buf.len() {
            return None;
        }
        Some(self.buf[pos].clone())
    }

    // try to get enough elements in the buffer for self.pos
    fn prep_buffer(&mut self) {
        if let Some(ref mut nexter) = self.src {
            while self.pos >= (self.buf.len() as isize) {
                if let Some(tok) = nexter.next() {
                    self.buf.push(tok);
                } else {
                    break;
                }
            }
        }
    }

    pub fn prev(&mut self) -> Option<T> {
        if self.pos >= 0 { self.pos -= 1; }
        self.curr()
    }

    pub fn peek(&mut self) -> Option<T> {
        let backtrack = self.pos;
        let peeked = self.next();
        self.pos = backtrack;
        peeked
    }

    pub fn peek_prev(&mut self) -> Option<T> {
        let backtrack = self.pos;
        let peeked = self.prev();
        self.pos = backtrack;
        peeked
    }

    pub fn view(&self) -> &[T] {
        let n = self.pos as usize + 1;
        &self.buf[..n]
    }

    pub fn ignore(&mut self) {
        if self.pos >= 0 {
            let n = self.pos as usize + 1;
            self.buf = if self.buf.len() > n {
                self.buf[n..].to_vec()
            } else {
                Vec::new()
            }
        }
        self.pos = -1;
    }

    pub fn extract(&mut self) -> Vec<T> {
        let tokens = self.view().to_vec();
        self.ignore();
        tokens
    }
}


impl<T: Clone + Hash + Eq> Scanner<T> {
    // Advance the scanner only if the next char is in the 'any' set,
    // self.curr() will return the matched char if accept matched any
    pub fn accept_any(&mut self, any: &HashSet<T>) -> Option<T> {
        let backtrack = self.pos();
        if let Some(next) = self.next() {
            if any.contains(&next) { return Some(next); }
        }
        self.set_pos(backtrack);
        None
    }

    // Skip over the 'over' set, result is if the scanner was advanced,
    // after skip a call to self.curr() will return the last matching char
    pub fn skip_all(&mut self, over: &HashSet<T>) -> bool {
        let mut advanced = false;
        while self.accept_any(over).is_some() { advanced = true; }
        advanced
    }

    // Find an element in the 'any' set or EOF, return if the scanner advanced,
    // After until a call to self.curr() returns the last non-matching char
    pub fn until_any(&mut self, any: &HashSet<T>) -> bool {
        let mut advanced = false;
        while let Some(next) = self.peek() {
            if any.contains(&next) { break; }
            self.next();
            advanced = true;
        }
        advanced
    }
}

static WHITE: &str = " \n\r\t";

impl Scanner<char> {
    pub fn extract_string(&mut self) -> String {
        use std::iter::FromIterator;
        let tokens = String::from_iter(self.view().iter().cloned());
        self.ignore();
        tokens
    }

    pub fn accept_any_char(&mut self, any: &str) -> Option<char> {
        let backtrack = self.pos();
        if let Some(next) = self.next() {
            if any.contains(next) { return Some(next); }
        }
        self.set_pos(backtrack);
        None
    }

    pub fn accept_char(&mut self, c: char) -> bool {
        let backtrack = self.pos();
        if let Some(next) = self.next() {
            if next == c { return true; }
        }
        self.set_pos(backtrack);
        false
    }

    pub fn skip_all_chars(&mut self, over: &str) -> bool {
        let mut advanced = false;
        while self.accept_any_char(over).is_some() { advanced = true; }
        advanced
    }

    pub fn skip_ws(&mut self) -> bool { self.skip_all_chars(WHITE) }

    pub fn ignore_ws(&mut self) {
        self.skip_all_chars(WHITE);
        self.ignore();
    }

    pub fn until_any_char(&mut self, any: &str) -> bool {
        let mut advanced = false;
        while let Some(next) = self.peek() {
            if any.contains(next) { break; }
            self.next();
            advanced = true;
        }
        advanced
    }
}

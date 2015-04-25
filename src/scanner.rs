pub trait Nexter<T> {
    fn get_item(&mut self) -> Option<T>;
}

pub struct Scanner<T: Clone> {
    src: Option<Box<Nexter<T>>>,
    buf: Vec<T>,
    pos: isize,
}

impl<T: Clone> Scanner<T> {
    pub fn new(source: Box<Nexter<T>>) -> Scanner<T> {
        Scanner{src: Some(source), buf: Vec::new(), pos: -1}
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
                if let Some(tok) = nexter.get_item() {
                    self.buf.push(tok);
                } else {
                    break;
                }
            }
        }
    }

    pub fn next(&mut self) -> Option<T> {
        self.pos += 1;
        self.prep_buffer();
        let blen = self.buf.len() as isize;
        if self.pos >= blen {
            self.pos = blen;
        }
        self.curr()
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
            self.buf = self.buf[n..].to_vec();
        }
        self.pos = -1;
    }

    pub fn extract(&mut self) -> Vec<T> {
        let tokens = self.view().to_vec();
        self.ignore();
        tokens
    }
}

use std::collections::HashSet;
use std::hash::Hash;

impl<T: Clone + Hash + Eq> Scanner<T> {
    // Advance the scanner only if the next char is in the 'any' set,
    // self.curr() will return the matched char if accept matched any
    pub fn accept(&mut self, any: &HashSet<T>) -> Option<T> {
        if let Some(next) = self.peek() {
            if any.contains(&next) {
                self.next();
                return Some(next);
            }
        }
        None
    }

    // Skip over the 'over' set, result is if the scanner was advanced,
    // after skip a call to self.curr() will return the last matching char
    pub fn skip(&mut self, over: &HashSet<T>) -> bool {
        let mut advanced = false;
        while self.accept(over).is_some() {
            advanced = true;
        }
        return advanced;
    }

    // Find an element in the 'any' set or EOF, return if the scanner advanced,
    // After until a call to self.curr() returns the last non-matching char
    pub fn until(&mut self, any: &HashSet<T>) -> bool {
        let mut advanced = false;
        while let Some(next) = self.peek() {
            if any.contains(&next) {
                break;
            }
            self.next();
            advanced = true;
        }
        return advanced;
    }
}

static WHITE: &'static str = " \n\r\t";

impl Scanner<char> {
    pub fn from_str(source: &str) -> Scanner<char> {
        Scanner{src: None, buf: source.chars().collect(), pos: -1}
    }

    pub fn extract_string(&mut self) -> String {
        let tokens = self.view().iter().cloned().collect::<String>();
        self.ignore();
        tokens
    }

    pub fn accept_chars(&mut self, any: &str) -> Option<char> {
        let a: HashSet<_> = any.chars().collect();
        self.accept(&a)
    }

    pub fn skip_chars(&mut self, over: &str) -> bool {
        let o: HashSet<_> = over.chars().collect();
        self.skip(&o)
    }

    pub fn skip_ws(&mut self) -> bool {
        self.skip_chars(WHITE)
    }

    pub fn ignore_ws(&mut self) {
        self.skip_chars(WHITE);
        self.ignore();
    }

    pub fn until_chars(&mut self, any: &str) -> bool {
        let a: HashSet<_> = any.chars().collect();
        self.until(&a)
    }
}

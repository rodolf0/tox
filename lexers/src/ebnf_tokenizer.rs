#![deny(warnings)]

use crate::scanner::Scanner;

pub struct EbnfTokenizer<I: Iterator<Item=char>> {
    input: Scanner<I>,
    lookahead: Vec<String>,
}

impl<I: Iterator<Item=char>> EbnfTokenizer<I> {
    pub fn new(source: I) -> Self {
        EbnfTokenizer{input: Scanner::new(source), lookahead: Vec::new()}
    }

    pub fn scanner(source: I) -> Scanner<Self> {
        Scanner::new(Self::new(source))
    }
}

impl<I: Iterator<Item=char>> Iterator for EbnfTokenizer<I> {
    type Item = String;
    fn next(&mut self) -> Option<Self::Item> {
        // used for accumulating string parts
        if !self.lookahead.is_empty() {
            return self.lookahead.pop();
        }
        let s = &mut self.input;
        s.scan_whitespace();
        // discard comments starting with '#' until new-line
        if s.accept(&'#').is_some() {
            while let Some(nl) = s.next() {
                if nl == '\n' {
                    s.extract(); // ignore comment
                    // discard comment and allow more by restarting
                    return self.next();
                }
            }
        }
        if s.accept_any(&['[', ']', '{', '}', '(', ')', '|', ';']).is_some() {
            return Some(s.extract_string());
        }
        let backtrack = s.buffer_pos();
        if s.accept(&':').is_some() {
            if s.accept(&'=').is_some() {
                return Some(s.extract_string());
            }
            s.set_buffer_pos(backtrack);
        }
        let backtrack = s.buffer_pos();
        if let Some(q) = s.accept_any(&['"', '\'']) {
            while let Some(n) = s.next() {
                if n == q {
                    // store closing quote
                    self.lookahead.push(n.to_string());
                    // store string content
                    let v = s.extract_string();
                    self.lookahead.push(v[1..v.len()-1].to_string());
                    // return opening quote
                    return Some(q.to_string());
                }
            }
            s.set_buffer_pos(backtrack);
        }
        let backtrack = s.buffer_pos();
        s.accept(&'@');
        // NOTE: scan_identifier limits the valid options
        if let Some(id) = s.scan_identifier() {
            return Some(id);
        }
        // backtrack possible '@'
        s.set_buffer_pos(backtrack);
        None
    }
}

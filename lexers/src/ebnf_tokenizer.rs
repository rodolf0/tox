#![deny(warnings)]

use crate::helpers;
use crate::scanner::Scanner;


pub struct EbnfTokenizer(Scanner<char>, Vec<String>);

impl EbnfTokenizer {
    pub fn scanner(src: &str) -> Scanner<String> {
        Scanner::new(
            Box::new(
                EbnfTokenizer(Scanner::from_buf(src.chars()), vec!())))
    }
}

impl Iterator for EbnfTokenizer {
    type Item = String;
    fn next(&mut self) -> Option<Self::Item> {
        // used for accumulating string parts
        if !self.1.is_empty() {
            return self.1.pop();
        }
        let mut s = &mut self.0;
        s.ignore_ws();
        // discard comments starting with '#' until new-line
        if s.accept_char('#') {
            while let Some(nl) = s.next() {
                if nl == '\n' {
                    s.ignore();
                    // discard comment and allow more by restarting
                    return self.next();
                }
            }
        }
        if s.accept_any_char("[]{}()|;").is_some() {
            return Some(s.extract_string());
        }
        let backtrack = s.pos();
        if s.accept_any_char(":").is_some() {
            if s.accept_any_char("=").is_some() {
                return Some(s.extract_string());
            }
            s.set_pos(backtrack);
        }
        let backtrack = s.pos();
        if let Some(q) = s.accept_any_char("\"'") {
            while let Some(n) = s.next() {
                if n == q {
                    // store closing quote
                    self.1.push(n.to_string());
                    // store string content
                    let v = s.extract_string();
                    self.1.push(v[1..v.len()-1].to_string());
                    // return opening quote
                    return Some(q.to_string());
                }
            }
            s.set_pos(backtrack);
        }
        // NOTE: scan_identifier limits the valid options
        if let Some(id) = helpers::scan_identifier(&mut s) {
            return Some(id);
        }
        None
    }
}

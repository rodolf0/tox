#![deny(warnings)]

use crate::scanner::Scanner;

static WHITE: &[char] = &[' ', '\n', '\r', '\t'];
static DIGITS: &[char] = &['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
static HEXDIGITS: &[char] = &[
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
    'a', 'b', 'c', 'd', 'e', 'f', 'A', 'B', 'C', 'D', 'E', 'F'];
static ALPHA: &[char] = &['_',
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o',
    'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O',
    'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z'];
static ALNUM: &[char] = &['_',
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o',
    'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O',
    'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z'];


impl<I: Iterator<Item=char>> Scanner<I> {
    pub fn extract_string(&mut self) -> String {
        self.extract().into_iter().collect()
    }

    pub fn scan_whitespace(&mut self) -> Option<String> {
        self.skip_all(WHITE);
        Some(self.extract_string())
    }

    // scan numbers like -?[0-9]+(\.[0-9]+)?([eE][+-][0-9]+)?
    pub fn scan_number(&mut self) -> Option<String> {
        let backtrack = self.buffer_pos();
        // optional sign
        self.accept_any(&['+', '-']);
        // require integer part
        if !self.skip_all(DIGITS) {
            self.set_buffer_pos(backtrack);
            return None;
        }
        // check for fractional part, else it's just an integer
        let backtrack = self.buffer_pos();
        if self.accept(&'.').is_some() && !self.skip_all(DIGITS) {
            self.set_buffer_pos(backtrack);
            return Some(self.extract_string()); // integer
        }
        // check for exponent part
        let backtrack = self.buffer_pos();
        if self.accept_any(&['e', 'E']).is_some() {
            self.accept_any(&['+', '-']); // exponent sign is optional
            if !self.skip_all(DIGITS) {
                self.set_buffer_pos(backtrack);
                return Some(self.extract_string()); //float
            }
        }
        self.accept(&'i'); // accept imaginary numbers
        Some(self.extract_string())
    }

    pub fn scan_math_op(&mut self) -> Option<String> {
        const OPS: &[char] = &['+', '-', '*', '/', '%', '^', '!', '(', ')', ','];
        if self.accept_any(&['>', '=', '<']).is_some() {
            // accept '<', '>', '=', '<=', '>=', '=='
            self.accept(&'=');
            Some(self.extract_string())
        } else if self.accept(&'*').is_some() {
            // accept '*', '**'
            self.accept(&'*');
            Some(self.extract_string())
        } else if self.accept_any(OPS).is_some() {
            Some(self.extract_string())
        } else {
            None
        }
    }

    // scan integers like 0x34 0b10101 0o657
    pub fn scan_integer(&mut self) -> Option<String> {
        let backtrack = self.buffer_pos();
        if self.accept(&'0').is_some() &&
            match self.accept_any(&['x', 'o', 'b']) {
                Some('x') => self.skip_all(HEXDIGITS),
                Some('o') => self.skip_all(&HEXDIGITS[..8]),
                Some('b') => self.skip_all(&HEXDIGITS[..2]),
                _ => false,
            } {
            return Some(self.extract_string());
        }
        self.set_buffer_pos(backtrack);
        None
    }

    // scan a quoted string like "this is \"an\" example"
    pub fn scan_quoted_string(&mut self, q: char) -> Option<String> {
        let backtrack = self.buffer_pos();
        self.accept(&q)?;
        while let Some(n) = self.next() {
            if n == '\\' { self.next(); continue; }
            if n == q { return Some(self.extract_string()); }
        }
        self.set_buffer_pos(backtrack);
        None
    }

    // scan [a-zA-Z_][a-zA-Z0-9_]+
    pub fn scan_identifier(&mut self) -> Option<String> {
        self.accept_any(ALPHA)?;
        self.skip_all(ALNUM);
        Some(self.extract_string())
    }
}

///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_number() {
        let tests = vec![
            "987", "-543", "435i",
            "41.98", "-83.5", "-54.3i",
            "28e3", "54E+2", "54e-33", "43e0i", "3E8i",
            "-38e3", "-53e+5", "-65E-4", "-32E-4i", "-33e+2i",
            "85.365e3", "54.234E+2", "54.849e-33", "1.4e+2i", "3.14e-5i",
            "-38.657e3", "53.845e+5", "65.987E-4", "-4.4e+2i", "-6.14e-5i",
        ];
        for t in tests.iter() {
            let result = Scanner::new(t.chars()).scan_number();
            assert_eq!(Some(t.to_string()), result);
        }
    }

    #[test]
    fn scan_math_ops() {
        let tests = vec![
            "<", "<=", "==", ">=", ">", "(", ")", ",",
            "*", "**", "^", "!", "+", "-", "/", "%",
        ];
        for t in tests.iter() {
            let result = Scanner::new(t.chars()).scan_math_op();
            assert_eq!(Some(t.to_string()), result);
        }
    }

    #[test]
    fn scan_identifiers() {
        let tests = vec!["id1", "func", "anyword", "_00", "bla23"];
        for t in tests.iter() {
            let result = Scanner::new(t.chars()).scan_identifier();
            assert_eq!(Some(t.to_string()), result);
        }
    }

    #[test]
    fn scan_string() {
        let tests = vec![
            r"'this is a test'",
            r"'another test \' with an escaped quote'",
        ];
        for t in tests.iter() {
            let result = Scanner::new(t.chars()).scan_quoted_string('\'');
            assert_eq!(Some(t.to_string()), result);
        }
    }
}

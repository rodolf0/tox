use std::ops::{Deref, DerefMut};
use scanner::{Scanner, Nexter};

pub struct Tokenizer {
    source: Scanner<char>,
    delims: String,
}

impl Nexter<String> for Tokenizer {
    fn get_item(&mut self) -> Option<String> {
        self.source.ignore_ws();
        if self.source.until_chars(&self.delims) {
            return Some(self.source.extract_string());
        } else if let Some(c) = self.source.accept_chars(&self.delims) {
            self.source.ignore();
            return Some(c.to_string());
        }
        None
    }
}

// lexer splits on delims and ignores whitespace
pub struct Lexer(Scanner<String>);

impl Lexer {
    pub fn from_str<S: Into<String>>(input: &str, delims: S) -> Lexer {
        Lexer(Scanner::new(Box::new(Tokenizer{
            source: Scanner::from_str(input),
            delims: delims.into(),
        })))
    }
}

impl Deref for Lexer {
    type Target = Scanner<String>;
    fn deref<'a>(&'a self) -> &'a Scanner<String> { &self.0 }
}

impl DerefMut for Lexer {
    fn deref_mut<'a>(&'a mut self) -> &'a mut Scanner<String> { &mut self.0 }
}


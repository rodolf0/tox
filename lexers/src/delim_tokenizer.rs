use scanner::{Nexter, Scanner};

// A tokenizer that splits input on each delimiter
// The delimiters are not removed, space around tokens is trimmed
pub struct DelimTokenizer {
    s: Scanner<char>,
    delims: String,
}

impl DelimTokenizer {
    pub fn from_str<S: Into<String>>(src: &str, delims: S) -> Scanner<String> {
        Scanner::new(Box::new(
            DelimTokenizer{s: Scanner::from_str(src), delims: delims.into()}
        ))
    }
}

impl Nexter<String> for DelimTokenizer {
    fn get_item(&mut self) -> Option<String> {
        self.s.ignore_ws();
        if self.s.until_chars(&self.delims) {
            return Some(self.s.extract_string());
        } else if let Some(c) = self.s.accept_chars(&self.delims) {
            self.s.ignore();
            return Some(c.to_string());
        }
        None
    }
}

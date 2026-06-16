use crate::extractors::{quoted, quoted_no_delims};
use crate::string_tokenizer::StringTokenizer;

pub struct EbnfTokenizer<'a, I: Iterator<Item = char>>(StringTokenizer<'a, I>);

impl<'a> From<&'a str> for EbnfTokenizer<'a, std::str::Chars<'a>> {
    fn from(s: &'a str) -> Self {
        Self::new(s.chars())
    }
}

impl<'a, I: Iterator<Item = char>> EbnfTokenizer<'a, I> {
    pub fn new(source: I) -> Self {
        let tokenizer = StringTokenizer::new(source)
            .split_on(["[", "]", "{", "}", "(", ")", "|", ";", ":=", ":"], false)
            .split_by(|s| quoted(s, "\"", "\"", Some('\\')), false)
            .split_by(|s| quoted(s, "'", "'", Some('\\')), false)
            .split_by(|s| quoted_no_delims(s, "#", "\n", Some('\\')), true);
        EbnfTokenizer(tokenizer)
    }
}

impl<'a, I: Iterator<Item = char>> Iterator for EbnfTokenizer<'a, I> {
    type Item = String;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

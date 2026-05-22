use crate::string_tokenizer::{EscapePair, StringTokenizer};

pub struct EbnfTokenizer<I: Iterator<Item = char>>(StringTokenizer<I>);

impl<'a> From<&'a str> for EbnfTokenizer<std::str::Chars<'a>> {
    fn from(s: &'a str) -> Self {
        Self::new(s.chars())
    }
}

impl<I: Iterator<Item = char>> EbnfTokenizer<I> {
    pub fn new(source: I) -> Self {
        let tokenizer = StringTokenizer::new(source)
            .symbols(["[", "]", "{", "}", "(", ")", "|", ";", ":=", ":"])
            .escape_pairs([
                EscapePair::new("'", "'"),
                EscapePair::new("\"", "\""),
                // We keep the comment string in the Tokenizer, but we will skip it below in `next()`
                EscapePair::new("#", "\n").drop_delimiters(),
            ]);
        EbnfTokenizer(tokenizer)
    }
}

impl<I: Iterator<Item = char>> Iterator for EbnfTokenizer<I> {
    type Item = String;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let token = self.0.next()?;
            if token.starts_with('#') {
                continue; // drop comments
            }
            return Some(token);
        }
    }
}

#![deny(warnings)]

use crate::scanner::Scanner;


// A tokenizer that splits input on each delimiter
pub struct DelimTokenizer<I: Iterator<Item=char>> {
    src: Scanner<I>,
    delims: String,
    remove: bool, // drop the delimiters ?
}

impl<I: Iterator<Item=char>> DelimTokenizer<I> {
    pub fn new<S>(src: I, delims: S, remove: bool) -> Self
        where S: Into<String>
    {
        DelimTokenizer{src: Scanner::new(src), delims: delims.into(), remove}
    }
}

impl<I: Iterator<Item=char>> Iterator for DelimTokenizer<I> {
    type Item = String;
    fn next(&mut self) -> Option<Self::Item> {
        if self.src.until_any_char(&self.delims) {
            Some(self.src.extract_string())
        } else if let Some(c) = self.src.accept_any_char(&self.delims) {
            self.src.ignore();
            if self.remove { self.next() } else { Some(c.to_string()) }
        } else {
            None
        }
    }
}

///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::DelimTokenizer;

    #[test]
    fn delim_tokenizer() {
        let inputs = vec![
            ("this  is a   test ", " ", true),
            ("just,more,tests,hi", ",", true),
            ("another, test, here,going on", " ,", true),
            ("1+2*3/5", "/+*", false),
        ];
        let expect = vec![
            vec!["this", "is", "a", "test"],
            vec!["just", "more", "tests", "hi"],
            vec!["another", "test", "here", "going", "on"],
            vec!["1", "+", "2", "*", "3", "/", "5"],
        ];
        for (input, expected) in inputs.iter().zip(expect.iter()) {
            let mut lx = DelimTokenizer::new(input.0.chars(), input.1, input.2);
            for exp in expected.iter() { assert_eq!(*exp, lx.next().unwrap()); }
            assert_eq!(lx.next(), None);
        }
    }
}

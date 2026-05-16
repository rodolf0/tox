
use crate::scanner::Scanner;

// A tokenizer that splits input on each delimiter
pub struct DelimTokenizer<I: Iterator<Item = char>> {
    src: Scanner<I>,
    delims: Vec<char>,
    remove: bool, // drop the delimiters ?
}

impl<'a> DelimTokenizer<std::str::Chars<'a>> {
    pub fn from_str(s: &'a str, delims: &str, remove: bool) -> Self {
        Self::new(s.chars(), delims, remove)
    }
}

impl<I: Iterator<Item = char>> DelimTokenizer<I> {
    pub fn new(src: I, delims: &str, remove: bool) -> Self {
        DelimTokenizer {
            src: Scanner::new(src),
            delims: delims.chars().collect(),
            remove,
        }
    }
}

impl<I: Iterator<Item = char>> Iterator for DelimTokenizer<I> {
    type Item = String;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.src.until(&self.delims[..]) {
                return Some(self.src.extract_string());
            } else {
                let c = self.src.accept(&self.delims[..])?;
                self.src.extract(); // commit delimiter, reset buffer
                if !self.remove {
                    return Some(c.to_string());
                }
                // If we remove, just continue the loop
            }
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
            let mut lx = DelimTokenizer::new(input.0.chars(), &input.1, input.2);
            for exp in expected.iter() {
                assert_eq!(*exp, lx.next().unwrap());
            }
            assert_eq!(lx.next(), None);
        }
    }
}

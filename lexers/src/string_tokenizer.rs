use crate::scanner::{Scan, Scanner};

#[derive(Clone)]
struct Splitter {
    text: String,
    is_delimiter: bool, // delimiters are dropped, symbols returned.
}

pub struct StringTokenizer<I: Iterator<Item = char>> {
    src: Scanner<I>,
    splitters: Vec<Splitter>,
    // Configures what chars are discarded while scanning. Default: whitespace.
    trimmer: fn(&char) -> bool,
}

impl<'a> From<&'a str> for StringTokenizer<std::str::Chars<'a>> {
    fn from(s: &'a str) -> Self {
        Self::new(s.chars())
    }
}

impl<I: Iterator<Item = char>> StringTokenizer<I> {
    pub fn new(source: I) -> Self {
        StringTokenizer {
            src: source.scanner(),
            splitters: Vec::new(),
            trimmer: |c: &char| c.is_whitespace(),
        }
    }

    pub fn symbols(mut self, syms: impl IntoIterator<Item = impl AsRef<str>>) -> Self {
        for s in syms {
            self.splitters.push(Splitter {
                text: s.as_ref().to_string(),
                is_delimiter: false,
            });
        }
        self.splitters
            .sort_by_key(|b| std::cmp::Reverse(b.text.len()));
        self
    }

    pub fn delimiters(mut self, delims: impl IntoIterator<Item = impl AsRef<str>>) -> Self {
        for s in delims {
            self.splitters.push(Splitter {
                text: s.as_ref().to_string(),
                is_delimiter: true,
            });
        }
        self.splitters
            .sort_by_key(|b| std::cmp::Reverse(b.text.len()));
        self
    }

    pub fn trimmer(mut self, trimmer: fn(&char) -> bool) -> Self {
        self.trimmer = trimmer;
        self
    }
}

impl<I: Iterator<Item = char>> Iterator for StringTokenizer<I> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // Skip any chars we don't care about (eg: whitespace)
            if self.src.accept_while(self.trimmer).len() > 0 {
                let _ = self.src.lift(); // drop it
            }
            // Check if there's a splitter to return at current position.
            if let Some(splitter) = self
                .splitters
                .iter()
                .find(|s| self.src.accept_seq(s.text.chars()))
            {
                // It's a delimiter, skip and look for the next token
                if splitter.is_delimiter {
                    let _ = self.src.lift(); // drop it
                    continue;
                } else {
                    return Some(self.src.lift().collect());
                }
            }
            // Advance as long as it's not a trimmer char AND we haven't hit a splitter.
            let mut token_len = 0;
            while let Some(not_eof_char) = self.src.peek() {
                // loop til EOF
                if (self.trimmer)(not_eof_char) {
                    break; // hit a trimmer
                }
                if self
                    .splitters
                    .iter()
                    .any(|s| self.src.peek_seq(s.text.chars()))
                {
                    break; // hit a splitter
                }
                token_len += 1;
                self.src.advance();
            }
            if token_len > 0 {
                return Some(self.src.lift().collect());
            }
            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::StringTokenizer;

    #[test]
    fn split_on_trimmer() {
        // multiple whitespace
        let r = StringTokenizer::from("this  is a   test ");
        assert_eq!(r.collect::<Vec<_>>(), ["this", "is", "a", "test"]);
        // different whitespace
        let r = StringTokenizer::from("\tthis is  a\ntest");
        assert_eq!(r.collect::<Vec<_>>(), ["this", "is", "a", "test"]);
        // user-specified trimer
        let r = StringTokenizer::from(":this is: a test").trimmer(|c| *c == ':');
        assert_eq!(r.collect::<Vec<_>>(), ["this is", " a test"]);
    }

    #[test]
    fn split_on_delimiters() {
        // just delimiters
        let r = StringTokenizer::from("just,more+tests").delimiters([",", "+"]);
        assert_eq!(r.collect::<Vec<_>>(), ["just", "more", "tests"]);
        // multiple delimiters
        let r = StringTokenizer::from("+just,more+tests++hi").delimiters(["+"]);
        assert_eq!(r.collect::<Vec<_>>(), ["just,more", "tests", "hi"]);
        // delimiters and trimmers (whitespace)
        let r = StringTokenizer::from("just,more tests").delimiters([","]);
        assert_eq!(r.collect::<Vec<_>>(), ["just", "more", "tests"]);
    }

    #[test]
    fn split_on_symbols() {
        // single-char symbols
        let lx = StringTokenizer::from("1+2*3/5").symbols(["/", "+", "*"]);
        assert_eq!(lx.collect::<Vec<_>>(), ["1", "+", "2", "*", "3", "/", "5"]);
        // mixed-length symbols
        let lx = StringTokenizer::from("a:=3 b:4 c=5").symbols([":=", ":"]);
        assert_eq!(
            lx.collect::<Vec<_>>(),
            ["a", ":=", "3", "b", ":", "4", "c=5"]
        );
        // symbols and delimiters
        let lx = StringTokenizer::from("a:=3,b:4")
            .symbols([":=", ":"])
            .delimiters([","]);
        assert_eq!(lx.collect::<Vec<_>>(), ["a", ":=", "3", "b", ":", "4"]);
    }
}

use crate::scanner::{Scan, Scanner};

#[derive(Clone)]
pub struct EscapePair {
    start: String,
    end: String,
    escape_char: Option<char>,
    keep_delimiters: bool,
}

impl EscapePair {
    pub fn new(start: impl AsRef<str>, end: impl AsRef<str>) -> Self {
        Self {
            start: start.as_ref().to_string(),
            end: end.as_ref().to_string(),
            escape_char: Some('\\'),
            keep_delimiters: true,
        }
    }

    pub fn escape_char(mut self, escape_char: Option<char>) -> Self {
        self.escape_char = escape_char;
        self
    }

    pub fn drop_delimiters(mut self) -> Self {
        self.keep_delimiters = false;
        self
    }
}

#[derive(Clone)]
struct Splitter {
    text: String,
    is_delimiter: bool, // delimiters are dropped, symbols returned.
}

pub struct StringTokenizer<I: Iterator<Item = char>> {
    src: Scanner<I>,
    splitters: Vec<Splitter>,
    escape_pairs: Vec<EscapePair>,
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
            escape_pairs: Vec::new(),
            trimmer: |c: &char| c.is_whitespace(),
        }
    }

    pub fn escape_pairs(mut self, pairs: impl IntoIterator<Item = EscapePair>) -> Self {
        for p in pairs {
            self.escape_pairs.push(p);
        }
        self.escape_pairs
            .sort_by_key(|p| std::cmp::Reverse(p.start.len()));
        self
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
            if !self.src.accept_while(self.trimmer).is_empty() {
                let _ = self.src.lift(); // drop it
            }

            // Check if we hit an escape pair region
            if let Some(pair) = self
                .escape_pairs
                .iter()
                .find(|p| self.src.accept_seq(p.start.chars()))
            {
                // consume all escaped content until end delimiter.
                while let Some(c) = self.src.peek() {
                    if Some(*c) == pair.escape_char {
                        self.src.advance(); // consume escape char
                        self.src.advance(); // consume escaped char
                        continue;
                    }
                    if self.src.accept_seq(pair.end.chars()) {
                        break;
                    }
                    self.src.advance();
                }
                // strip delimiters if needed
                let result: String = self.src.lift().collect();
                if !pair.keep_delimiters {
                    let mut s = result.as_str();
                    s = s.strip_prefix(&pair.start).unwrap_or(s);
                    s = s.strip_suffix(&pair.end).unwrap_or(s);
                    return Some(s.to_string());
                }
                return Some(result);
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
    use super::{EscapePair, StringTokenizer};

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

    #[test]
    fn escape_pairs() {
        // Keep quotes
        let lx = StringTokenizer::from(r#"hello "escaped \" string" world"#)
            .escape_pairs([EscapePair::new("\"", "\"")]);
        assert_eq!(
            lx.collect::<Vec<_>>(),
            ["hello", r#""escaped \" string""#, "world"]
        );

        // Drop quotes
        let lx = StringTokenizer::from(r#"hello "escaped \" string" world"#)
            .escape_pairs([EscapePair::new("\"", "\"").drop_delimiters()]);
        assert_eq!(
            lx.collect::<Vec<_>>(),
            ["hello", r#"escaped \" string"#, "world"]
        );

        // Custom block comment with no escape char, dropping the bounds
        let lx = StringTokenizer::from("code /* some \n comment */ more code").escape_pairs([
            EscapePair::new("/*", "*/")
                .escape_char(None)
                .drop_delimiters(),
        ]);
        assert_eq!(
            lx.collect::<Vec<_>>(),
            ["code", " some \n comment ", "more", "code"]
        );

        // Unterminated quote
        let lx = StringTokenizer::from(r#"start "unterminated"#)
            .escape_pairs([EscapePair::new("\"", "\"")]);
        assert_eq!(lx.collect::<Vec<_>>(), ["start", r#""unterminated"#]);
    }
}

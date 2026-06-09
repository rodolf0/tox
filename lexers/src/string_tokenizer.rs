use crate::scanner::{Scan, Scanner};

trait Extractor2<I: Iterator<Item = char>> {
    fn extract<'a>(&self, scanner: &'a mut Scanner<I>) -> Option<&'a [char]>;
}

impl<I: Iterator<Item = char>, const N: usize> Extractor2<I> for [&str; N] {
    fn extract<'a>(&self, scanner: &'a mut Scanner<I>) -> Option<&'a [char]> {
        for splitter in *self {
            if scanner.accept_seq(splitter.chars()) {
                return Some(scanner.view());
            }
        }
        None
    }
}

impl<I: Iterator<Item = char>, Func> Extractor2<I> for Func
where
    Func: Fn(&mut Scanner<I>) -> Option<&[char]>,
{
    fn extract<'a>(&self, scanner: &'a mut Scanner<I>) -> Option<&'a [char]> {
        self(scanner)
    }
}

pub fn symbols<I: Iterator<Item = char>>(
    syms: impl IntoIterator<Item = impl AsRef<str>>,
) -> impl Fn(&mut Scanner<I>) -> Option<&[char]> {
    let syms = syms
        .into_iter()
        .map(|s| s.as_ref().to_string())
        .collect::<Vec<_>>();
    move |scanner: &mut Scanner<I>| {
        for splitter in &syms {
            if scanner.accept_seq(splitter.chars()) {
                return Some(scanner.view());
            }
        }
        None
    }
}

pub fn quoted<I: Iterator<Item = char>>(
    start: &str,
    end: &str,
    escape: Option<char>,
) -> impl Fn(&mut Scanner<I>) -> Option<&[char]> {
    move |scanner: &mut Scanner<I>| {
        if extract_quoted(scanner, start, end, escape).is_some() {
            return Some(scanner.view());
        }
        None
    }
}

// TODO: move this to extractors.rs
pub fn extract_quoted<'a, I: Iterator<Item = char>>(
    s: &'a mut Scanner<I>,
    start: &str,
    end: &str,
    escape: Option<char>,
) -> Option<&'a [char]> {
    let cp = s.checkpoint();
    if s.accept_seq(start.chars()) {
        // consume all escaped content until end delimiter.
        while let Some(c) = s.peek() {
            if Some(*c) == escape {
                s.advance(); // consume escape char
            } else if s.accept_seq(end.chars()) {
                return Some(s.view());
            }
            s.advance();
        }
    }
    s.restore(cp);
    None
    //
    // // strip delimiters if needed
    // let result: String = self.src.lift().collect();
    // if !pair.keep_delimiters {
    //     let mut s = result.as_str();
    //     s = s.strip_prefix(&pair.start).unwrap_or(s);
    //     s = s.strip_suffix(&pair.end).unwrap_or(s);
    //     return Some(s.to_string());
    // }
    // return Some(result);
}

struct Extractor<'a, I: Iterator<Item = char>> {
    extractor: Box<dyn Fn(&mut Scanner<I>) -> Option<&[char]> + 'a>,
    discard: bool, // should this match be discarded or returned.
}

// pub struct StringTokenizer<I: Iterator<Item = char>> {
pub struct StringTokenizer<'a, I: Iterator<Item = char>> {
    src: Scanner<I>,
    // Configures what chars are discarded while scanning. Default: whitespace.
    trimmer: fn(&char) -> bool,
    // Extractors act like complex splitters.
    // They can match a splitter by using the scanner.
    // On no match they need to leave the scanner in its original state.
    // escape pairs are a form of extractor.
    extractors: Vec<Extractor<'a, I>>,
    extractors2: Vec<(Box<dyn Extractor2<I> + 'a>, bool)>,
    queued_token: Option<String>,
}

// impl<'a> From<&'a str> for StringTokenizer<std::str::Chars<'a>> {
impl<'a> From<&'a str> for StringTokenizer<'a, std::str::Chars<'a>> {
    fn from(s: &'a str) -> Self {
        Self::new(s.chars())
    }
}

// impl<I: Iterator<Item = char>> StringTokenizer<I> {
impl<'a, I: Iterator<Item = char>> StringTokenizer<'a, I> {
    pub fn new(source: I) -> Self {
        StringTokenizer {
            src: source.scanner(),
            trimmer: |c: &char| c.is_whitespace(),
            extractors: Vec::new(),
            extractors2: Vec::new(),
            queued_token: None,
        }
    }

    pub fn split_on<S>(mut self, s: S, discard: bool) -> Self
    where
        S: Fn(&mut Scanner<I>) -> Option<&[char]> + 'a,
    {
        self.extractors.push(Extractor {
            extractor: Box::new(s),
            discard,
        });
        self
    }

    pub fn split_on2(mut self, s: impl Extractor2<I> + 'a, discard: bool) -> Self {
        self.extractors2.push((Box::new(s), discard));
        self
    }

    pub fn extractor<E>(mut self, e: E, discard: bool) -> Self
    where
        E: Fn(&mut Scanner<I>) -> Option<&[char]> + 'a,
    {
        self.extractors.push(Extractor {
            extractor: Box::new(e),
            discard,
        });
        self
    }

    pub fn delimiters(mut self, delims: impl IntoIterator<Item = impl AsRef<str> + 'a>) -> Self {
        for splitter in delims {
            self.extractors.push(Extractor {
                extractor: Box::new(move |s| {
                    s.accept_seq(splitter.as_ref().chars()).then_some(s.view())
                }),
                discard: true,
            });
        }
        self
    }

    pub fn trimmer(mut self, trimmer: fn(&char) -> bool) -> Self {
        self.trimmer = trimmer;
        self
    }
}

impl<'a, I: Iterator<Item = char>> Iterator for StringTokenizer<'a, I> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(token) = self.queued_token.take() {
                return Some(token);
            }

            let sofar = self.src.matched_len();

            // Skip any chars we don't care about (eg: whitespace)
            if !self.src.accept_while(self.trimmer).is_empty() {
                if sofar > 0 {
                    let ret = Some(self.src.lift_first(sofar).collect());
                    let _ = self.src.lift(); // drop the trimmer chars
                    return ret;
                }
                let _ = self.src.lift(); // drop the trimmer chars
            }

            // Check extractors (TODO: consolidate with splitters / escape pairs)
            if let Some(extractor) = self
                .extractors
                .iter_mut()
                .find(|e| (e.extractor)(&mut self.src).is_some())
            {
                if sofar > 0 {
                    let ret = Some(self.src.lift_first(sofar).collect());
                    let next = self.src.lift();
                    if !extractor.discard {
                        self.queued_token = Some(next.collect());
                    }
                    return ret;
                }

                let ret = self.src.lift();
                if !extractor.discard {
                    return Some(ret.collect());
                }
            }

            if let Some((_extractor, discard)) = self
                .extractors2
                .iter_mut()
                .find(|e| e.0.extract(&mut self.src).is_some())
            {
                if sofar > 0 {
                    let ret = Some(self.src.lift_first(sofar).collect());
                    let next = self.src.lift();
                    if !*discard {
                        self.queued_token = Some(next.collect());
                    }
                    return ret;
                }

                let ret = self.src.lift();
                if !*discard {
                    return Some(ret.collect());
                }
            }

            if self.src.advance().is_none() {
                if sofar > 0 {
                    return Some(self.src.lift_first(sofar).collect());
                }
                return None; // EOF
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::StringTokenizer;
    use crate::string_tokenizer::*;

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
        // let lx = StringTokenizer::from("1+2*3/5").symbols(["/", "+", "*"]);
        let lx = StringTokenizer::from("1+2*3/5").split_on(symbols(["/", "+", "*"]), false);
        assert_eq!(lx.collect::<Vec<_>>(), ["1", "+", "2", "*", "3", "/", "5"]);

        let lx = StringTokenizer::from("1+2*3/5").split_on2(["/", "+", "*"], false);
        assert_eq!(lx.collect::<Vec<_>>(), ["1", "+", "2", "*", "3", "/", "5"]);

        // mixed-length symbols
        // let lx = StringTokenizer::from("a:=3 b:4 c=5").symbols([":=", ":"]);
        let lx = StringTokenizer::from("a:=3 b:4 c=5").split_on(symbols([":=", ":"]), false);
        assert_eq!(
            lx.collect::<Vec<_>>(),
            ["a", ":=", "3", "b", ":", "4", "c=5"]
        );
        // symbols and delimiters
        let lx = StringTokenizer::from("a:=3,b:4")
            .split_on(symbols([":=", ":"]), false)
            .delimiters([","]);
        assert_eq!(lx.collect::<Vec<_>>(), ["a", ":=", "3", "b", ":", "4"]);

        let lx = StringTokenizer::from(",b:4")
            .split_on(symbols([":=", ":"]), false)
            .delimiters([","]);
        assert_eq!(lx.collect::<Vec<_>>(), ["b", ":", "4"]);
    }

    #[test]
    fn escape_pairs() {
        // Keep quotes
        let lx = StringTokenizer::from(r#"hello "escaped \" string" world"#)
            .split_on(quoted("\"", "\"", Some('\\')), false);
        assert_eq!(
            lx.collect::<Vec<_>>(),
            ["hello", r#""escaped \" string""#, "world"]
        );

        let lx = StringTokenizer::from(r#"hello "escaped \" string" world"#)
            .split_on(|s| extract_quoted(s, "\"", "\"", Some('\\')), false);
        assert_eq!(
            lx.collect::<Vec<_>>(),
            ["hello", r#""escaped \" string""#, "world"]
        );

        // Drop quotes
        // let lx = StringTokenizer::from(r#"hello "escaped \" string" world"#)
        //     .escape_pairs([EscapePair::new("\"", "\"").drop_delimiters()]);
        // assert_eq!(
        //     lx.collect::<Vec<_>>(),
        //     ["hello", r#"escaped \" string"#, "world"]
        // );
        //
        // // Custom block comment with no escape char, dropping the bounds
        // let lx = StringTokenizer::from("code /* some \n comment */ more code").escape_pairs([
        //     EscapePair::new("/*", "*/")
        //         .escape_char(None)
        //         .drop_delimiters(),
        // ]);
        // assert_eq!(
        //     lx.collect::<Vec<_>>(),
        //     ["code", " some \n comment ", "more", "code"]
        // );
        //
        // // Unterminated quote
        // let lx = StringTokenizer::from(r#"start "unterminated"#)
        //     .escape_pairs([EscapePair::new("\"", "\"")]);
        // assert_eq!(lx.collect::<Vec<_>>(), ["start", r#""unterminated"#]);
    }
}

use crate::scanner::{Scan, Scanner};

pub trait Extractor<I: Iterator<Item = char>> {
    fn extract<'a>(&self, scanner: &'a mut Scanner<I>) -> Option<&'a [char]>;
}

impl<I, F> Extractor<I> for F
where
    I: Iterator<Item = char>,
    F: Fn(&mut Scanner<I>) -> Option<&[char]>,
{
    fn extract<'a>(&self, scanner: &'a mut Scanner<I>) -> Option<&'a [char]> {
        self(scanner)
    }
}

impl<I: Iterator<Item = char>, const N: usize> Extractor<I> for [&str; N] {
    fn extract<'a>(&self, scanner: &'a mut Scanner<I>) -> Option<&'a [char]> {
        let cp = scanner.checkpoint();
        for splitter in *self {
            if scanner.accept_seq(splitter.chars()).is_some() {
                return Some(scanner.view_from(cp));
            }
        }
        None
    }
}

impl<I: Iterator<Item = char>> Extractor<I> for &str {
    fn extract<'a>(&self, scanner: &'a mut Scanner<I>) -> Option<&'a [char]> {
        scanner.accept_seq(self.chars())
    }
}

struct ExtractorConfig<'a, I: Iterator<Item = char>> {
    e: Box<dyn Extractor<I> + 'a>,
    discard: bool,
}

pub struct StringTokenizer<'a, I: Iterator<Item = char>> {
    src: Scanner<I>,
    // Configures what chars are discarded while scanning. Default: whitespace.
    trimmer: fn(&char) -> bool,
    extractors: Vec<ExtractorConfig<'a, I>>,
    queued_token: Option<String>,
}

impl<'a> From<&'a str> for StringTokenizer<'a, std::str::Chars<'a>> {
    fn from(s: &'a str) -> Self {
        Self::new(s.chars())
    }
}

impl<'a, I: Iterator<Item = char>> StringTokenizer<'a, I> {
    pub fn new(source: I) -> Self {
        StringTokenizer {
            src: source.scanner(),
            trimmer: |c: &char| c.is_whitespace(),
            extractors: Vec::new(),
            queued_token: None,
        }
    }

    pub fn split_by(
        self,
        f: impl Fn(&mut Scanner<I>) -> Option<&[char]> + 'a,
        discard: bool,
    ) -> Self {
        self.split_on(f, discard)
    }

    pub fn split_on(mut self, s: impl Extractor<I> + 'a, discard: bool) -> Self {
        self.extractors.push(ExtractorConfig {
            e: Box::new(s),
            discard,
        });
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
        'outer: loop {
            if let Some(token) = self.queued_token.take() {
                return Some(token);
            }
            let pre_matched = self.src.matched_len();

            // Skip any chars we don't care about (eg: whitespace)
            if self.src.accept_while(self.trimmer).is_some() {
                let matched = self.src.lift();
                if pre_matched > 0 {
                    return Some(matched.take(pre_matched).collect());
                }
            }

            for ExtractorConfig { e, discard } in &self.extractors {
                if let Some(extract) = e.extract(&mut self.src) {
                    let queued = if !discard {
                        Some(extract.into_iter().collect())
                    } else {
                        None
                    };
                    // consume the scanner's buffer (pre-matched and cur match)
                    let matched = self.src.lift();
                    // Return the pre-existent match and queue the just extracted.
                    if pre_matched > 0 {
                        self.queued_token = queued;
                        return Some(matched.take(pre_matched).collect());
                    }
                    // non-discard extractor matched
                    if let Some(token) = queued {
                        return Some(token);
                    }
                    // extractor matched, discarded so nothing to return
                    // (nor queued). Restart the loop to avoid eating a char below.
                    continue 'outer;
                }
            }

            if self.src.advance().is_none() {
                if pre_matched == 0 {
                    return None; // EOF
                }
                // retrun any remaining content (pre-matched)
                return Some(self.src.lift().collect());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::StringTokenizer;
    use crate::extractors::{quoted, quoted_no_delims};

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
        let r = StringTokenizer::from("just,more+tests").split_on([",", "+"], true);
        assert_eq!(r.collect::<Vec<_>>(), ["just", "more", "tests"]);
        // multiple delimiters
        let r = StringTokenizer::from("+just,more+tests++hi").split_on("+", true);
        assert_eq!(r.collect::<Vec<_>>(), ["just,more", "tests", "hi"]);
        // delimiters and trimmers (whitespace)
        let r = StringTokenizer::from("just,more tests").split_on(",", true);
        assert_eq!(r.collect::<Vec<_>>(), ["just", "more", "tests"]);
        // Check no tripping on multiple delimiters
        let r = StringTokenizer::from(",,just").split_on(",", true);
        assert_eq!(r.collect::<Vec<_>>(), ["just"]);
    }

    #[test]
    fn split_on_symbols() {
        // single-char symbols
        let lx = StringTokenizer::from("1+2*3/5").split_on(["/", "+", "*"], false);
        assert_eq!(lx.collect::<Vec<_>>(), ["1", "+", "2", "*", "3", "/", "5"]);
        // mixed-length symbols
        let lx = StringTokenizer::from("a:=3 b:4 c=5").split_on([":=", ":"], false);
        assert_eq!(
            lx.collect::<Vec<_>>(),
            ["a", ":=", "3", "b", ":", "4", "c=5"]
        );
        // symbols and delimiters
        let lx = StringTokenizer::from("a:=3,b:4")
            .split_on([":=", ":"], false)
            .split_on([","], true);
        assert_eq!(lx.collect::<Vec<_>>(), ["a", ":=", "3", "b", ":", "4"]);
        let lx = StringTokenizer::from(",b:4")
            .split_on([":=", ":"], false)
            .split_on([","], true);
        assert_eq!(lx.collect::<Vec<_>>(), ["b", ":", "4"]);
    }

    #[test]
    fn escape_pairs() {
        // Keep quotes
        let lx = StringTokenizer::from(r#"hello "escaped \" string" world"#)
            .split_by(|s| quoted(s, "\"", "\"", Some('\\')), false);
        assert_eq!(
            lx.collect::<Vec<_>>(),
            ["hello", r#""escaped \" string""#, "world"]
        );

        // Drop quotes
        let lx = StringTokenizer::from(r#"hello "escaped \" string" world"#)
            .split_by(|s| quoted_no_delims(s, "\"", "\"", Some('\\')), false);
        assert_eq!(
            lx.collect::<Vec<_>>(),
            ["hello", r#"escaped \" string"#, "world"]
        );

        // Custom block comment with no escape char, dropping the bounds
        let lx = StringTokenizer::from("code /* some \n comment */ more code")
            .split_by(|s| quoted_no_delims(s, "/*", "*/", None), false);
        assert_eq!(
            lx.collect::<Vec<_>>(),
            ["code", " some \n comment ", "more", "code"]
        );

        // Unterminated quote
        let lx = StringTokenizer::from(r#"start "unterminated"#)
            .split_by(|s| quoted(s, "\"", "\"", Some('\\')), false);
        assert_eq!(lx.collect::<Vec<_>>(), ["start", r#""unterminated"#]);
    }
}

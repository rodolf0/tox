use crate::scanner::{Scan, Scanner};
use crate::string_tokenizer::Extractor;

struct TypedExtractorConfig<'a, I: Iterator<Item = char>, T> {
    e: Box<dyn Extractor<I> + 'a>,
    mapper: Box<dyn Fn(&[char]) -> Option<T> + 'a>,
}

pub struct TypedTokenizer<'a, I: Iterator<Item = char>, T> {
    src: Scanner<I>,
    trimmer: fn(&char) -> bool,
    extractors: Vec<TypedExtractorConfig<'a, I, T>>,
    fallback: Box<dyn Fn(&[char]) -> Option<T> + 'a>,
    queued_token: Option<T>,
}

impl<'a, I: Iterator<Item = char>, T> TypedTokenizer<'a, I, T> {
    pub fn new(source: I, fallback: impl Fn(&[char]) -> Option<T> + 'a) -> Self {
        TypedTokenizer {
            src: source.scanner(),
            trimmer: |c: &char| c.is_whitespace(),
            extractors: Vec::new(),
            fallback: Box::new(fallback),
            queued_token: None,
        }
    }

    pub fn split_by(
        self,
        f: impl Fn(&mut Scanner<I>) -> Option<&[char]> + 'a,
        mapper: impl Fn(&[char]) -> Option<T> + 'a,
    ) -> Self {
        self.split_on(f, mapper)
    }

    pub fn split_on(
        mut self,
        s: impl Extractor<I> + 'a,
        mapper: impl Fn(&[char]) -> Option<T> + 'a,
    ) -> Self {
        self.extractors.push(TypedExtractorConfig {
            e: Box::new(s),
            mapper: Box::new(mapper),
        });
        self
    }

    pub fn trimmer(mut self, trimmer: fn(&char) -> bool) -> Self {
        self.trimmer = trimmer;
        self
    }
}

impl<'a, I: Iterator<Item = char>, T> Iterator for TypedTokenizer<'a, I, T> {
    type Item = T;

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
                    let chars: Vec<char> = matched.take(pre_matched).collect();
                    if let Some(token) = (self.fallback)(&chars) {
                        return Some(token);
                    }
                    continue 'outer;
                }
            }

            for TypedExtractorConfig { e, mapper } in &self.extractors {
                if let Some(extract) = e.extract(&mut self.src) {
                    let queued = mapper(extract);
                    // consume the scanner's buffer (pre-matched and cur match)
                    let matched = self.src.lift();
                    // Return the pre-existent match and queue the just extracted.
                    if pre_matched > 0 {
                        self.queued_token = queued;
                        let chars: Vec<char> = matched.take(pre_matched).collect();
                        if let Some(token) = (self.fallback)(&chars) {
                            return Some(token);
                        }
                        if let Some(token) = self.queued_token.take() {
                            return Some(token);
                        }
                        continue 'outer;
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
                // return any remaining content (pre-matched)
                let chars: Vec<char> = self.src.lift().collect();
                if let Some(token) = (self.fallback)(&chars) {
                    return Some(token);
                }
                return None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    enum Token {
        Word(String),
        Punct(String),
        Number(String),
    }

    #[test]
    fn test_typed_tokenizer() {
        let input = "hello 123 , world";
        let tokenizer = TypedTokenizer::new(input.chars(), |chars| {
            let s: String = chars.iter().collect();
            Some(Token::Word(s))
        })
        .split_on([",", "."], |chars| {
            let s: String = chars.iter().collect();
            Some(Token::Punct(s))
        })
        .split_by(
            |scanner: &mut Scanner<std::str::Chars>| {
                let cp = scanner.checkpoint();
                if scanner
                    .accept_while(|c: &char| c.is_ascii_digit())
                    .is_some()
                {
                    Some(scanner.view_from(cp))
                } else {
                    None
                }
            },
            |chars| {
                let s: String = chars.iter().collect();
                Some(Token::Number(s))
            },
        );

        let tokens: Vec<_> = tokenizer.collect();
        assert_eq!(
            tokens,
            vec![
                Token::Word("hello".to_string()),
                Token::Number("123".to_string()),
                Token::Punct(",".to_string()),
                Token::Word("world".to_string()),
            ]
        );
    }

    #[test]
    fn test_discard() {
        let input = "hello /* comment */ world";
        let tokenizer = TypedTokenizer::new(input.chars(), |chars| {
            let s: String = chars.iter().collect();
            Some(Token::Word(s))
        })
        .split_on("/* comment */", |_| None);

        let tokens: Vec<_> = tokenizer.collect();
        assert_eq!(
            tokens,
            vec![
                Token::Word("hello".to_string()),
                Token::Word("world".to_string()),
            ]
        );
    }
}

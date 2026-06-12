use crate::extractors::quoted;
use crate::string_tokenizer::StringTokenizer;

#[derive(Clone, PartialEq, Debug)]
pub enum LispToken {
    OParen,
    CParen,
    Quote,
    QuasiQuote,
    UnQuote,
    UnQSplice,
    True,
    False,
    Symbol(String),
    Number(f64),
    String(String),
}

pub struct LispTokenizer<'a, I: Iterator<Item = char>>(StringTokenizer<'a, I>);

impl<'a> From<&'a str> for LispTokenizer<'a, std::str::Chars<'a>> {
    fn from(s: &'a str) -> Self {
        Self::new(s.chars())
    }
}

impl<'a, I: Iterator<Item = char>> LispTokenizer<'a, I> {
    pub fn new(source: I) -> Self {
        let tokenizer = StringTokenizer::new(source)
            .split_on(["(", ")", "'", "`", ",@", ","], false)
            .split_by(|s| quoted(s, "\"", "\"", Some('\\')), false);
        LispTokenizer(tokenizer)
    }
}

impl<'a, I: Iterator<Item = char>> Iterator for LispTokenizer<'a, I> {
    type Item = LispToken;
    fn next(&mut self) -> Option<Self::Item> {
        let token_str = self.0.next()?;

        use std::str::FromStr;
        let token = match token_str.as_str() {
            "(" => LispToken::OParen,
            ")" => LispToken::CParen,
            "'" => LispToken::Quote,
            "`" => LispToken::QuasiQuote,
            ",@" => LispToken::UnQSplice,
            "," => LispToken::UnQuote,
            "#t" => LispToken::True,
            "#f" => LispToken::False,
            s if s.starts_with('"') => {
                let mut inner = s;
                inner = inner.strip_prefix('"').unwrap_or(inner);
                inner = inner.strip_suffix('"').unwrap_or(inner);
                LispToken::String(inner.to_string())
            }
            num => match f64::from_str(num) {
                Ok(n) => LispToken::Number(n),
                _ => LispToken::Symbol(token_str),
            },
        };
        Some(token)
    }
}

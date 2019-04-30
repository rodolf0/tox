#![deny(warnings)]

use crate::scanner::Scanner;

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

pub struct LispTokenizer<I: Iterator<Item = char>>(Scanner<I>);

impl<I: Iterator<Item = char>> LispTokenizer<I> {
    pub fn new(source: I) -> Self {
        LispTokenizer(Scanner::new(source))
    }

    pub fn scanner(source: I) -> Scanner<Self> {
        Scanner::new(Self::new(source))
    }
}

impl<I: Iterator<Item = char>> Iterator for LispTokenizer<I> {
    type Item = LispToken;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.scan_whitespace();
        if let Some(s) = self.0.scan_quoted_string('"') {
            return Some(LispToken::String(s));
        }
        if let Some(lexeme) = self.0.accept_any(&[')', '(', '\'', '`', ',']) {
            let token = match lexeme {
                '(' => LispToken::OParen,
                ')' => LispToken::CParen,
                '\'' => LispToken::Quote,
                '`' => LispToken::QuasiQuote,
                ',' => {
                    if self.0.accept(&'@').is_some() {
                        LispToken::UnQSplice
                    } else {
                        LispToken::UnQuote
                    }
                }
                _ => unreachable!(),
            };
            self.0.extract(); // ignore
            return Some(token);
        }
        if self.0.until_any(&[')', ' ', '\n', '\r', '\t']) {
            use std::str::FromStr;
            let lexeme = self.0.extract_string();
            return match &lexeme[..] {
                "#t" => Some(LispToken::True),
                "#f" => Some(LispToken::False),
                num => match f64::from_str(num) {
                    Ok(n) => Some(LispToken::Number(n)),
                    _ => Some(LispToken::Symbol(lexeme)),
                },
            };
        }
        None
    }
}

///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::{LispToken, LispTokenizer};

    #[test]
    fn lisp_tokenizer() {
        use LispToken::*;
        let inputs = vec!["(+ 3 4 5)", "(max 'a \"hello\")"];
        let expect = vec![
            vec![
                OParen,
                Symbol(format!("+")),
                Number(3.0),
                Number(4.0),
                Number(5.0),
                CParen,
            ],
            vec![
                OParen,
                Symbol(format!("max")),
                Quote,
                Symbol(format!("a")),
                String(format!("\"hello\"")),
                CParen,
            ],
        ];
        for (input, expected) in inputs.iter().zip(expect.iter()) {
            let mut lx = LispTokenizer::new(input.chars());
            for exp in expected.iter() {
                assert_eq!(*exp, lx.next().unwrap());
            }
            assert_eq!(lx.next(), None);
        }
    }
}

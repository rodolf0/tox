#![deny(warnings)]

use helpers;
use scanner::Scanner;
use std::str::FromStr;


#[derive(Clone, PartialEq, Debug)]
pub enum LispToken {
    OParen, CParen,
    Quote, QuasiQuote, UnQuote, UnQSplice,
    True, False,
    Symbol(String),
    Number(f64),
    String(String),
}

pub struct LispTokenizer(Scanner<char>);

impl LispTokenizer {
    pub fn scanner(source: &str) -> Scanner<LispToken> {
        Scanner::new(Box::new(LispTokenizer(Scanner::from_str(source))))
    }
}

impl Iterator for LispTokenizer {
    type Item = LispToken;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.ignore_ws();
        if let Some(s) = helpers::scan_quoted_string(&mut self.0, '"') {
            Some(LispToken::String(s))
        } else if let Some(t) = self.0.accept_any_char(")(\'`,") {
            let token = match t {
                '(' => LispToken::OParen,
                ')' => LispToken::CParen,
                '\'' => LispToken::Quote,
                '`' => LispToken::QuasiQuote,

                ',' => {
                    if self.0.accept_char('@') { LispToken::UnQSplice }
                    else { LispToken::UnQuote }
                },
                _ => unreachable!()
            };
            self.0.ignore();
            Some(token)
        } else if self.0.until_any_char(") \n\r\t") { // or til EOF
            let token = self.0.extract_string();
            match &token[..] {
                "#t" => Some(LispToken::True),
                "#f" => Some(LispToken::False),
                num  => match f64::from_str(num) {
                    Ok(n) => Some(LispToken::Number(n)),
                    Err(_)  => Some(LispToken::Symbol(token.clone())),
                }
            }
        } else {
            None
        }
    }
}

///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::{LispToken, LispTokenizer};

    #[test]
    fn test_lisp_tokenizer() {
        let inputs = vec![
            "(+ 3 4 5)",
            "(max 'a \"hello\")",
        ];
        let expect = vec![
            vec![LispToken::OParen, LispToken::Symbol(format!("+")),
                 LispToken::Number(3.0), LispToken::Number(4.0),
                 LispToken::Number(5.0), LispToken::CParen],
            vec![LispToken::OParen, LispToken::Symbol(format!("max")),
                 LispToken::Quote, LispToken::Symbol(format!("a")),
                 LispToken::String(format!("\"hello\"")), LispToken::CParen],
        ];
        for (input, expected) in inputs.iter().zip(expect.iter()) {
            let mut lx = LispTokenizer::scanner(input);
            for exp in expected.iter() { assert_eq!(*exp, lx.next().unwrap()); }
            assert_eq!(lx.next(), None);
        }
    }
}

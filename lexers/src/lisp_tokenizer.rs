use scanner::{Nexter, Scanner};
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
    pub fn from_str(source: &str) -> Scanner<LispToken> {
        Scanner::new(Box::new(LispTokenizer(Scanner::from_str(source))))
    }
}


impl Nexter<LispToken> for LispTokenizer {
    fn get_item(&mut self) -> Option<LispToken> {
        self.0.ignore_ws();
        let token = match self.0.next() {
            Some('(')  => LispToken::OParen,
            Some(')')  => LispToken::CParen,

            Some('\'') => LispToken::Quote,
            Some('`')  => LispToken::QuasiQuote,
            Some(',')  => match self.0.peek() {
                Some('@') => { self.0.next(); LispToken::UnQSplice },
                _ => LispToken::UnQuote,
            },

            Some('"')  => {
                self.0.until_any_char("\"");
                if self.0.next() != Some('"') { // consume closing quote
                    self.0.ignore();
                    return None; // drop partial string, parse as unexpected EOF
                } else {
                    let token = self.0.extract();
                    LispToken::String(token.iter()
                                  .take(token.len() - 1)
                                  .skip(1).cloned().collect())
                }
            },
            Some(_) => {
                self.0.until_any_char(" \n\r\t)");
                let token = self.0.extract_string();
                match &token[..] {
                    "#t" => LispToken::True,
                    "#f" => LispToken::False,
                    num  => match f64::from_str(num) {
                        Ok(n) => LispToken::Number(n),
                        Err(_)  => LispToken::Symbol(token.clone())
                    }
                }
            },
            None => return None
        };
        self.0.ignore();
        Some(token)
    }
}

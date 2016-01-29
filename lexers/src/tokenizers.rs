use helpers;
use scanner::{Nexter, Scanner};
use std::str::FromStr;

#[derive(Clone, PartialEq, Debug)]
pub enum MathToken {
    Unknown(String),
    Number(f64),
    Variable(String),
    Function(String),
    UOp(String), BOp(String),
    OParen, CParen, Comma,
}

pub struct MathTokenizer {
    src: Scanner<char>,
    prev: Option<MathToken>
}

impl MathTokenizer {
    pub fn from_str(source: &str) -> Scanner<MathToken> {
        Scanner::new(Box::new(
            MathTokenizer{src: Scanner::from_str(source), prev: None}))
    }

    // when would a minus be unary? we need to know the prev token
    fn makes_unary(prev: &Option<MathToken>) -> bool {
        match *prev {
            Some(MathToken::Number(_)) => false,
            Some(MathToken::Variable(_)) => false,
            Some(MathToken::CParen) => false,
            _ => true
        }
    }

    fn get_token(&mut self) -> Option<MathToken> {
        self.src.ignore_ws(); // discard whatever came before + and spaces
        if let Some(op) = helpers::scan_math_op(&mut self.src) {
            match op.as_ref() {
                "(" => Some(MathToken::OParen),
                ")" => Some(MathToken::CParen),
                "," => Some(MathToken::Comma),
                "!" => Some(MathToken::UOp(op)),
                "-" if Self::makes_unary(&self.prev) => Some(MathToken::UOp(op)),
                _ => Some(MathToken::BOp(op)),
            }
        } else if let Some(id) = helpers::scan_identifier(&mut self.src) {
            match self.src.peek() {
                Some('(') => Some(MathToken::Function(id)),
                _ => Some(MathToken::Variable(id))
            }
        } else if let Some(num) = helpers::scan_number(&mut self.src) {
            Some(MathToken::Number(f64::from_str(&num).unwrap()))
        } else if let Some(_) = self.src.next() {
            Some(MathToken::Unknown(self.src.extract_string()))
        } else {
            None
        }
    }
}

impl Nexter<MathToken> for MathTokenizer {
    fn get_item(&mut self) -> Option<MathToken> {
        let token = self.get_token();
        self.prev = token.clone();
        token
    }
}

///////////////////////////////////////////////////////////////////////////////

// A tokenizer that splits input on each delimiter
pub struct DelimTokenizer {
    src: Scanner<char>,
    delims: String,
    remove: bool, // drop the delimiters ?
}

impl DelimTokenizer {
    pub fn from_str<S>(src: &str, delims: S, remove: bool) -> Scanner<String>
    where S: Into<String> {
        Scanner::new(Box::new(
            DelimTokenizer{src: Scanner::from_str(src),
                delims: delims.into(), remove: remove}))
    }
}

impl Nexter<String> for DelimTokenizer {
    fn get_item(&mut self) -> Option<String> {
        if self.src.until_any_char(&self.delims) {
            Some(self.src.extract_string())
        } else if let Some(c) = self.src.accept_any_char(&self.delims) {
            self.src.ignore();
            match self.remove {
                false => Some(c.to_string()),
                true => self.get_item()
            }
        } else {
            None
        }
    }
}

///////////////////////////////////////////////////////////////////////////////

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

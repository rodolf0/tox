use helpers;
use scanner::{Nexter, Scanner};
use std::str::FromStr;

#[derive(Clone, PartialEq, Debug)]
pub enum MathToken {
    Unknown(String),
    Number(f64),
    Variable(String),
    Function(String, usize), // arity
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
                Some('(') => Some(MathToken::Function(id, 0)),
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
        if let Some(s) = helpers::scan_quoted_string(&mut self.0, '"') {
            Some(LispToken::String(s))
        } else if let Some(t) = self.0.accept_any_char(")(\'`,") {
            let token = match t {
                '(' => LispToken::OParen,
                ')' => LispToken::CParen,
                '\'' => LispToken::Quote,
                '`' => LispToken::QuasiQuote,
                ',' => match self.0.accept_char('@') {
                    true => LispToken::UnQSplice,
                    false => LispToken::UnQuote,
                },
                _ => unreachable!()
            };
            self.0.ignore();
            Some(token)
        } else {
            if self.0.until_any_char(") \n\r\t") { // or til EOF
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
}

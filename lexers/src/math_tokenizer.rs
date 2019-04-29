#![deny(warnings)]

use crate::helpers;
use crate::scanner::Scanner;


#[derive(Clone, PartialEq, Debug)]
pub enum MathToken {
    Unknown(String),
    Number(f64),
    Variable(String),
    Function(String, usize), // arity
    UOp(String), BOp(String),
    OParen, CParen, Comma,
}

pub struct MathTokenizer<I: Iterator<Item=char>> {
    src: Scanner<I>,
    prev: Option<MathToken>
}

impl<I: Iterator<Item=char>> MathTokenizer<I> {
    pub fn new(source: I) -> Self {
        MathTokenizer{src: Scanner::new(source), prev: None}
    }

    pub fn scanner(source: I) -> Scanner<Self> {
        Scanner::new(Self::new(source))
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
            use std::str::FromStr;
            Some(MathToken::Number(f64::from_str(&num).unwrap()))
        } else if self.src.next().is_some() {
            Some(MathToken::Unknown(self.src.extract_string()))
        } else {
            None
        }
    }
}

impl<I: Iterator<Item=char>> Iterator for MathTokenizer<I> {
    type Item = MathToken;
    fn next(&mut self) -> Option<Self::Item> {
        let token = self.get_token();
        self.prev = token.clone();
        token
    }
}

///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::{MathToken, MathTokenizer};

    #[test]
    fn basic_ops() {
        let mut lx = MathTokenizer::new("3+4*2/-(1-5)^2^3".chars());
        let expect = [
            MathToken::Number(3.0),
            MathToken::BOp(format!("+")),
            MathToken::Number(4.0),
            MathToken::BOp(format!("*")),
            MathToken::Number(2.0),
            MathToken::BOp(format!("/")),
            MathToken::UOp(format!("-")),
            MathToken::OParen,
            MathToken::Number(1.0),
            MathToken::BOp(format!("-")),
            MathToken::Number(5.0),
            MathToken::CParen,
            MathToken::BOp(format!("^")),
            MathToken::Number(2.0),
            MathToken::BOp(format!("^")),
            MathToken::Number(3.0),
        ];
        for exp_token in expect.iter() {
            let token = lx.next().unwrap();
            assert_eq!(*exp_token, token);
        }
        assert_eq!(lx.next(), None);
    }

    #[test]
    fn mixed_ops() {
        let mut lx = MathTokenizer::new("3.4e-2 * sin(x)/(7! % -4) * max(2, x)".chars());
        let expect = [
            MathToken::Number(3.4e-2),
            MathToken::BOp(format!("*")),
            MathToken::Function(format!("sin"), 0),
            MathToken::OParen,
            MathToken::Variable(format!("x")),
            MathToken::CParen,
            MathToken::BOp(format!("/")),
            MathToken::OParen,
            MathToken::Number(7.0),
            MathToken::UOp(format!("!")),
            MathToken::BOp(format!("%")),
            MathToken::UOp(format!("-")),
            MathToken::Number(4.0),
            MathToken::CParen,
            MathToken::BOp(format!("*")),
            MathToken::Function(format!("max"), 0),
            MathToken::OParen,
            MathToken::Number(2.0),
            MathToken::Comma,
            MathToken::Variable(format!("x")),
            MathToken::CParen,
        ];
        for exp_token in expect.iter() {
            let token = lx.next().unwrap();
            assert_eq!(*exp_token, token);
        }
        assert_eq!(lx.next(), None);
    }

    #[test]
    fn unary_ops() {
        let mut lx = MathTokenizer::new("x---y".chars());
        let expect = [
            MathToken::Variable(format!("x")),
            MathToken::BOp(format!("-")),
            MathToken::UOp(format!("-")),
            MathToken::UOp(format!("-")),
            MathToken::Variable(format!("y")),
        ];
        for exp_token in expect.iter() {
            let token = lx.next().unwrap();
            assert_eq!(*exp_token, token);
        }
        assert_eq!(lx.next(), None);
    }
}

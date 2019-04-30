#![deny(warnings)]

use crate::scanner::Scanner;

#[derive(Clone, PartialEq, Debug)]
pub enum MathToken {
    Unknown(String),
    Number(f64),
    Variable(String),
    Function(String, usize), // arity
    UOp(String),
    BOp(String),
    OParen,
    CParen,
    Comma,
}

pub struct MathTokenizer<I: Iterator<Item = char>> {
    src: Scanner<I>,
    prev: Option<MathToken>,
}

impl<I: Iterator<Item = char>> MathTokenizer<I> {
    pub fn new(source: I) -> Self {
        MathTokenizer {
            src: Scanner::new(source),
            prev: None,
        }
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
            _ => true,
        }
    }

    fn get_token(&mut self) -> Option<MathToken> {
        self.src.scan_whitespace(); // discard whatever came before + and spaces
        if let Some(op) = self.src.scan_math_op() {
            return match op.as_ref() {
                "(" => Some(MathToken::OParen),
                ")" => Some(MathToken::CParen),
                "," => Some(MathToken::Comma),
                "!" => Some(MathToken::UOp(op)),
                "-" if Self::makes_unary(&self.prev) => Some(MathToken::UOp(op)),
                _ => Some(MathToken::BOp(op)),
            };
        }
        if let Some(id) = self.src.scan_identifier() {
            return match self.src.peek() {
                Some('(') => Some(MathToken::Function(id, 0)),
                _ => Some(MathToken::Variable(id)),
            };
        }
        if let Some(num) = self.src.scan_number() {
            use std::str::FromStr;
            return Some(MathToken::Number(f64::from_str(&num).unwrap()));
        }
        if self.src.next().is_some() {
            return Some(MathToken::Unknown(self.src.extract_string()));
        }
        None
    }
}

impl<I: Iterator<Item = char>> Iterator for MathTokenizer<I> {
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
    use super::{MathToken::*, MathTokenizer};

    #[test]
    fn basic_ops() {
        let mut lx = MathTokenizer::new("3+4*2/-(1-5)^2^3".chars());
        let expect = [
            Number(3.0),
            BOp(format!("+")),
            Number(4.0),
            BOp(format!("*")),
            Number(2.0),
            BOp(format!("/")),
            UOp(format!("-")),
            OParen,
            Number(1.0),
            BOp(format!("-")),
            Number(5.0),
            CParen,
            BOp(format!("^")),
            Number(2.0),
            BOp(format!("^")),
            Number(3.0),
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
            Number(3.4e-2),
            BOp(format!("*")),
            Function(format!("sin"), 0),
            OParen,
            Variable(format!("x")),
            CParen,
            BOp(format!("/")),
            OParen,
            Number(7.0),
            UOp(format!("!")),
            BOp(format!("%")),
            UOp(format!("-")),
            Number(4.0),
            CParen,
            BOp(format!("*")),
            Function(format!("max"), 0),
            OParen,
            Number(2.0),
            Comma,
            Variable(format!("x")),
            CParen,
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
            Variable(format!("x")),
            BOp(format!("-")),
            UOp(format!("-")),
            UOp(format!("-")),
            Variable(format!("y")),
        ];
        for exp_token in expect.iter() {
            let token = lx.next().unwrap();
            assert_eq!(*exp_token, token);
        }
        assert_eq!(lx.next(), None);
    }
}

#![deny(warnings)]

use helpers;
use scanner::Scanner;
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
    pub fn scanner(source: &str) -> Scanner<MathToken> {
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

impl Iterator for MathTokenizer {
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
    fn test1() {
        let mut lx = MathTokenizer::scanner("3+4*2/-(1-5)^2^3");
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
    fn test2() {
        let mut lx = MathTokenizer::scanner("3.4e-2 * sin(x)/(7! % -4) * max(2, x)");
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
    fn test3() {
        let mut lx = MathTokenizer::scanner("sqrt(-(1-x^2) / (1 + x^2))");
        let expect = [
            MathToken::Function(format!("sqrt"), 0),
            MathToken::OParen,
            MathToken::UOp(format!("-")),
            MathToken::OParen,
            MathToken::Number(1.0),
            MathToken::BOp(format!("-")),
            MathToken::Variable(format!("x")),
            MathToken::BOp(format!("^")),
            MathToken::Number(2.0),
            MathToken::CParen,
            MathToken::BOp(format!("/")),
            MathToken::OParen,
            MathToken::Number(1.0),
            MathToken::BOp(format!("+")),
            MathToken::Variable(format!("x")),
            MathToken::BOp(format!("^")),
            MathToken::Number(2.0),
            MathToken::CParen,
            MathToken::CParen,
        ];
        for exp_token in expect.iter() {
            let token = lx.next().unwrap();
            assert_eq!(*exp_token, token);
        }
        assert_eq!(lx.next(), None);
    }

    #[test]
    fn test4() {
        let mut lx = MathTokenizer::scanner("x---y");
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

    #[test]
    fn test5() {
        let mut lx = MathTokenizer::scanner("max(0, 1, 3)");
        let expect = [
            MathToken::Function(format!("max"), 0),
            MathToken::OParen,
            MathToken::Number(0.0),
            MathToken::Comma,
            MathToken::Number(1.0),
            MathToken::Comma,
            MathToken::Number(3.0),
            MathToken::CParen,
        ];
        for exp_token in expect.iter() {
            let token = lx.next().unwrap();
            assert_eq!(*exp_token, token);
        }
        assert_eq!(lx.next(), None);
    }
}

#![deny(warnings)]

use crate::scanner::Scanner;

#[derive(Clone, PartialEq, Debug)]
pub enum MathToken {
    Unknown(String),
    Number(f64),
    Quantity(f64, String, String),
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
        !matches!(*prev,
            Some(MathToken::Number(_)) |
            Some(MathToken::Variable(_)) |
            Some(MathToken::CParen))
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
            self.src.scan_whitespace(); // discard whatever came before + and spaces
            use std::str::FromStr;
            let value = f64::from_str(&num).unwrap();
            if let Some((prefix, unit)) = self.src.scan_unit() {
                return Some(MathToken::Quantity(value, prefix, unit));
            }
            return Some(MathToken::Number(value));
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
            BOp("+".to_string()),
            Number(4.0),
            BOp("*".to_string()),
            Number(2.0),
            BOp("/".to_string()),
            UOp("-".to_string()),
            OParen,
            Number(1.0),
            BOp("-".to_string()),
            Number(5.0),
            CParen,
            BOp("^".to_string()),
            Number(2.0),
            BOp("^".to_string()),
            Number(3.0),
        ];
        for exp_token in expect.iter() {
            let token = lx.next().unwrap();
            assert_eq!(*exp_token, token);
        }
        assert_eq!(lx.next(), None);

        let mut lx = MathTokenizer::new("x := a + b".chars());
        let expect = [
            Variable("x".to_string()),
            BOp(":=".to_string()),
            Variable("a".to_string()),
            BOp("+".to_string()),
            Variable("b".to_string()),
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
            BOp("*".to_string()),
            Function("sin".to_string(), 0),
            OParen,
            Variable("x".to_string()),
            CParen,
            BOp("/".to_string()),
            OParen,
            Number(7.0),
            UOp("!".to_string()),
            BOp("%".to_string()),
            UOp("-".to_string()),
            Number(4.0),
            CParen,
            BOp("*".to_string()),
            Function("max".to_string(), 0),
            OParen,
            Number(2.0),
            Comma,
            Variable("x".to_string()),
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
            Variable("x".to_string()),
            BOp("-".to_string()),
            UOp("-".to_string()),
            UOp("-".to_string()),
            Variable("y".to_string()),
        ];
        for exp_token in expect.iter() {
            let token = lx.next().unwrap();
            assert_eq!(*exp_token, token);
        }
        assert_eq!(lx.next(), None);
    }

    #[test]
    fn quantity() {
        let mut lx = MathTokenizer::new("30km / (10 s) * 20g * 3 GHz".chars());
        let expect = [
            Quantity(30.0, "k".to_string(), "m".to_string()),
            BOp("/".to_string()),
            OParen,
            Quantity(10.0, "".to_string(), "s".to_string()),
            CParen,
            BOp("*".to_string()),
            Quantity(20.0, "".to_string(), "g".to_string()),
            BOp("*".to_string()),
            Quantity(3.0, "G".to_string(), "Hz".to_string()),
        ];
        for exp_token in expect.iter() {
            let token = lx.next().unwrap();
            assert_eq!(*exp_token, token);
        }
        assert_eq!(lx.next(), None);
    }
}

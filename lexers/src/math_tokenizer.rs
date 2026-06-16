use crate::scanner::Scanner;
use crate::typed_tokenizer::TypedTokenizer;
use std::str::FromStr;

#[derive(Clone, PartialEq, Debug)]
pub enum MathToken {
    Unknown(String),
    Number(f64),
    Quantity(f64, String, String),
    Variable(String),
    Function(String, usize),
    UOp(String),
    BOp(String),
    OParen,
    CParen,
    Comma,
}

pub struct MathTokenizer<'a, I: Iterator<Item = char> + 'a> {
    tokenizer: TypedTokenizer<'a, I, MathToken>,
    queued: Option<MathToken>,
    prev_makes_unary: bool,
}

impl<'a> From<&'a str> for MathTokenizer<'a, std::str::Chars<'a>> {
    fn from(s: &'a str) -> Self {
        Self::new(s.chars())
    }
}

fn makes_unary(prev: Option<&MathToken>) -> bool {
    !matches!(
        prev,
        Some(MathToken::Number(_))
            | Some(MathToken::Quantity(_, _, _))
            | Some(MathToken::Variable(_))
            | Some(MathToken::CParen)
    )
}

impl<'a, I: Iterator<Item = char> + 'a> MathTokenizer<'a, I> {
    pub fn new(source: I) -> Self {
        use MathToken::*;

        let tokenizer = TypedTokenizer::new(source, |chars| {
            let s: String = chars.iter().collect();
            let mut sc = Scanner::new(s.chars());
            if let Some(id) = crate::extractors::identifier(&mut sc) {
                if id.len() == s.len() {
                    return Some(Variable(s));
                }
            }
            Some(Unknown(s))
        })
        // NOTE: order matters to avoid matching shortest first !
        .split_on(["<=", ">=", "==", "<", ">", "!="], |s| {
            Some(BOp(s.iter().collect()))
        })
        .split_on([":=", "="], |s| Some(BOp(s.iter().collect())))
        .split_on(["**", "^", "*", "/", "%", "+", "-"], |s| {
            Some(BOp(s.iter().collect()))
        })
        .split_on("(", |_| Some(OParen))
        .split_on(")", |_| Some(CParen))
        .split_on(",", |_| Some(Comma))
        .split_on("!", |_| Some(UOp("!".to_string())))
        .split_by(crate::extractors::number, |chars| {
            let s: String = chars.iter().collect();
            if let Ok(val) = f64::from_str(&s) {
                Some(Number(val))
            } else {
                Some(Unknown(s))
            }
        });
        MathTokenizer {
            tokenizer: tokenizer,
            queued: None,
            prev_makes_unary: makes_unary(None),
        }
    }
}

impl<'a, I: Iterator<Item = char> + 'a> Iterator for MathTokenizer<'a, I> {
    type Item = MathToken;

    fn next(&mut self) -> Option<Self::Item> {
        use MathToken::*;
        // Get the queued token, or next, or EOF.
        let token = self.queued.take().or_else(|| self.tokenizer.next())?;
        // Fill the queue. Some classifications depend on the next token.
        self.queued = self.tokenizer.next();
        // Check if the token needs to be re-classified
        let token = match token {
            BOp(op) if op == "-" && self.prev_makes_unary => UOp(op),
            Variable(v) if self.queued == Some(OParen) => Function(v, 0),
            Number(n) => {
                match self.queued {
                    Some(Variable(ref v)) => {
                        let unit_scanner = &mut Scanner::new(v.chars());
                        let unit = crate::extractors::unit(unit_scanner);
                        if unit_scanner.next().is_none() && let Some((p, u)) = unit {
                            self.queued = None;
                            Quantity(n, p.to_owned(), u.to_owned())
                        } else {
                            Number(n)
                        }
                    },
                    _ => Number(n)
                }
            }
            other => other,
        };
        // Remember if the token will affect unary ops later
        self.prev_makes_unary = makes_unary(Some(&token));
        Some(token)
    }
}

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

    #[test]
    fn non_quantity() {
        let mut lx = MathTokenizer::new("30kms".chars());
        let expect = [Number(30.0), Variable("kms".to_string())];
        for exp_token in expect.iter() {
            let token = lx.next().unwrap();
            assert_eq!(*exp_token, token);
        }
        assert_eq!(lx.next(), None);
    }
}

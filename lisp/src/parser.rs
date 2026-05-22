use crate::procedure::Procedure;
use lexers::{LispToken, LispTokenizer};
use std::rc::Rc;
use std::string;

#[derive(PartialEq, Debug)]
pub enum ParseError {
    UnexpectedCParen,
    UnexpectedEOF,
    NotImplemented,
}

#[derive(Clone, PartialEq, PartialOrd, Debug)]
pub enum Expr {
    List(Vec<Expr>),
    String(String),
    Symbol(String),
    Number(f64),
    True,
    False,
    Proc(Rc<Procedure>),
    Quote(Box<Expr>),
    QuasiQuote(Box<Expr>),
    UnQuote(Box<Expr>),
    UnQSplice(Box<Expr>),
}

impl string::ToString for Expr {
    fn to_string(&self) -> String {
        match self {
            &Expr::Symbol(ref s) => s.clone(),
            &Expr::String(ref s) => s.clone(),
            &Expr::Number(n) => format!("{}", n),
            &Expr::List(ref v) => {
                let base = match v.first() {
                    Some(expr) => expr.to_string(),
                    None => String::new(),
                };
                format!(
                    "({})",
                    v.iter()
                        .skip(1)
                        .fold(base, |a, ref it| format!("{} {}", a, it.to_string()))
                )
            }
            &Expr::True => format!("#t"),
            &Expr::False => format!("#f"),
            &Expr::Proc(ref p) => format!("{:?}", *p),
            &Expr::Quote(ref e) => format!("'{}", e.to_string()),
            &Expr::QuasiQuote(ref e) => format!("`{}", e.to_string()),
            &Expr::UnQuote(ref e) => format!(",{}", e.to_string()),
            &Expr::UnQSplice(ref e) => format!(",@{}", e.to_string()),
        }
    }
}

fn _parse<I>(lex: &mut std::iter::Peekable<LispTokenizer<I>>) -> Result<Expr, ParseError>
where
    I: Iterator<Item = char>,
{
    match lex.next() {
        None => Err(ParseError::UnexpectedEOF),
        Some(LispToken::CParen) => Err(ParseError::UnexpectedCParen),
        Some(LispToken::True) => Ok(Expr::True),
        Some(LispToken::False) => Ok(Expr::False),
        Some(LispToken::String(n)) => Ok(Expr::String(n)),
        Some(LispToken::Number(n)) => Ok(Expr::Number(n)),
        Some(LispToken::Symbol(s)) => Ok(Expr::Symbol(s)),
        Some(LispToken::OParen) => {
            let mut list = Vec::new();
            while lex.peek() != Some(&LispToken::CParen) {
                list.push(_parse(lex)?);
            }
            lex.next(); // get over that CParen
            Ok(Expr::List(list))
        }
        Some(LispToken::Quote) => Ok(Expr::Quote(Box::new(_parse(lex)?))),
        Some(LispToken::QuasiQuote) => Ok(Expr::QuasiQuote(Box::new(_parse(lex)?))),
        Some(LispToken::UnQuote) => Ok(Expr::UnQuote(Box::new(_parse(lex)?))),
        Some(LispToken::UnQSplice) => Ok(Expr::UnQSplice(Box::new(_parse(lex)?))),
    }
}

pub fn parse(expr: &str) -> Result<Expr, ParseError> {
    _parse(&mut LispTokenizer::from(expr).peekable())
}

#[cfg(test)]
mod tests {
    use super::{Expr, parse};

    #[test]
    fn test_lisp1() {
        let p = parse("(begin (define r 10) (* pi (* r r)))");
        let r = Expr::List(vec![
            Expr::Symbol(format!("begin")),
            Expr::List(vec![
                Expr::Symbol(format!("define")),
                Expr::Symbol(format!("r")),
                Expr::Number(10.0),
            ]),
            Expr::List(vec![
                Expr::Symbol(format!("*")),
                Expr::Symbol(format!("pi")),
                Expr::List(vec![
                    Expr::Symbol(format!("*")),
                    Expr::Symbol(format!("r")),
                    Expr::Symbol(format!("r")),
                ]),
            ]),
        ]);
        assert_eq!(p.unwrap(), r);
    }
}

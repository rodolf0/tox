use lexers::{Scanner, LispToken, LispTokenizer};
use crate::procedure::Procedure;
use std::string;
use std::rc::Rc;

#[derive(PartialEq, Debug)]
pub enum ParseError {
    UnexpectedCParen,
    UnexpectedEOF,
    NotImplemented,
}

#[derive(Clone, PartialEq, PartialOrd, Debug)]
pub enum LispExpr {
    List(Vec<LispExpr>),
    String(String),
    Symbol(String),
    Number(f64),
    True, False,
    Proc(Rc<Procedure>),
    Quote(Box<LispExpr>),
    QuasiQuote(Box<LispExpr>),
    UnQuote(Box<LispExpr>),
    UnQSplice(Box<LispExpr>),
}

impl string::ToString for LispExpr {
    fn to_string(&self) -> String {
        match self {
            &LispExpr::Symbol(ref s) => s.clone(),
            &LispExpr::String(ref s) => s.clone(),
            &LispExpr::Number(n) => format!("{}", n),
            &LispExpr::List(ref v) => {
                let base = match v.first() {
                    Some(expr) => expr.to_string(),
                    None => String::new()
                };
                format!("({})", v.iter().skip(1)
                    .fold(base, |a, ref it|
                          format!("{} {}", a, it.to_string())))
            },
            &LispExpr::True  => format!("#t"),
            &LispExpr::False => format!("#f"),
            &LispExpr::Proc(ref p) => format!("{:?}", *p),
            &LispExpr::Quote(ref e) => format!("'{}", e.to_string()),
            &LispExpr::QuasiQuote(ref e) => format!("`{}", e.to_string()),
            &LispExpr::UnQuote(ref e) => format!(",{}", e.to_string()),
            &LispExpr::UnQSplice(ref e) => format!(",@{}", e.to_string()),
        }
    }
}


pub struct Parser;

impl Parser {
    pub fn parse_str(expr: &str) -> Result<LispExpr, ParseError> {
        Self::parse(&mut LispTokenizer::scanner(expr))
    }

    fn parse(lex: &mut Scanner<LispToken>) -> Result<LispExpr, ParseError> {
        match lex.next() {
            None                        => Err(ParseError::UnexpectedEOF),
            Some(LispToken::CParen)     => Err(ParseError::UnexpectedCParen),
            Some(LispToken::True)       => Ok(LispExpr::True),
            Some(LispToken::False)      => Ok(LispExpr::False),
            Some(LispToken::String(n))  => Ok(LispExpr::String(n)),
            Some(LispToken::Number(n))  => Ok(LispExpr::Number(n)),
            Some(LispToken::Symbol(s))  => Ok(LispExpr::Symbol(s)),
            Some(LispToken::OParen)     => {
                let mut list = Vec::new();
                while lex.peek() != Some(LispToken::CParen) { // even when != None
                    match Parser::parse(lex) {
                        Err(err) => return Err(err),
                        Ok(expr) => list.push(expr),
                    }
                }
                lex.next(); // get over that CParen
                Ok(LispExpr::List(list))
            },
            Some(LispToken::Quote) => {
                let expr = Parser::parse(lex)?;
                Ok(LispExpr::Quote(Box::new(expr)))
            },
            Some(LispToken::QuasiQuote) => {
                let expr = Parser::parse(lex)?;
                Ok(LispExpr::QuasiQuote(Box::new(expr)))
            },
            Some(LispToken::UnQuote) => {
                let expr = Parser::parse(lex)?;
                Ok(LispExpr::UnQuote(Box::new(expr)))
            },
            Some(LispToken::UnQSplice) => {
                let expr = Parser::parse(lex)?;
                Ok(LispExpr::UnQSplice(Box::new(expr)))
            }
        }
    }
}

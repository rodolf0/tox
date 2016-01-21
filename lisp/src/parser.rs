use lisp::{Token, Lexer, Procedure};
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
        Self::parse(&mut Lexer::from_str(expr))
    }

    fn parse(lex: &mut Lexer) -> Result<LispExpr, ParseError> {
        match lex.next() {
            None                    => Err(ParseError::UnexpectedEOF),
            Some(Token::CParen)     => Err(ParseError::UnexpectedCParen),
            Some(Token::True)       => Ok(LispExpr::True),
            Some(Token::False)      => Ok(LispExpr::False),
            Some(Token::String(n))  => Ok(LispExpr::String(n)),
            Some(Token::Number(n))  => Ok(LispExpr::Number(n)),
            Some(Token::Symbol(s))  => Ok(LispExpr::Symbol(s)),
            Some(Token::OParen)     => {
                let mut list = Vec::new();
                while lex.peek() != Some(Token::CParen) { // even when != None
                    match Parser::parse(lex) {
                        Err(err) => return Err(err),
                        Ok(expr) => list.push(expr),
                    }
                }
                lex.next(); // get over that CParen
                Ok(LispExpr::List(list))
            },
            Some(Token::Quote) => {
                let expr = try!(Parser::parse(lex));
                Ok(LispExpr::Quote(Box::new(expr)))
            },
            Some(Token::QuasiQuote) => {
                let expr = try!(Parser::parse(lex));
                Ok(LispExpr::QuasiQuote(Box::new(expr)))
            },
            Some(Token::UnQuote) => {
                let expr = try!(Parser::parse(lex));
                Ok(LispExpr::UnQuote(Box::new(expr)))
            },
            Some(Token::UnQSplice) => {
                let expr = try!(Parser::parse(lex));
                Ok(LispExpr::UnQSplice(Box::new(expr)))
            }
        }
    }
}

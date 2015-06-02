use scanner::{Scanner, Nexter};
use std::ops::{Deref, DerefMut};
use std::str::FromStr;

#[derive(Clone, PartialEq, Debug)]
pub enum Atom {
    Symbol(String),
    Number(f64),
}

#[derive(Clone, PartialEq, Debug)]
enum Token {
    OParen,
    CParen,
    Quote,
    Atom(Atom),
}

struct Tokenizer {
    src: Scanner<char>,
}

impl Nexter<Token> for Tokenizer {
    fn get_item(&mut self) -> Option<Token> {
        self.src.ignore_ws();
        match self.src.next() {
            Some('(')  => Some(Token::OParen),
            Some(')')  => Some(Token::CParen),
            Some('\'') => Some(Token::Quote),
            Some(_) => {
                self.src.until_chars(" \n\r\t'()");
                let token = self.src.extract_string();
                match f64::from_str(&token) {
                    Ok(num) => Some(Token::Atom(Atom::Number(num))),
                    Err(_)  => Some(Token::Atom(Atom::Symbol(token)))
                }
            },
            None => None
        }
    }
}

struct Lexer {
    output: Scanner<Token>,
}

impl Deref for Lexer {
    type Target = Scanner<Token>;
    fn deref<'a>(&'a self) -> &'a Scanner<Token> { &self.output }
}

impl DerefMut for Lexer {
    fn deref_mut<'a>(&'a mut self) -> &'a mut Scanner<Token> { &mut self.output }
}

impl Lexer {
    fn from_str(source: &str) -> Lexer {
        let tokenizer = Box::new(Tokenizer{src: Scanner::from_str(source)});
        Lexer{output: Scanner::new(tokenizer)}
    }
}

#[derive(PartialEq, Debug)]
pub enum ParseError {
    UnexpectedCParen,
    UnexpectedEOF,
    NotImplemented,
}

pub struct Parser;

#[derive(PartialEq, Debug)]
pub enum LispExpr {
    List(Vec<LispExpr>),
    Atom(Atom),
}

impl Parser {
    pub fn parse_str(expr: &str) -> Result<LispExpr, ParseError> {
        Self::parse(&mut Lexer::from_str(expr))
    }

    fn parse(lex: &mut Lexer) -> Result<LispExpr, ParseError> {
        match lex.next() {
            None                    => Err(ParseError::UnexpectedEOF),
            Some(Token::CParen)     => Err(ParseError::UnexpectedCParen),
            Some(Token::Quote)      => Err(ParseError::NotImplemented),
            Some(Token::Atom(atom)) => Ok(LispExpr::Atom(atom)),
            Some(Token::OParen)     => {
                let mut list = Vec::new();
                while lex.peek() != Some(Token::CParen) { // even None
                    match Parser::parse(lex) {
                        Err(err) => return Err(err),
                        Ok(expr) => list.push(expr),
                    }
                }
                lex.next(); // get over that CParen
                Ok(LispExpr::List(list))
            },
        }
    }
}

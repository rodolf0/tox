use scanner::{Scanner, Nexter};
use std::collections::HashMap;
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

#[derive(Clone, PartialEq, Debug)]
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
                while lex.peek() != Some(Token::CParen) { // even != None
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

#[derive(PartialEq, Debug)]
pub enum EvalErr {
    UnknownVar,
    UnknownFunction,
    InvalidExpr,
    WrongArgs,
    WrongNumberOfArgs,
}

pub struct LispContext {
    vars: HashMap<String, LispExpr>,
    funcs: HashMap<String, fn(LispExpr) -> LispExpr>,
    outer: Option<Box<LispContext>>,
}

impl LispContext {
    pub fn new() -> LispContext {
        let mut funcs = HashMap::new();
        //funcs.insert(format!("+"),
            //|args: Vec<LispExpr>| LispExpr::Atom(Atom::Number(
                //args.iter().fold(0.0, |a, &item| a + item)
            //)));
        LispContext{vars: HashMap::new(), funcs: funcs, outer: None}
    }

    pub fn eval(&mut self, expr: &str) -> Result<LispExpr, EvalErr> {
        let e = Parser::parse_str(expr);
        Self::_eval(&e.unwrap(), self)
    }

    // TODO: return <&LispExpr, EvalErr>
    fn _eval(expr: &LispExpr, ctx: &mut LispContext) -> Result<LispExpr, EvalErr> {
        match expr {
            &LispExpr::Atom(ref atom) => match atom {
                &Atom::Number(_) => Ok(LispExpr::Atom(atom.clone())),
                &Atom::Symbol(ref sym) => match ctx.vars.get(sym) {
                    Some(value) => Ok(value.clone()),
                    None => Err(EvalErr::UnknownVar)
                }
            },
            &LispExpr::List(ref list) => match list.first() {
                Some(&LispExpr::Atom(Atom::Symbol(ref first))) => match &(*first)[..] {
                    "quote" if list.len() > 1   => Ok(LispExpr::List(list[1..].to_vec())),
                    "if"    if list.len() == 4  => {
                        let (test, conseq, alt) = (&list[1], &list[2], &list[3]);
                        match Self::_eval(test, ctx) {
                            Err(err) => Err(err),
                            Ok(ref res) if true  => Self::_eval(conseq, ctx), // TODO num != 0 || list != []
                            Ok(ref res) if false => Self::_eval(alt, ctx),
                            _ => unreachable!()
                        }
                    },
                    "define" if list.len() == 3 => {
                        if let (&LispExpr::Atom(Atom::Symbol(ref sym)), exp) = (&list[1], &list[2]) {
                            match Self::_eval(exp, ctx) {
                                Ok(res) => { ctx.vars.insert(sym.clone(), res.clone()); Ok(res) }, // TODO check type to insert in proper struct
                                Err(err) => Err(err)
                            }
                        } else {
                            Err(EvalErr::WrongArgs)
                        }
                    },
                    "quote" | "if" | "define" => Err(EvalErr::WrongNumberOfArgs),
                    _ if list.len() > 1 => {
                        match Self::_eval(&list[0], ctx) {
                            Ok(LispExpr::Atom(Atom::Symbol(fsym))) => {
                                //let args = list[1..].iter().map(|arg| Self::_eval(arg, ctx)).collect::<Vec<Result<LispExpr, EvalErr>>>();
                                //if let Some(err) = args.iter().filter(|arg| arg.is_err()).next() {
                                    //return Err(err.unwrap_err());
                                //}
                                //let args = LispExpr::List(args.iter().map(|arg| arg.unwrap()).collect());
                                let args = LispExpr::List(list[1..].iter().map(|arg| Self::_eval(arg, ctx).unwrap()).collect());
                                match ctx.funcs.get(&fsym) {
                                    Some(func) => Ok(func(args)),
                                    None => Err(EvalErr::UnknownFunction)
                                }
                            }, // TODO check if num or procedure
                            Ok(_)    => Err(EvalErr::InvalidExpr), // expected symbol
                            Err(err) => Err(err),
                        }
                    },
                    _ => Err(EvalErr::WrongNumberOfArgs),
                },
                _ => Err(EvalErr::InvalidExpr) // () | ((x) ...) | (3 ...)
            },
        }
    }
}

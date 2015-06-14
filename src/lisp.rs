use scanner::{Scanner, Nexter};
use std::collections::HashMap;
use std::str::FromStr;
use std::string;
use std::ops;
use std::cmp;

#[derive(Clone, PartialEq, Debug)]
enum Token {
    OParen, CParen,
    //Quote(String), QuasiQuote, UnQuote, UnQSplice,
    True, False,
    Symbol(String),
    Number(f64),
    String(String),
}

struct Tokenizer {
    src: Scanner<char>,
}

impl Nexter<Token> for Tokenizer {
    fn get_item(&mut self) -> Option<Token> {
        self.src.ignore_ws();
        let token = match self.src.next() {
            Some('(')  => Token::OParen,
            Some(')')  => Token::CParen,

            // TODO parse quoted expr
            //Some('\'') => Token::Quote,
            //Some('`')  => Token::QuasiQuote,
            //Some(',')  => match self.src.peek() {
                //Some('@') => { self.src.next(); Token::UnQSplice },
                //_ => Token::UnQuote,
            //},

            Some('"')  => {
                self.src.until_chars("\"");
                if self.src.next() != Some('"') { // consume closing quote
                    self.src.ignore();
                    return None; // drop partial string, parse as unexpected EOF
                } else {
                    let token = self.src.extract();
                    Token::String(token.iter()
                                  .take(token.len() - 2)
                                  .skip(1).cloned().collect())
                }
            },
            Some(_) => {
                self.src.until_chars(" \n\r\t)");
                let token = self.src.extract_string();
                match &token[..] {
                    "#t" => Token::True,
                    "#f" => Token::False,
                    num  => match f64::from_str(num) {
                        Ok(n) => Token::Number(n),
                        Err(_)  => Token::Symbol(token.clone())
                    }
                }
            },
            None => return None
        };
        self.src.ignore();
        Some(token)
    }
}

struct Lexer {
    output: Scanner<Token>,
}

impl ops::Deref for Lexer {
    type Target = Scanner<Token>;
    fn deref<'a>(&'a self) -> &'a Scanner<Token> { &self.output }
}

impl ops::DerefMut for Lexer {
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
    String(String),
    Symbol(String),
    Number(f64),
    True, False,
    //Quote(Box<LispExpr>),
    //QuasiQuote(Box<LispExpr>),
    //UnQuote(Box<LispExpr>),
    //UnQSplice(Box<LispExpr>),
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
            //&LispExpr::Quote(ref e) => format!("'{}", e.to_string()),
            //&LispExpr::QuasiQuote(ref e) => format!("`{}", e.to_string()),
            //&LispExpr::UnQuote(ref e) => format!(",{}", e.to_string()),
            //&LispExpr::UnQSplice(ref e) => format!(",@{}", e.to_string()),
        }
    }
}

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
                while lex.peek() != Some(Token::CParen) { // even != None
                    match Parser::parse(lex) {
                        Err(err) => return Err(err),
                        Ok(expr) => list.push(expr),
                    }
                }
                lex.next(); // get over that CParen
                Ok(LispExpr::List(list))
            },
            //Some(Token::Quote)      => Err(ParseError::NotImplemented),
            //Some(Token::QuasiQuote) => Err(ParseError::NotImplemented),
            //Some(Token::UnQuote)    => Err(ParseError::NotImplemented),
            //Some(Token::UnQSplice)  => Err(ParseError::NotImplemented),
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum EvalErr {
    UnknownVar(String),
    UnknownFunction(String),
    NotCallable,
    InvalidExpr,
    NotImplemented,
}

pub struct LispContext {
    vars: HashMap<String, LispExpr>,
    procs: HashMap<String, Box<Fn(Vec<LispExpr>) -> Result<LispExpr, EvalErr>>>,
    outer: Option<Box<LispContext>>,
}

//struct Procedure {
    //params: Vec<LispExpr>,
    //body: LispExpr,
    //env: LispContext,
//}

//impl Procedure{
    //fn new(params: Vec<LispExpr>, body: LispExpr, env: LispContext) -> Procedure{
        //Procedure{params: params, body: body, env: env}
    //}

    //fn call(&self, args: Vec<LispExpr>) -> LispExpr {
        ////args
        //self.env.eval(self.body, self.env)
    //}
//}

fn foldop<T>(op: T, args: Vec<LispExpr>) -> Result<LispExpr, EvalErr>
        where T: Fn(f64, f64) -> f64 {
    let base = match args.first() {
        Some(&LispExpr::Number(n)) => n,
        _ => return Err(EvalErr::InvalidExpr)
    };
    let mut rest = Vec::new();
    for arg in args.iter().skip(1) {
        match arg {
            &LispExpr::Number(n) => rest.push(n),
            _ => return Err(EvalErr::InvalidExpr)
        }
    }
    Ok(LispExpr::Number(rest.iter().fold(base, |ac, &item| op(ac, item))))
}

impl LispContext {
    pub fn new() -> LispContext {
        let mut procs: HashMap<String, Box<Fn(Vec<LispExpr>) -> Result<LispExpr, EvalErr>>> = HashMap::new();
        procs.insert(format!("+"), Box::new(|args| foldop(ops::Add::add, args)));
        procs.insert(format!("-"), Box::new(|args| foldop(ops::Sub::sub, args)));
        procs.insert(format!("*"), Box::new(|args| foldop(ops::Mul::mul, args)));
        procs.insert(format!("/"), Box::new(|args| foldop(ops::Div::div, args)));
        procs.insert(format!("%"), Box::new(|args| foldop(ops::Rem::rem, args)));
        //procs.insert(format!("<"), Box::new(|args| (args[0] as f64) < (args[1] as f64)));
        //procs.insert(format!("first"), Box::new(|args| args[0]));
        //procs.insert(format!("tail"), Box::new(|args| args[1..]));
        let mut vars = HashMap::new();
        //vars.insert(format!("#f"), false);
        LispContext{vars: vars, procs: procs, outer: None}
    }

    pub fn eval_str(&mut self, expr: &str) -> Result<LispExpr, EvalErr> {
        let e = Parser::parse_str(expr);
        Self::eval(&e.unwrap(), self)
    }

    pub fn eval(expr: &LispExpr, ctx: &mut LispContext) -> Result<LispExpr, EvalErr> {
        match expr {
            &LispExpr::True => Ok(LispExpr::True),
            &LispExpr::False => Ok(LispExpr::False),
            &LispExpr::String(ref s) => Ok(LispExpr::String(s.clone())),
            &LispExpr::Number(num) => Ok(LispExpr::Number(num)),
            &LispExpr::Symbol(ref sym) => match ctx.vars.get(sym) {
                Some(value) => Ok(value.clone()),
                None => Err(EvalErr::UnknownVar(sym.clone()))
            },

            //&LispExpr::Quote(_) => Err(EvalErr::NotImplemented),
            //&LispExpr::QuasiQuote(_) => Err(EvalErr::NotImplemented),
            //&LispExpr::UnQuote(_) => Err(EvalErr::NotImplemented),
            //&LispExpr::UnQSplice(_) => Err(EvalErr::NotImplemented),

            &LispExpr::List(ref list) => match list.first() {
                Some(&LispExpr::Symbol(ref pname)) => match (&(*pname)[..], list.len()) {
                    ("quote", 2)  => Ok(list[1].clone()),
                    ("if", 4)     => {
                        let (test, conseq, alt) = (&list[1], &list[2], &list[3]);
                        match Self::eval(test, ctx) {
                            Err(err) => Err(err),
                            Ok(LispExpr::Symbol(ref s)) if s == "#f "=> Self::eval(alt, ctx),
                            Ok(_) => Self::eval(conseq, ctx),
                        }
                    },
                    ("define", 3) => {
                        match (&list[1], &list[2]) {
                            (&LispExpr::Symbol(ref var), val) => match Self::eval(val, ctx) {
                                Ok(expr) => { ctx.vars.insert(var.clone(), expr.clone()); Ok(expr) }, // TODO check type to insert in proper struct
                                Err(err) => Err(err)
                            },
                            _ => Err(EvalErr::InvalidExpr)
                        }
                    },
                    (_, _) => {
                        let mut args = Vec::new();
                        for arg in list.iter().skip(1) {
                            match Self::eval(arg, ctx) {
                                Err(err) => return Err(err),
                                Ok(expr) => args.push(expr)
                            }
                        }
                        match ctx.procs.get(pname) {
                            Some(procedure) => procedure(args),
                            None => Err(EvalErr::UnknownFunction(pname.clone()))
                        }
                    },
                },
                Some(&LispExpr::List(_)) => { Err(EvalErr::NotImplemented) },
                _ => Err(EvalErr::NotCallable) // list.first is None or LispExpr::Number

            }
        }
    }
}

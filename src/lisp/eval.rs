use lisp::{Lexer, Token, Parser};
use lisp::{Procs, ctx_globals};

use std::collections::HashMap;
use std::iter::FromIterator;
use std::string;


#[derive(Clone, PartialEq, PartialOrd, Debug)]
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
    Proc(Procedure),
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
            _ => format!("%unknown%")
            //&LispExpr::Quote(ref e) => format!("'{}", e.to_string()),
            //&LispExpr::QuasiQuote(ref e) => format!("`{}", e.to_string()),
            //&LispExpr::UnQuote(ref e) => format!(",{}", e.to_string()),
            //&LispExpr::UnQSplice(ref e) => format!(",@{}", e.to_string()),
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

#[derive(Clone)]
pub struct LispContext {
    vars: HashMap<String, LispExpr>,
    procs: Procs,
    outer: Option<Box<LispContext>>,
}

#[derive(PartialOrd, PartialEq, Clone, Debug)]
struct Procedure {
    params: Vec<String>,
    body: Box<LispExpr>,
    env: LispContext,
}

impl Procedure{
    fn new(params: Vec<String>, body: LispExpr, env: LispContext) -> Procedure {
        Procedure{params: params, body: Box::new(body), env: env}
    }

    fn call(&self, args: Vec<LispExpr>) -> Result<LispExpr, EvalErr> {
        //let mut env = LispContext::nested(&self.params, &args, Some(Box::new(self.env.clone())));
        let mut env = LispContext::nested(&self.params, &args, None);
        LispContext::eval(&self.body, &mut env)
    }
}

impl LispContext {
    pub fn new() -> LispContext {
        let vars = HashMap::new();
        LispContext{vars: vars, procs: ctx_globals(), outer: None}
    }

    fn nested(params: &Vec<String>,
              args: &Vec<LispExpr>,
              outer: Option<Box<LispContext>>) -> LispContext {
        let vars = HashMap::from_iter(params.iter().cloned().zip(args.iter().cloned()));
        LispContext{
            vars: vars.clone(),
            procs: ctx_globals(),
            outer: outer
        }
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
                    ("lambda", 3) => {
                        let body = &list[2];
                        let mut vars = Vec::new();
                        if let LispExpr::List(ref vs) = list[1] {
                            for v in vs.iter() {
                                match &v {

                                }
                            }
                        }

                        Ok(LispExpr::Proc(Procedure::new(vars, body.clone(), ctx.clone())))
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

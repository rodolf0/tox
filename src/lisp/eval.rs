use lisp::{Lexer, Token, Parser, LispExpr, Procedure};
use lisp::{Fp, ctx_globals};

use std::collections::HashMap;
use std::iter::FromIterator;
use std::rc::Rc;

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
    outer: Option<Rc<LispContext>>,
}

impl LispContext {
    pub fn new() -> LispContext {
        let vars = HashMap::new();
        LispContext{vars: vars, outer: None}
    }

    pub fn nested(params: &Vec<String>,
              args: &Vec<LispExpr>,
              outer: Option<Rc<LispContext>>) -> LispContext {
        let vars = HashMap::from_iter(params.iter().cloned().zip(args.iter().cloned()));
        LispContext{
            vars: vars.clone(),
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
            &LispExpr::Proc(ref p) => Ok(LispExpr::Proc(p.clone())),
            &LispExpr::Symbol(ref sym) => match ctx.vars.get(sym) { // TODO: env.find
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
                            Ok(LispExpr::False) => Self::eval(alt, ctx),
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
                                match v {
                                    &LispExpr::Symbol(ref x) => vars.push(x.clone()),
                                    _ => return Err(EvalErr::InvalidExpr)
                                }
                            }
                        }
                        Ok(LispExpr::Proc(Box::new(Procedure::new(vars, body.clone(), Rc::new(ctx.clone())))))
                    },
                    (_, _) => {
                        let mut args = Vec::new();
                        for arg in list.iter().skip(1) {
                            match Self::eval(arg, ctx) {
                                Err(err) => return Err(err),
                                Ok(expr) => args.push(expr)
                            }
                        }
                        match ctx.vars.get(pname) {
                            Some(&LispExpr::Proc(ref pr)) => pr.call(&args),
                            _ => Err(EvalErr::UnknownFunction(pname.clone()))
                        }
                    },
                },
                Some(&LispExpr::List(_)) => { Err(EvalErr::NotImplemented) },
                _ => Err(EvalErr::NotCallable) // list.first is None or LispExpr::Number

            }
        }
    }
}

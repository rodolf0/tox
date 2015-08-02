use lisp::{Parser, ParseError, LispExpr, Procedure};
use lisp::builtins;

use std::collections::HashMap;
use std::iter::FromIterator;
use std::cell::RefCell;
use std::rc::Rc;

macro_rules! check {
    ($argcheck:expr, $err:expr) => {
        if ! $argcheck { return Err($err); }
    }
}

#[derive(PartialEq, Debug)]
pub enum EvalErr {
    ParseError(ParseError),
    UnknownSym(String),
    UnknownFunction(String),
    NotCallable,
    InvalidExpr,
    NotImplemented,
}

#[derive(Clone)]
pub struct LispContext {
    syms: RefCell<HashMap<String, LispExpr>>,
    outer: Option<Rc<LispContext>>,
}

impl LispContext {
    pub fn new() -> LispContext {
        LispContext{syms: RefCell::new(builtins()), outer: None}
    }

    pub fn nested(params: Vec<String>, args: Vec<LispExpr>,
                  outer: Option<Rc<LispContext>>) -> LispContext {
        LispContext{
            syms: RefCell::new(
                    HashMap::from_iter(
                        params.into_iter().zip(args.into_iter()))),
            outer: outer
        }
    }

    pub fn lookup(&self, sym: &str) -> Option<&LispContext> {
        if self.syms.borrow().contains_key(sym) {
            Some(self)
        } else if let Some(ref otx) = self.outer {
            otx.lookup(sym)
        } else {
            None
        }
    }

    pub fn eval_str(expr: &str) -> Result<LispExpr, EvalErr> {
        match Parser::parse_str(expr) {
            Ok(ref expr) => Self::eval(expr, &Rc::new(LispContext::new())),
            Err(err) => Err(EvalErr::ParseError(err))
        }
    }

    pub fn eval(expr: &LispExpr, ctx: &Rc<LispContext>) -> Result<LispExpr, EvalErr> {
        match expr {
            &LispExpr::True => Ok(LispExpr::True),
            &LispExpr::False => Ok(LispExpr::False),
            &LispExpr::String(ref s) => Ok(LispExpr::String(s.clone())),
            &LispExpr::Number(num) => Ok(LispExpr::Number(num)),
            &LispExpr::Proc(ref p) => Ok(LispExpr::Proc(p.clone())),
            &LispExpr::Symbol(ref sym) => match ctx.lookup(sym) {
                Some(cx) => Ok(cx.syms.borrow().get(sym).unwrap().clone()),
                None => Err(EvalErr::UnknownSym(sym.clone()))
            },

            &LispExpr::Quote(ref expr) => Ok(*expr.clone()),
            &LispExpr::QuasiQuote(_) => Err(EvalErr::NotImplemented),
            &LispExpr::UnQuote(_) => Err(EvalErr::NotImplemented),
            &LispExpr::UnQSplice(_) => Err(EvalErr::NotImplemented),

            &LispExpr::List(ref list) => match list.first() {
                Some(&LispExpr::Symbol(ref first)) => {
                    match &first[..] {
                        "quote" => {
                            check!(list.len() == 2, EvalErr::InvalidExpr);
                            Ok(list[1].clone())
                        },
                        "if" => {
                            check!(list.len() == 4, EvalErr::InvalidExpr);
                            let (test, conseq, alt) = (&list[1], &list[2], &list[3]);
                            match Self::eval(test, ctx) {
                                Ok(LispExpr::False) => Self::eval(alt, ctx),
                                Ok(_) => Self::eval(conseq, ctx),
                                Err(err) => Err(err)
                            }
                        },
                        "define" => {
                            check!(list.len() == 3, EvalErr::InvalidExpr);
                            match (&list[1], &list[2]) {
                                (&LispExpr::Symbol(ref var), expr) => match Self::eval(expr, ctx) {
                                    Ok(value) => {
                                        ctx.syms.borrow_mut().insert(var.clone(), value);
                                        // TODO: should return None
                                        Ok(LispExpr::True)
                                    },
                                    Err(err) => Err(err)
                                },
                                _ => Err(EvalErr::InvalidExpr)
                            }
                        },
                        "set!" => {
                            check!(list.len() == 3, EvalErr::InvalidExpr);
                            match (&list[1], &list[2]) {
                                (&LispExpr::Symbol(ref var), expr) => match Self::eval(expr, ctx) {
                                    Ok(value) => {
                                        match ctx.lookup(var) {
                                            Some(cx) => cx.syms.borrow_mut().insert(var.clone(), value),
                                            None => return Err(EvalErr::UnknownSym(var.clone()))
                                        };
                                        // TODO: should return None
                                        Ok(LispExpr::True)
                                    },
                                    Err(err) => Err(err)
                                },
                                _ => Err(EvalErr::InvalidExpr)
                            }
                        },
                        "lambda" => {
                            check!(list.len() == 3, EvalErr::InvalidExpr);
                            let mut vars = Vec::new();
                            match list[1] {
                                LispExpr::List(ref varlist) => for var in varlist.iter() {
                                    match var {
                                        &LispExpr::Symbol(ref v) => vars.push(v.clone()),
                                        _ => return Err(EvalErr::InvalidExpr)
                                    }
                                },
                                _ => return Err(EvalErr::InvalidExpr)
                            };
                            let body = &list[2];
                            Ok(LispExpr::Proc(Rc::new(Procedure::new(vars, body.clone(), ctx.clone()))))
                        },
                        _ => {
                            let mut args = Vec::new();
                            for arg in list.iter().skip(1) {
                                match Self::eval(arg, ctx) {
                                    Err(err) => return Err(err),
                                    Ok(expr) => args.push(expr)
                                }
                            }
                            let opfirst = match ctx.lookup(first) {
                                Some(cx) => cx.syms.borrow().get(first).cloned(),
                                None => return Err(EvalErr::UnknownFunction(first.clone()))
                            };
                            match opfirst {
                                Some(LispExpr::Proc(pr)) => pr.call(args),
                                _ => Err(EvalErr::UnknownFunction(first.clone()))
                            }
                        }
                    }
                },
                Some(&LispExpr::List(ref first)) => {
                    let mut expr = match Self::eval(&LispExpr::List(first.clone()), ctx) {
                        Err(err) => return Err(err),
                        Ok(sym) => vec![sym]
                    };
                    expr.extend(list.iter().skip(1).cloned());
                    Self::eval(&LispExpr::List(expr), ctx)
                },
                Some(&LispExpr::Proc(ref first)) => {
                    let mut args = Vec::new();
                    for arg in list.iter().skip(1) {
                        match Self::eval(arg, ctx) {
                            Err(err) => return Err(err),
                            Ok(expr) => args.push(expr)
                        }
                    }
                    first.call(args)
                },
                None => Ok(LispExpr::List(Vec::new())),
                _ => Err(EvalErr::InvalidExpr)
            }
        }
    }
}

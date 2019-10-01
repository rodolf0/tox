use crate::eval::{EvalErr, LispContext};
use crate::parser::LispExpr;
use std::{fmt, cmp};
use std::rc::Rc;

pub type Fp = Rc<dyn Fn(&Vec<LispExpr>) -> Result<LispExpr, EvalErr>>;

enum Body {
    Lisp(LispExpr),
    Builtin(Fp),
}

pub struct Procedure {
    params: Vec<String>,
    body: Body,
    env: Option<Rc<LispContext>>,
}

impl cmp::PartialEq for Procedure {
    fn eq(&self, _other: &Procedure) -> bool { false }
}

impl cmp::PartialOrd for Procedure {
    fn partial_cmp(&self, _other: &Procedure) -> Option<cmp::Ordering> { None }
}

impl fmt::Debug for Procedure {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(lambda ({:?}) ...)", self.params)
    }
}

impl Procedure {
    pub fn new(params: Vec<String>, body: LispExpr, env: Rc<LispContext>) -> Procedure {
        Procedure{params: params, body: Body::Lisp(body), env: Some(env)}
    }

    pub fn builtin(fp: Fp) -> Procedure {
        Procedure{params: Vec::new(), body: Body::Builtin(fp), env: None}
    }

    pub fn call(&self, args: Vec<LispExpr>) -> Result<LispExpr, EvalErr> {
        match self.body {
            Body::Builtin(ref fp) => fp(&args),
            Body::Lisp(ref expr) => {
                let env = LispContext::nested(self.params.clone(), args, self.env.clone());
                LispContext::eval(expr, &Rc::new(env))
            }
        }
    }
}

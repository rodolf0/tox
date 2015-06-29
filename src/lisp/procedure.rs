use lisp::{EvalErr, LispExpr, LispContext};
use std::{fmt, cmp, clone};
use std::rc::Rc;

#[derive(Clone)]
pub struct Procedure {
    params: Vec<String>,
    body: LispExpr,
    env: Rc<LispContext>,
}

impl cmp::PartialEq for Procedure {
    fn eq(&self, other: &Procedure) -> bool { false }
}

impl cmp::PartialOrd for Procedure {
    fn partial_cmp(&self, other: &Procedure) -> Option<cmp::Ordering> { None }
}

impl fmt::Debug for Procedure {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "lambda ({:?})", self.params)
    }
}

// TODO: needed?
//impl clone::Clone for Procedure {
    //fn clone(&self) -> Self {
        //Procedure{
            //params: self.params.clone(),
            //body: self.body.clone(),
            //env: self.env.clone()
        //}
    //}
//}

impl Procedure {
    pub fn new(params: Vec<String>, body: LispExpr, env: Rc<LispContext>) -> Procedure {
        Procedure{params: params, body: body, env: env}
    }

    pub fn call(&self, args: &Vec<LispExpr>) -> Result<LispExpr, EvalErr> {
        //let mut env = LispContext::nested(&self.params, &args, Some(Box::new(self.env.clone())));
        let mut env = LispContext::nested(&self.params, args, None);
        LispContext::eval(&self.body, &mut env)
    }
}


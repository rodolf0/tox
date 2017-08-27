#![deny(warnings)]

use lox_scanner::TT;
use lox_parser::{Expr, Stmt};
use lox_environment::Environment;
use std::cell::RefCell;
use std::rc::Rc;
use std::fmt;


#[derive(Clone,Debug,PartialEq)]
pub enum V {
    Nil,
    Num(f64),
    Bool(bool),
    Str(String),
}

impl V {
    fn is_truthy(&self) -> bool {
        match self {
            &V::Nil => false,
            &V::Bool(ref b) => *b,
            _ => true
        }
    }

    fn num(&self) -> Result<f64, String> {
        match self {
            &V::Num(ref n) => Ok(*n),
            o => Err(format!("expected V::Num, found {:?}", o))
        }
    }
}

impl fmt::Display for V {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &V::Nil => write!(f, "nil"),
            &V::Bool(ref b) => write!(f, "{}", b),
            &V::Num(ref n) => write!(f, "{}", n),
            &V::Str(ref s) => write!(f, "\"{}\"", s),
        }
    }
}

type EvalResult = Result<V, String>;

pub struct LoxInterpreter {
    env: Rc<RefCell<Environment>>,
    errors: bool,
}

impl LoxInterpreter {
    pub fn new() -> Self {
        LoxInterpreter{
            env: Rc::new(RefCell::new(Environment::new(None))), errors: false}
    }

    fn eval(&mut self, expr: &Expr) -> EvalResult {
        match expr {
            &Expr::Nil => Ok(V::Nil),
            &Expr::Num(n) => Ok(V::Num(n)),
            &Expr::Str(ref s) => Ok(V::Str(s.to_string())),
            &Expr::Bool(ref b) => Ok(V::Bool(*b)),
            &Expr::Grouping(ref expr) => self.eval(&*expr),
            &Expr::Unary(ref op, ref expr) => {
                let expr = self.eval(expr)?;
                match op.token {
                    TT::MINUS => Ok(V::Num(-expr.num()?)),
                    TT::BANG => Ok(V::Bool(!expr.is_truthy())),
                    _ => unreachable!("LoxIntepreter: bad Unary op {:?}", op)
                }
            },
            &Expr::Binary(ref lhs, ref op, ref rhs) => {
                let lhs = self.eval(lhs)?;
                let rhs = self.eval(rhs)?;
                match op.token {
                    TT::SLASH => Ok(V::Num(lhs.num()? / rhs.num()?)),
                    TT::STAR => Ok(V::Num(lhs.num()? * rhs.num()?)),
                    TT::MINUS => Ok(V::Num(lhs.num()? - rhs.num()?)),
                    TT::PLUS => match (&lhs, &rhs) {
                        (&V::Num(ref l), &V::Num(ref r)) => Ok(V::Num(l + r)),
                        (&V::Str(ref l), &V::Str(ref r)) =>
                            Ok(V::Str(format!("{}{}", l, r))),
                        (&V::Str(ref l), ref other) =>
                            Ok(V::Str(format!("{}{}", l, other))),
                        (ref other, &V::Str(ref r)) =>
                            Ok(V::Str(format!("{}{}", other, r))),
                        _ => Err(format!("can't {:?} + {:?}", lhs, rhs))
                    },
                    TT::GT => Ok(V::Bool(lhs.num()? > rhs.num()?)),
                    TT::GE => Ok(V::Bool(lhs.num()? >= rhs.num()?)),
                    TT::LT => Ok(V::Bool(lhs.num()? < rhs.num()?)),
                    TT::LE => Ok(V::Bool(lhs.num()? <= rhs.num()?)),
                    TT::EQ => Ok(V::Bool(lhs == rhs)),
                    TT::NE => Ok(V::Bool(lhs != rhs)),
                    _ => unreachable!("LoxIntepreter: bad Binary op {:?}", op)
                }
            },
            &Expr::Var(ref var) => self.env.borrow().get(var),
            &Expr::Assign(ref var, ref expr) => {
                let value = self.eval(expr)?;
                self.env.borrow_mut().assign(var.clone(), value)
            }
        }
    }

    fn exec_block(&mut self, statements: &Vec<Stmt>,
                  env: Rc<RefCell<Environment>>) -> Option<String> {
        let prev_env = self.env.clone();
        self.env = env;
        for stmt in statements {
            if let Some(err) = self.execute(stmt) {
                // restore interpreter's env
                self.env = prev_env;
                return Some(err);
            }
        }
        // restore interpreter's env
        self.env = prev_env;
        None
    }

    fn execute(&mut self, stmt: &Stmt) -> Option<String> {
        match stmt {
            &Stmt::Expr(ref expr) => if let Err(err) = self.eval(expr) {
                return Some(err);
            },
            &Stmt::Print(ref expr) => match self.eval(expr) {
                Ok(value) => println!("{}", value),
                Err(err) => return Some(err)
            },
            &Stmt::Var(ref name, ref init) => {
                let value = match self.eval(init) {
                    Ok(value) => value,
                    Err(err) => return Some(err)
                };
                self.env.borrow_mut().define(name.to_string(), value);
            },
            &Stmt::Block(ref stmts) => {
                let curenv = Environment::new(Some(self.env.clone()));
                return self.exec_block(stmts, Rc::new(RefCell::new(curenv)));
            },
            &Stmt::If(ref expr, ref then_branch, ref else_branch) => {
                let condition = match self.eval(expr) {
                    Ok(cond) => cond,
                    Err(err) => return Some(err)
                };
                return match condition.is_truthy() {
                    true => self.execute(then_branch),
                    false => match else_branch {
                        &Some(ref eb) => self.execute(eb),
                        _ => None
                    }
                };
            },
        }
        None
    }

    pub fn interpret(&mut self, statements: &Vec<Stmt>) -> Option<String> {
        for stmt in statements {
            if let Some(err) = self.execute(stmt) {
                self.errors = true;
                return Some(err);
            }
        }
        None
    }
}

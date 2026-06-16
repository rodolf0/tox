use crate::eval::EvalErr;
use crate::parser::Expr;
use crate::procedure::Procedure;
use std::collections::HashMap;
use std::rc::Rc;
use std::{cmp, ops};

macro_rules! builtin {
    ($fnexpr:expr) => {
        Expr::Proc(Rc::new(Procedure::builtin(Rc::new($fnexpr))))
    };
}

fn foldop<T>(op: T, args: &Vec<Expr>) -> Result<Expr, EvalErr>
where
    T: Fn(f64, f64) -> f64,
{
    let base = match args.first() {
        Some(&Expr::Number(n)) => n,
        _ => return Err(EvalErr::InvalidExpr),
    };
    let mut rest = Vec::new();
    for arg in args.iter().skip(1) {
        match arg {
            &Expr::Number(n) => rest.push(n),
            _ => return Err(EvalErr::InvalidExpr),
        }
    }
    Ok(Expr::Number(
        rest.iter().fold(base, |ac, &item| op(ac, item)),
    ))
}

fn foldcmp<T>(op: T, args: &Vec<Expr>) -> Result<Expr, EvalErr>
where
    T: Fn(&Expr, &Expr) -> bool,
{
    if args.len() < 2 {
        return Err(EvalErr::InvalidExpr);
    }
    match args[..].windows(2).all(|win| op(&win[0], &win[1])) {
        true => Ok(Expr::True),
        false => Ok(Expr::False),
    }
}

fn first(args: &Vec<Expr>) -> Result<Expr, EvalErr> {
    match args.first() {
        Some(&Expr::List(ref l)) if l.len() > 0 => Ok(l.first().unwrap().clone()),
        _ => Err(EvalErr::InvalidExpr),
    }
}

fn tail(args: &Vec<Expr>) -> Result<Expr, EvalErr> {
    match args.first() {
        Some(&Expr::List(ref l)) => Ok(Expr::List(l.iter().skip(1).cloned().collect())),
        _ => Err(EvalErr::InvalidExpr),
    }
}

fn cons(args: &Vec<Expr>) -> Result<Expr, EvalErr> {
    if args.len() != 2 {
        return Err(EvalErr::InvalidExpr);
    }
    match args[1] {
        Expr::List(ref b) => {
            let mut a = vec![args[0].clone()];
            a.extend(b.clone());
            Ok(Expr::List(a))
        }
        _ => Ok(Expr::List(vec![args[0].clone(), args[1].clone()])),
    }
}

pub fn builtins() -> HashMap<String, Expr> {
    let mut builtins: HashMap<String, Expr> = HashMap::new();

    builtins.insert(format!("+"), builtin!(|args| foldop(ops::Add::add, &args)));
    builtins.insert(
        format!("-"),
        builtin!(|args| match args.len() {
            1 => match args.first() {
                // special handling of negation op
                Some(&Expr::Number(n)) => Ok(Expr::Number(-n)),
                _ => Err(EvalErr::InvalidExpr),
            },
            _ => foldop(ops::Sub::sub, &args),
        }),
    );
    builtins.insert(format!("*"), builtin!(|args| foldop(ops::Mul::mul, &args)));
    builtins.insert(format!("/"), builtin!(|args| foldop(ops::Div::div, &args)));
    builtins.insert(format!("%"), builtin!(|args| foldop(ops::Rem::rem, &args)));
    builtins.insert(
        format!("<"),
        builtin!(|args| foldcmp(cmp::PartialOrd::lt, &args)),
    );
    builtins.insert(
        format!("<="),
        builtin!(|args| foldcmp(cmp::PartialOrd::le, &args)),
    );
    builtins.insert(
        format!(">"),
        builtin!(|args| foldcmp(cmp::PartialOrd::gt, &args)),
    );
    builtins.insert(
        format!(">="),
        builtin!(|args| foldcmp(cmp::PartialOrd::ge, &args)),
    );
    builtins.insert(
        format!("="),
        builtin!(|args| foldcmp(cmp::PartialEq::eq, &args)),
    );
    builtins.insert(
        format!("!="),
        builtin!(|args| foldcmp(cmp::PartialEq::ne, &args)),
    );
    builtins.insert(format!("first"), builtin!(|args| first(&args)));
    builtins.insert(format!("tail"), builtin!(|args| tail(&args)));
    builtins.insert(format!("cons"), builtin!(|args| cons(&args)));
    builtins.insert(
        format!("list"),
        builtin!(|args| Ok(Expr::List(args.clone()))),
    );
    builtins.insert(
        format!("length"),
        builtin!(|args| match args.first() {
            Some(&Expr::String(ref s)) => Ok(Expr::Number(s.len() as f64)),
            Some(&Expr::List(ref list)) => Ok(Expr::Number(list.len() as f64)),
            _ => Err(EvalErr::InvalidExpr),
        }),
    );
    builtins.insert(
        format!("number?"),
        builtin!(|args| match args.first() {
            Some(&Expr::Number(_)) => Ok(Expr::True),
            _ => Ok(Expr::False),
        }),
    );
    builtins.insert(
        format!("list?"),
        builtin!(|args| match args.first() {
            Some(&Expr::List(_)) => Ok(Expr::True),
            _ => Ok(Expr::False),
        }),
    );
    builtins.insert(
        format!("symbol?"),
        builtin!(|args| match args.first() {
            Some(&Expr::Symbol(_)) => Ok(Expr::True),
            _ => Ok(Expr::False),
        }),
    );
    builtins.insert(
        format!("procedure?"),
        builtin!(|args| match args.first() {
            Some(&Expr::Proc(_)) => Ok(Expr::True),
            _ => Ok(Expr::False),
        }),
    );
    builtins.insert(
        format!("null?"),
        builtin!(|args| match args.first() {
            Some(&Expr::List(ref list)) if list.len() == 0 => Ok(Expr::True),
            _ => Ok(Expr::False),
        }),
    );
    builtins.insert(
        format!("begin"),
        builtin!(|args| match args.last() {
            Some(expr) => Ok(expr.clone()),
            _ => Err(EvalErr::InvalidExpr),
        }),
    );
    builtins
}

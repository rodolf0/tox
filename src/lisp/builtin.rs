use lisp::{LispExpr, EvalErr, Procedure};
use std::collections::HashMap;
use std::{ops, cmp};
use std::rc::Rc;

macro_rules! builtin {
    ($fnexpr:expr) => {
        LispExpr::Proc(Rc::new(Procedure::builtin(Rc::new($fnexpr))))
    }
}

fn foldop<T>(op: T, args: &Vec<LispExpr>) -> Result<LispExpr, EvalErr>
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

fn foldcmp<T>(op: T, args: &Vec<LispExpr>) -> Result<LispExpr, EvalErr>
        where T: Fn(&LispExpr, &LispExpr) -> bool {
    if args.len() < 2 {
        return Err(EvalErr::InvalidExpr);
    }
    match args[..].windows(2).all(|win| op(&win[0], &win[1])) {
        true => Ok(LispExpr::True),
        false => Ok(LispExpr::False),
    }
}

fn first(args: &Vec<LispExpr>) -> Result<LispExpr, EvalErr> {
    match args.first() {
        Some(&LispExpr::List(ref l)) if l.len() > 0 =>
            Ok(l.first().unwrap().clone()),
        _ => Err(EvalErr::InvalidExpr)
    }
}

fn tail(args: &Vec<LispExpr>) -> Result<LispExpr, EvalErr> {
    match args.first() {
        Some(&LispExpr::List(ref l)) =>
            Ok(LispExpr::List(l.iter().skip(1).cloned().collect())),
        _ => Err(EvalErr::InvalidExpr)
    }
}

fn cons(args: &Vec<LispExpr>) -> Result<LispExpr, EvalErr> {
    if args.len() != 2 { return Err(EvalErr::InvalidExpr); }
    match args[1] {
        LispExpr::List(ref b) => {
            let mut a = vec![args[0].clone()];
            a.extend(b.clone());
            Ok(LispExpr::List(a))
        },
        _ => Ok(LispExpr::List(vec![args[0].clone(), args[1].clone()]))
    }
}

pub fn builtins() -> HashMap<String, LispExpr> {
    let mut builtins: HashMap<String, LispExpr> = HashMap::new();

    builtins.insert(format!("+"), builtin!(|args| foldop(ops::Add::add, &args)));
    builtins.insert(format!("-"), builtin!(|args| match args.len() {
        1 => match args.first() { // special handling of negation op
            Some(&LispExpr::Number(n)) => Ok(LispExpr::Number(-n)),
            _ => Err(EvalErr::InvalidExpr)
        },
        _ => foldop(ops::Sub::sub, &args)
    }));
    builtins.insert(format!("*"), builtin!(|args| foldop(ops::Mul::mul, &args)));
    builtins.insert(format!("/"), builtin!(|args| foldop(ops::Div::div, &args)));
    builtins.insert(format!("%"), builtin!(|args| foldop(ops::Rem::rem, &args)));
    builtins.insert(format!("<"), builtin!(|args| foldcmp(cmp::PartialOrd::lt, &args)));
    builtins.insert(format!("<="), builtin!(|args| foldcmp(cmp::PartialOrd::le, &args)));
    builtins.insert(format!(">"), builtin!(|args| foldcmp(cmp::PartialOrd::gt, &args)));
    builtins.insert(format!(">="), builtin!(|args| foldcmp(cmp::PartialOrd::ge, &args)));
    builtins.insert(format!("="), builtin!(|args| foldcmp(cmp::PartialEq::eq, &args)));
    builtins.insert(format!("!="), builtin!(|args| foldcmp(cmp::PartialEq::ne, &args)));
    builtins.insert(format!("first"), builtin!(|args| first(&args)));
    builtins.insert(format!("tail"), builtin!(|args| tail(&args)));
    builtins.insert(format!("cons"), builtin!(|args| cons(&args)));
    builtins.insert(format!("list"), builtin!(|args| Ok(LispExpr::List(args.clone()))));
    builtins.insert(format!("length"), builtin!(|args| match args.first() {
        Some(&LispExpr::List(ref list)) => Ok(LispExpr::Number(list.len() as f64)),
        _ => Err(EvalErr::InvalidExpr)
    }));
    builtins.insert(format!("number?"), builtin!(|args| match args.first() {
        Some(&LispExpr::Number(_)) => Ok(LispExpr::True), _ => Ok(LispExpr::False)
    }));
    builtins.insert(format!("list?"), builtin!(|args| match args.first() {
        Some(&LispExpr::List(_)) => Ok(LispExpr::True), _ => Ok(LispExpr::False)
    }));
    builtins.insert(format!("symbol?"), builtin!(|args| match args.first() {
        Some(&LispExpr::Symbol(_)) => Ok(LispExpr::True), _ => Ok(LispExpr::False)
    }));
    builtins.insert(format!("procedure?"), builtin!(|args| match args.first() {
        Some(&LispExpr::Proc(_)) => Ok(LispExpr::True), _ => Ok(LispExpr::False)
    }));
    builtins.insert(format!("null?"), builtin!(|args| match args.first() {
        Some(&LispExpr::List(ref list)) if list.len() == 0 => Ok(LispExpr::True),
        _ => Ok(LispExpr::False)
    }));
    builtins
}

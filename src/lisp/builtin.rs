use lisp::{LispExpr, EvalErr, Fp, Procedure};
use std::collections::HashMap;
use std::ops;
use std::cmp;
use std::rc::Rc;

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

// TODO: revise

fn first(args: &Vec<LispExpr>) -> Result<LispExpr, EvalErr> {
    match args.first() {
        Some(&LispExpr::List(ref l)) if l.len() > 0 => Ok(l.first().unwrap().clone()),
        _ => Err(EvalErr::InvalidExpr)
    }
}

fn tail(args: &Vec<LispExpr>) -> Result<LispExpr, EvalErr> {
    match args.first() {
        Some(&LispExpr::List(ref l)) => Ok(LispExpr::List(l.iter().skip(1).cloned().collect())),
        _ => Err(EvalErr::InvalidExpr)
    }
}

fn cons(args: &Vec<LispExpr>) -> Result<LispExpr, EvalErr> {
    if args.len() != 2 {
        return Err(EvalErr::InvalidExpr);
    }
    match args[1] { // TODO: what about empty lists, they should be cleared
        LispExpr::List(ref b) => {
            let mut a = vec![args[0].clone()];
            a.extend(b.clone());
            Ok(LispExpr::List(a))
        },
        _ => Ok(LispExpr::List(vec![args[0].clone(), args[1].clone()]))
    }
}

pub fn builtins() -> HashMap<String, LispExpr> {
    let mut procs: HashMap<String, LispExpr> = HashMap::new();

    let p = Procedure::builtin(Rc::new(|args| foldop(ops::Add::add, &args)));
    procs.insert(format!("+"), LispExpr::Proc(Box::new(p)));

    //procs.insert(format!("+"), Box::new(|args| foldop(ops::Add::add, &args)));
    //procs.insert(format!("-"), Box::new(|args| match args.len() {
        //1 => match args.first() { // special handling of negation op
            //Some(&LispExpr::Number(n)) => Ok(LispExpr::Number(-n)),
            //_ => Err(EvalErr::InvalidExpr)
        //},
        //_ => foldop(ops::Sub::sub, &args)
    //}));
    //procs.insert(format!("*"), Box::new(|args| foldop(ops::Mul::mul, &args)));
    //procs.insert(format!("/"), Box::new(|args| foldop(ops::Div::div, &args)));
    //procs.insert(format!("%"), Box::new(|args| foldop(ops::Rem::rem, &args)));
    //procs.insert(format!("<"), Box::new(|args| foldcmp(cmp::PartialOrd::lt, &args)));
    //procs.insert(format!("<="), Box::new(|args| foldcmp(cmp::PartialOrd::le, &args)));
    //procs.insert(format!(">"), Box::new(|args| foldcmp(cmp::PartialOrd::gt, &args)));
    //procs.insert(format!(">="), Box::new(|args| foldcmp(cmp::PartialOrd::ge, &args)));
    //procs.insert(format!("="), Box::new(|args| foldcmp(cmp::PartialEq::eq, &args)));
    //procs.insert(format!("!="), Box::new(|args| foldcmp(cmp::PartialEq::ne, &args)));
    //procs.insert(format!("first"), Box::new(|args| first(&args)));
    //procs.insert(format!("tail"), Box::new(|args| tail(&args)));
    //procs.insert(format!("cons"), Box::new(|args| cons(&args)));
    //procs.insert(format!("list"), Box::new(|args| Ok(LispExpr::List(args.clone()))));
    procs
}

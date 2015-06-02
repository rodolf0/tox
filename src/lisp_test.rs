use lisp::{Parser, LispExpr, Atom};

#[test]
fn test_lisp1() {
    let p = Parser::parse_str("(begin (define r 10) (* pi (* r r)))");
    let r = LispExpr::List(vec![
        LispExpr::Atom(Atom::Symbol(format!("begin"))),
        LispExpr::List(vec![
            LispExpr::Atom(Atom::Symbol(format!("define"))),
            LispExpr::Atom(Atom::Symbol(format!("r"))),
            LispExpr::Atom(Atom::Number(10.0)),
        ]),
        LispExpr::List(vec![
            LispExpr::Atom(Atom::Symbol(format!("*"))),
            LispExpr::Atom(Atom::Symbol(format!("pi"))),
            LispExpr::List(vec![
                LispExpr::Atom(Atom::Symbol(format!("*"))),
                LispExpr::Atom(Atom::Symbol(format!("r"))),
                LispExpr::Atom(Atom::Symbol(format!("r"))),
            ]),
        ]),
    ]);
    assert_eq!(p.unwrap(), r);
}

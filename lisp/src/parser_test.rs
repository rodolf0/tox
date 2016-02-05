use parser::{Parser, LispExpr};

#[test]
fn test_lisp1() {
    let p = Parser::parse_str("(begin (define r 10) (* pi (* r r)))");
    let r = LispExpr::List(vec![
        LispExpr::Symbol(format!("begin")),
        LispExpr::List(vec![
            LispExpr::Symbol(format!("define")),
            LispExpr::Symbol(format!("r")),
            LispExpr::Number(10.0),
        ]),
        LispExpr::List(vec![
            LispExpr::Symbol(format!("*")),
            LispExpr::Symbol(format!("pi")),
            LispExpr::List(vec![
                LispExpr::Symbol(format!("*")),
                LispExpr::Symbol(format!("r")),
                LispExpr::Symbol(format!("r")),
            ]),
        ]),
    ]);
    assert_eq!(p.unwrap(), r);
}

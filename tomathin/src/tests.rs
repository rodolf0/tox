#[test]
fn xx() {
    let input = r#"FindRoot[Sum[360, Sum[a, b]], List["1, 2, 3"]]"#;
    let tok = crate::tokenizer::Tokenizer::new(input.chars());
    let x = crate::parser::parser().unwrap();

    println!("{:?}", x(tok).unwrap());
}

fn convert(t: crate::parser::T) -> crate::Expr {
    use crate::parser::T;
    use crate::Expr;
    match t {
        T::Expr(h, args) => {
            let mut cargs = Vec::new();
            for a in args {
                cargs.push(convert(a));
            }
            Expr::Expr(h, cargs)
        }
        T::Symbol(x) => Expr::Symbol(x),
        T::String(s) => Expr::String(s),
        T::Number(n) => Expr::Number(n),
        other => panic!("convert failed on '{:?}'", other),
    }
}

#[test]
fn xx2() {
    // let input = r#"ReplaceAll[Plus[x, Times[2, x]], Rule[x, 3]]"#;
    // let input = r#"ReplaceAll[Plus[x, Times[2, x]], Rule[Times[2, x], 3]]"#;
    let input = r#"
        ReplaceAll[
            Plus[x, Times[2, x]],
            List[
                Rule[Times[2, x], 3],
                Rule[Plus[x, 3], 4]
            ]]"#;
    let tok = crate::tokenizer::Tokenizer::new(input.chars());
    let x = crate::parser::parser().unwrap();
    let out = convert(x(tok).unwrap().remove(0));
    println!("\n\n{:?}", out);
    let out2 = crate::evaluate(out);
    println!("\n\n{:?}", out2);
    assert_eq!(out2, Ok(crate::Expr::Number(4.0)));
}

#[test]
fn xx3() {
    let input = r#"
        ReplaceAll[
            Plus[x, Times[2, x]],
            List[
                Rule[Times[2, x], 3],
                Rule[Plus, Times]
            ]]"#;
    let tok = crate::tokenizer::Tokenizer::new(input.chars());
    let x = crate::parser::parser().unwrap();
    let out = convert(x(tok).unwrap().remove(0));
    println!("\n\n{:?}", out);
    let out2 = crate::evaluate(out);
    println!("\n\n{:?}", out2);
}

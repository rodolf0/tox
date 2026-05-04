// https://reference.wolfram.com/language/tutorial/Expressions.html
// https://reference.wolfram.com/language/tutorial/OperatorInputForms.html
// https://reference.wolfram.com/language/tutorial/InputSyntax.html

use crate::expr::Expr;

fn grammar_str() -> &'static str {
    r#"
    compound_expr := expr { ';' expr } [ ';' ] ;

    expr := set ;

    set := replace_all (':='|'=') @opset set | replace_all ;

    replace_all := replace_all '/.' rule | rule ;

    rule := arith '->' rule | arith ;

    arith := arith ('+'|'-') @opsum arith_mul | arith_mul ;

    arith_mul := arith_mul ('*'|'/'|'%') @opmul arith_pow | arith_pow ;

    arith_pow := '-' arith_pow | unsure '^' arith_pow | unsure ;

    unsure := unsure '~' arith_fac | arith_fac ;

    arith_fac := arith_fac '!' | primary ;

    arglist := expr { ',' expr } ;

    primary := atom
            | '(' expr ')'
            | primary '[' [ arglist ] ']'
            ;

    atom := '"' string '"'
         | symbol
         | number
         | '{' [ arglist ] '}'
         ;
    "#
}

#[derive(Clone, Debug, PartialEq)]
enum T {
    Expr(Box<T>, Vec<T>),
    Arglist(Vec<T>),
    String(String),
    Symbol(String),
    Number(f64),
    Nop,
}

// use to destructure T enum into a specific alternative
macro_rules! pull {
    ($p:path, $e:expr) => {
        match $e {
            $p(value) => value,
            n => panic!("Bad pull match={:?}", n),
        }
    };
}

fn convert(t: T) -> Expr {
    match t {
        T::Expr(head, args) => Expr::Head(
            Box::new(convert(*head)),
            args.into_iter().map(|a| convert(a)).collect(),
        ),
        T::Symbol(x) => Expr::Symbol(x),
        T::String(s) => Expr::String(s),
        T::Number(n) => Expr::Number(n),
        other => panic!("Bug: convert failed on '{:?}'", other),
    }
}

fn math_bin_op(lhs: T, op: T, rhs: T) -> T {
    let reduce = op == T::Symbol("Plus".into()) || op == T::Symbol("Times".into());
    let mut new_args = Vec::new();
    match lhs {
        T::Expr(h, a) if *h == op && reduce => new_args.extend(a),
        other => new_args.push(other),
    }
    match rhs {
        T::Expr(h, a) if *h == op && reduce => new_args.extend(a),
        other => new_args.push(other),
    }
    T::Expr(Box::new(op), new_args)
}

fn build_parser() -> Result<earlgrey::Parser<'static, T>, String> {
    earlgrey::ParserBuilder::<T>::new(grammar_str(), "compound_expr")
        .terminal("string", |s| Some(T::String(s.into())))
        .terminal("symbol", |s| {
            if s.chars().enumerate().all(|(i, c)| {
                i == 0 && c.is_alphabetic() || i > 0 && (c.is_alphanumeric() || c == '_')
            }) {
                Some(T::Symbol(s.into()))
            } else {
                None
            }
        })
        .terminal("number", |n| n.parse::<f64>().ok().map(T::Number))
        .unmapped_literal(|tok| match tok {
            "^" => T::Symbol("Power".into()),
            _ => T::Nop,
        })
        .optional_empty(|| T::Arglist(vec![]))
        .list_empty(|| T::Arglist(vec![]))
        .list_action(|list, mut items| {
            let mut a = pull!(T::Arglist, list);
            a.push(items.remove(1));
            T::Arglist(a)
        })
        .action3("compound_expr -> expr {;expr} [;]", |expr, list, _| {
            let a = pull!(T::Arglist, list);
            if a.is_empty() {
                expr
            } else {
                let mut args = vec![expr];
                args.extend(a);
                T::Expr(Box::new(T::Symbol("CompoundExpression".into())), args)
            }
        })
        .action3("set -> replace_all @opset set", |lhs, op, rhs| {
            T::Expr(Box::new(op), vec![lhs, rhs])
        })
        .action1("@opset -> :=", |_| T::Symbol("SetDelayed".into()))
        .action1("@opset -> =", |_| T::Symbol("Set".into()))
        .action3("replace_all -> replace_all /. rule", |lhs, _, rhs| {
            T::Expr(Box::new(T::Symbol("ReplaceAll".into())), vec![lhs, rhs])
        })
        .action3("rule -> arith -> rule", |lhs, _, rhs| {
            T::Expr(Box::new(T::Symbol("Rule".into())), vec![lhs, rhs])
        })
        .action3("arith -> arith @opsum arith_mul", math_bin_op)
        .action3("arith_mul -> arith_mul @opmul arith_pow", math_bin_op)
        .action2("arith_pow -> - arith_pow", |_, pow| match pow {
            T::Number(n) => T::Number(-n),
            other => T::Expr(
                Box::new(T::Symbol("Times".into())),
                vec![T::Number(-1.0), other],
            ),
        })
        .action3("arith_pow -> unsure ^ arith_pow", math_bin_op)
        .action3("unsure -> unsure ~ arith_fac", |n0, _, n1| {
            T::Expr(Box::new(T::Symbol("Unsure".into())), vec![n0, n1])
        })
        .action2("arith_fac -> arith_fac !", |fac, _| {
            T::Expr(
                Box::new(T::Symbol("Gamma".into())),
                vec![T::Expr(
                    Box::new(T::Symbol("Plus".into())),
                    vec![T::Number(1.0), fac],
                )],
            )
        })
        .action2("arglist -> expr {,expr}", |expr, list| {
            let mut a = vec![expr];
            a.extend(pull!(T::Arglist, list));
            T::Arglist(a)
        })
        .action3("primary -> ( expr )", |_, expr, _| expr)
        .action4("primary -> primary [ [arglist] ]", |head, _, arglist, _| {
            T::Expr(Box::new(head), pull!(T::Arglist, arglist))
        })
        .action3("atom -> \" string \"", |_, s, _| {
            assert!(matches!(s, T::String(_)));
            s
        })
        .action3("atom -> { [arglist] }", |_, arglist, _| {
            T::Expr(
                Box::new(T::Symbol("List".into())),
                pull!(T::Arglist, arglist),
            )
        })
        .action1("@opsum -> +", |_| T::Symbol("Plus".into()))
        .action1("@opsum -> -", |_| T::Symbol("Minus".into()))
        .action1("@opmul -> *", |_| T::Symbol("Times".into()))
        .action1("@opmul -> /", |_| T::Symbol("Divide".into()))
        .action1("@opmul -> %", |_| T::Symbol("Mod".into()))
        .build()
}

pub fn expr_tree(input: &str) -> Result<(), String> {
    let parser = build_parser()?;
    let tokenizer = crate::tokenizer::Tokenizer::new(input.chars());
    for tree in parser.parse_sexpr(tokenizer)? {
        println!("{}", tree.print());
    }
    Ok(())
}

pub fn parser() -> Result<impl Fn(&str) -> Result<Expr, String>, String> {
    let parser = build_parser()?;
    Ok(move |input: &str| {
        let tokenizer = crate::tokenizer::Tokenizer::new(input.chars());
        let mut trees = parser.parse_all(tokenizer)?;
        if trees.len() > 1 {
            for t in &trees {
                eprintln!("{:?}", t);
            }
            assert!(
                trees.windows(2).all(|w| w[0] == w[1]),
                "Bug: Amaiguous grammar"
            );
            panic!("Bug: Amaiguous grammar (2)");
        }
        Ok(convert(trees.swap_remove(0)))
    })
}

#[cfg(test)]
mod tests {
    use super::parser;
    use crate::expr::Expr;
    use crate::expr::Expr::*;

    #[test]
    fn basic_expr() -> Result<(), std::string::String> {
        let input = r#"FindRoot[Sum[360, Sum[a, b]], List["1, 2, 3"], {x, 2}]"#;
        let expected = Expr::from_head(
            "FindRoot",
            vec![
                Expr::from_head(
                    "Sum",
                    vec![
                        Number(360.0),
                        Expr::from_head("Sum", vec![Symbol("a".into()), Symbol("b".into())]),
                    ],
                ),
                Expr::from_head("List", vec![String("1, 2, 3".into())]),
                Expr::from_head("List", vec![Symbol("x".into()), Number(2.0)]),
            ],
        );
        assert_eq!(parser()?(input)?, expected);
        Ok(())
    }

    #[test]
    fn recursive_expr() -> Result<(), std::string::String> {
        let input = r#"f[x][y, z]"#;
        let expected = Head(
            Box::new(Expr::from_head("f", vec![Symbol("x".into())])),
            vec![Symbol("y".into()), Symbol("z".into())],
        );
        assert_eq!(parser()?(input)?, expected);
        Ok(())
    }
}

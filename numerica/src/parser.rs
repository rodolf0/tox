// https://reference.wolfram.com/language/tutorial/Expressions.html
// https://reference.wolfram.com/language/tutorial/OperatorInputForms.html
// https://reference.wolfram.com/language/tutorial/InputSyntax.html

use crate::expr::Expr;

fn grammar_str() -> &'static str {
    r#"
    arglist := arglist ',' expr | expr ;

    expr := set ;

    set := replace_all (':='|'=') @opset set | replace_all ;

    replace_all := replace_all '/.' rule | rule ;

    rule := arith '->' rule | arith ;

    arith := arith ('+'|'-') @opsum arith_mul | arith_mul ;
    arith_mul := arith_mul ('*'|'/'|'%') @opmul arith_pow | arith_pow ;
    arith_pow := '-' arith_pow | arith_fac '^' arith_pow | arith_fac ;
    arith_fac := arith_fac '!' | primary ;

    primary := atom
            | '(' expr ')'
            | primary '[' arglist ']'
            ;

    atom := '"' string '"'
         | symbol
         | number
         | '{' arglist '}'
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
        // TODO: eventually change this to non String exclusive
        T::Expr(h, args) => Expr::Expr(
            pull!(T::Symbol, *h),
            args.into_iter().map(|a| convert(a)).collect(),
        ),
        T::Symbol(x) => Expr::Symbol(x),
        T::String(s) => Expr::String(s),
        T::Number(n) => Expr::Number(n),
        other => panic!("Bug: convert failed on '{:?}'", other),
    }
}

pub fn parser() -> Result<impl Fn(&str) -> Result<Expr, String>, String> {
    let grammar = earlgrey::EbnfGrammarParser::new(grammar_str(), "expr")
        .plug_terminal("string", |_| true)
        .plug_terminal("symbol", |s| {
            s.chars().enumerate().all(|(i, c)| {
                i == 0 && c.is_alphabetic() || i > 0 && (c.is_alphanumeric() || c == '_')
            })
        })
        .plug_terminal("number", |n| n.parse::<f64>().is_ok())
        .into_grammar()?;
    let mut evaler = earlgrey::EarleyForest::new(|terminal, lexeme| match terminal {
        "symbol" => T::Symbol(lexeme.into()),
        "number" => T::Number(lexeme.parse::<f64>().unwrap()),
        "string" => T::String(lexeme.into()),
        "^" => T::Symbol("Power".into()),
        "!" => T::Symbol("!".into()), // TODO
        _ => T::Nop,
    });

    evaler.action("atom -> \" string \"", |mut args| {
        assert!(matches!(args[1], T::String(_)));
        args.swap_remove(1)
    });
    evaler.action("atom -> symbol", |mut args| {
        assert!(matches!(args[0], T::Symbol(_)));
        args.swap_remove(0)
    });
    evaler.action("atom -> number", |mut args| {
        assert!(matches!(args[0], T::Number(_)));
        args.swap_remove(0)
    });
    evaler.action("atom -> { arglist }", |mut args| {
        let arglist = pull!(T::Arglist, args.swap_remove(1));
        T::Expr(Box::new(T::Symbol("List".into())), arglist)
    });

    evaler.action("primary -> atom", |mut args| args.swap_remove(0));
    evaler.action("primary -> ( expr )", |mut args| args.swap_remove(1));
    evaler.action("primary -> primary [ arglist ]", |mut args| {
        let arglist = pull!(T::Arglist, args.swap_remove(2));
        let head = args.swap_remove(0);
        T::Expr(Box::new(head), arglist)
    });

    evaler.action("expr -> set", |mut args| args.swap_remove(0));

    evaler.action("set -> replace_all", |mut args| args.swap_remove(0));
    evaler.action("set -> replace_all @opset set", |mut args| {
        let rhs = args.swap_remove(2);
        let op = args.swap_remove(1);
        let lhs = args.swap_remove(0);
        T::Expr(Box::new(op), vec![lhs, rhs])
    });
    evaler.action("@opset -> :=", |_| T::Symbol("SetDelayed".into()));
    evaler.action("@opset -> =", |_| T::Symbol("Set".into()));

    evaler.action("replace_all -> rule", |mut args| args.swap_remove(0));
    evaler.action("replace_all -> replace_all /. rule", |mut args| {
        let rhs = args.swap_remove(2);
        let lhs = args.swap_remove(0);
        T::Expr(Box::new(T::Symbol("ReplaceAll".into())), vec![lhs, rhs])
    });

    evaler.action("rule -> arith", |mut args| args.swap_remove(0));
    evaler.action("rule -> arith -> rule", |mut args| {
        let rhs = args.swap_remove(2);
        let lhs = args.swap_remove(0);
        T::Expr(Box::new(T::Symbol("Rule".into())), vec![lhs, rhs])
    });

    evaler.action("arglist -> expr", |mut args| {
        // Don't check type could be any
        T::Arglist(vec![args.swap_remove(0)])
    });
    evaler.action("arglist -> arglist , expr", |mut args| {
        let expr = args.swap_remove(2); // Don't check type could be any
        let mut arglist = pull!(T::Arglist, args.swap_remove(0));
        arglist.push(expr);
        T::Arglist(arglist)
    });

    fn math_bin_op(mut args: Vec<T>) -> T {
        assert_eq!(args.len(), 3);
        let rhs = args.swap_remove(2);
        let op = args.swap_remove(1);
        let lhs = args.swap_remove(0);
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

    evaler.action("arith -> arith @opsum arith_mul", math_bin_op);
    evaler.action("arith -> arith_mul", |mut args| args.swap_remove(0));

    evaler.action("arith_mul -> arith_mul @opmul arith_pow", math_bin_op);
    evaler.action("arith_mul -> arith_pow", |mut args| args.swap_remove(0));

    evaler.action("arith_pow -> - arith_pow", |mut args| {
        match args.swap_remove(1) {
            T::Number(n) => T::Number(-n),
            other => T::Expr(
                Box::new(T::Symbol("Times".into())),
                vec![T::Number(-1.0), other],
            ),
        }
    });
    evaler.action("arith_pow -> arith_fac ^ arith_pow", math_bin_op);
    evaler.action("arith_pow -> arith_fac", |mut args| args.swap_remove(0));

    evaler.action("arith_fac -> arith_fac !", |mut args| {
        match args.swap_remove(0) {
            T::Number(n) => T::Number(crate::gamma(1.0 + n)),
            other => T::Expr(
                Box::new(T::Symbol("Gamma".into())),
                vec![T::Expr(
                    Box::new(T::Symbol("Plus".into())),
                    vec![T::Number(1.0), other],
                )],
            ),
        }
    });
    evaler.action("arith_fac -> primary", |mut args| args.swap_remove(0));

    evaler.action("@opsum -> +", |_| T::Symbol("Plus".into()));
    evaler.action("@opsum -> -", |_| T::Symbol("Minus".into()));
    evaler.action("@opmul -> *", |_| T::Symbol("Times".into()));
    evaler.action("@opmul -> /", |_| T::Symbol("Divide".into()));
    evaler.action("@opmul -> %", |_| T::Symbol("Mod".into()));

    let parser = earlgrey::EarleyParser::new(grammar);
    Ok(move |input: &str| {
        let tokenizer = crate::tokenizer::Tokenizer::new(input.chars());
        let mut trees = evaler.eval_all_recursive(&parser.parse(tokenizer)?)?;
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
    use crate::expr::Expr::*;

    #[test]
    fn basic_expr() -> Result<(), std::string::String> {
        let input = r#"FindRoot[Sum[360, Sum[a, b]], List["1, 2, 3"], {x, 2}]"#;
        let expected = Expr(
            "FindRoot".into(),
            vec![
                Expr(
                    "Sum".into(),
                    vec![
                        Number(360.0),
                        Expr("Sum".into(), vec![Symbol("a".into()), Symbol("b".into())]),
                    ],
                ),
                Expr("List".into(), vec![String("1, 2, 3".into())]),
                Expr("List".into(), vec![Symbol("x".into()), Number(2.0)]),
            ],
        );
        assert_eq!(parser()?(input)?, expected);
        Ok(())
    }

    // #[test]
    // fn recursive_expr() -> Result<(), std::string::String> {
    //     let input = r#"f[x][y, z]"#;
    //     let expected = Expr(
    //         Expr("f".into(), vec![Symbol("x".into())]),
    //         vec![Symbol("y".into()), Symbol("z".into())],
    //     );
    //     assert_eq!(parser()?(input)?, expected);
    //     Ok(())
    // }
}

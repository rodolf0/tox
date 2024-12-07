// https://reference.wolfram.com/language/tutorial/Expressions.html
// https://reference.wolfram.com/language/tutorial/OperatorInputForms.html
// https://reference.wolfram.com/language/tutorial/InputSyntax.html

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
    arith_fac := arith_fac '!' | '(' expr ')' | atom ;

    atom := '"' string '"'
         | symbol
         | number
         | head '[' arglist ']'
         | '{' arglist '}'
         ;
    "#
}

#[derive(Clone, Debug, PartialEq)]
pub enum T {
    Expr(String, Vec<T>),
    Arglist(Vec<T>),
    String(String),
    Symbol(String),
    Number(f64),
    Nop,
}

#[derive(PartialEq, Clone, Debug)]
pub enum Expr {
    Expr(String, Vec<Expr>),
    Symbol(String),
    Number(f64),
    Bool(bool),
    String(String),
}

fn convert(t: T) -> Expr {
    match t {
        T::Expr(h, args) => Expr::Expr(h, args.into_iter().map(|a| convert(a)).collect()),
        T::Symbol(x) => Expr::Symbol(x),
        T::String(s) => Expr::String(s),
        T::Number(n) => Expr::Number(n),
        other => panic!("Bug: convert failed on '{:?}'", other),
    }
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

pub fn parser() -> Result<impl Fn(&str) -> Result<Expr, String>, String> {
    let grammar = earlgrey::EbnfGrammarParser::new(grammar_str(), "expr")
        .plug_terminal("head", |h| {
            [
                "Divide",
                "Evaluate",
                "FindRoot",
                "Hold",
                "List",
                "Minus",
                "Plus",
                "Power",
                "ReplaceAll",
                "Rule",
                "Set",
                "Sum",
                "Times",
            ]
            .contains(&h)
        })
        .plug_terminal("string", |_| true)
        .plug_terminal("symbol", |s| {
            s.chars().enumerate().all(|(i, c)| {
                i == 0 && c.is_alphabetic() || i > 0 && (c.is_alphanumeric() || c == '_')
            })
        })
        .plug_terminal("number", |n| n.parse::<f64>().is_ok())
        .into_grammar()?;
    let mut evaler = earlgrey::EarleyForest::new(|terminal, lexeme| match terminal {
        "head" => T::Symbol(lexeme.to_string()),
        "symbol" => T::Symbol(lexeme.to_string()),
        "number" => T::Number(lexeme.parse::<f64>().unwrap()),
        "string" => T::String(lexeme.to_string()),
        "^" => T::Symbol("Power".to_string()),
        "!" => T::Symbol("!".to_string()), // TODO
        _ => T::Nop,
    });

    evaler.action("atom -> { arglist }", |mut args| {
        let arglist = pull!(T::Arglist, args.swap_remove(1));
        T::Expr("List".to_string(), arglist)
    });

    evaler.action("expr -> set", |mut args| args.swap_remove(0));

    evaler.action("set -> replace_all", |mut args| args.swap_remove(0));
    evaler.action("set -> replace_all @opset set", |mut args| {
        let rhs = args.swap_remove(2);
        let op = pull!(T::Symbol, args.swap_remove(1));
        let lhs = args.swap_remove(0);
        T::Expr(op, vec![lhs, rhs])
    });
    evaler.action("@opset -> :=", |_| T::Symbol("SetDelayed".to_string()));
    evaler.action("@opset -> =", |_| T::Symbol("Set".to_string()));

    evaler.action("replace_all -> rule", |mut args| args.swap_remove(0));
    evaler.action("replace_all -> replace_all /. rule", |mut args| {
        let rhs = args.swap_remove(2);
        let lhs = args.swap_remove(0);
        T::Expr("ReplaceAll".to_string(), vec![lhs, rhs])
    });

    evaler.action("rule -> arith", |mut args| args.swap_remove(0));
    evaler.action("rule -> arith -> rule", |mut args| {
        let rhs = args.swap_remove(2);
        let lhs = args.swap_remove(0);
        T::Expr("Rule".to_string(), vec![lhs, rhs])
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
    evaler.action("atom -> head [ arglist ]", |mut args| {
        let arglist = pull!(T::Arglist, args.swap_remove(2));
        let head = pull!(T::Symbol, args.swap_remove(0));
        T::Expr(head, arglist)
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
        let op = pull!(T::Symbol, args.swap_remove(1));
        let lhs = args.swap_remove(0);
        let reduce = op == "Plus" || op == "Times";
        let mut new_args = Vec::new();
        match lhs {
            T::Expr(h, a) if h == op && reduce => new_args.extend(a),
            other => new_args.push(other),
        }
        match rhs {
            T::Expr(h, a) if h == op && reduce => new_args.extend(a),
            other => new_args.push(other),
        }
        T::Expr(op, new_args)
    }

    evaler.action("arith -> arith @opsum arith_mul", math_bin_op);
    evaler.action("arith -> arith_mul", |mut args| args.swap_remove(0));

    evaler.action("arith_mul -> arith_mul @opmul arith_pow", math_bin_op);
    evaler.action("arith_mul -> arith_pow", |mut args| args.swap_remove(0));

    evaler.action("arith_pow -> - arith_pow", |mut args| {
        match args.swap_remove(1) {
            T::Number(n) => T::Number(-n),
            other => T::Expr("Times".to_string(), vec![T::Number(-1.0), other]),
        }
    });
    evaler.action("arith_pow -> arith_fac ^ arith_pow", math_bin_op);
    evaler.action("arith_pow -> arith_fac", |mut args| args.swap_remove(0));

    evaler.action("arith_fac -> arith_fac !", |mut args| {
        match args.swap_remove(0) {
            T::Number(n) => T::Number(crate::gamma(1.0 + n)),
            other => T::Expr(
                "Gamma".to_string(),
                vec![T::Expr("Plus".to_string(), vec![T::Number(1.0), other])],
            ),
        }
    });
    evaler.action("arith_fac -> ( expr )", |mut args| args.swap_remove(1));
    evaler.action("arith_fac -> atom", |mut args| args.swap_remove(0));

    evaler.action("@opsum -> +", |_| T::Symbol("Plus".to_string()));
    evaler.action("@opsum -> -", |_| T::Symbol("Minus".to_string()));
    evaler.action("@opmul -> *", |_| T::Symbol("Times".to_string()));
    evaler.action("@opmul -> /", |_| T::Symbol("Divide".to_string()));
    evaler.action("@opmul -> %", |_| T::Symbol("Mod".to_string()));

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

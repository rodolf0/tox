// https://reference.wolfram.com/language/tutorial/Expressions.html
// https://reference.wolfram.com/language/tutorial/OperatorInputForms.html
// https://reference.wolfram.com/language/tutorial/InputSyntax.html

fn grammar_str() -> &'static str {
    r#"
    # full form grammar
    expr      := head '[' arglist ']' ;
    arglist   := arglist ',' arg | arg ;
    bracketed := '(' arglist ')' | '{' arglist '}' ;
    arg       := expr | bracketed | '"' string '"' | symbol | number ;
    "#
}

#[derive(Clone, Debug)]
pub enum T {
    Expr(String, Vec<T>),
    Head(String),
    List(Vec<T>),
    Arglist(Vec<T>),
    Arg(Box<T>),
    Number(f64),
    Symbol(String),
    String(String),
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
        T::List(l) => Expr::Expr(
            "List".to_string(),
            l.into_iter().map(|i| convert(i)).collect(),
        ),
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

pub fn parser<InputIter>() -> Result<impl Fn(InputIter) -> Result<Expr, String>, String>
where
    InputIter: Iterator,
    InputIter::Item: AsRef<str> + std::fmt::Debug,
{
    let grammar = earlgrey::EbnfGrammarParser::new(grammar_str(), "expr")
        .plug_terminal("head", |h| {
            [
                "Sum",
                "FindRoot",
                "List",
                "Set",
                "ReplaceAll",
                "Plus",
                "Times",
                "Rule",
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
        "head" => T::Head(lexeme.to_string()),
        "symbol" => T::Symbol(lexeme.to_string()),
        "number" => T::Number(lexeme.parse::<f64>().unwrap()),
        "string" => T::String(lexeme.to_string()),
        _ => T::Nop,
    });
    evaler.action("expr -> head [ arglist ]", |mut args| {
        let arglist = pull!(T::Arglist, args.swap_remove(2));
        let head = pull!(T::Head, args.swap_remove(0));
        T::Expr(head, arglist)
    });
    evaler.action("arglist -> arglist , arg", |mut args| {
        let arg = pull!(T::Arg, args.swap_remove(2));
        let mut arglist = pull!(T::Arglist, args.swap_remove(0));
        arglist.push(*arg);
        T::Arglist(arglist)
    });
    evaler.action("arglist -> arg", |mut args| {
        let arg = pull!(T::Arg, args.swap_remove(0));
        T::Arglist(vec![*arg])
    });
    evaler.action("bracketed -> ( arglist )", |mut args| args.swap_remove(1));
    evaler.action("bracketed -> { arglist }", |mut args| {
        T::List(pull!(T::Arglist, args.swap_remove(1)))
    });
    evaler.action("arg -> expr", |mut args| {
        T::Arg(Box::new(args.swap_remove(0)))
    });
    evaler.action("arg -> bracketed", |mut args| {
        T::Arg(Box::new(args.swap_remove(0)))
    });
    evaler.action("arg -> \" string \"", |mut args| {
        T::Arg(Box::new(args.swap_remove(1)))
    });
    evaler.action("arg -> symbol", |mut args| {
        T::Arg(Box::new(args.swap_remove(0)))
    });
    evaler.action("arg -> number", |mut args| {
        T::Arg(Box::new(args.swap_remove(0)))
    });

    let parser = earlgrey::EarleyParser::new(grammar);
    Ok(move |input| {
        let mut trees = evaler.eval_all(&parser.parse(input)?)?;
        assert_eq!(trees.len(), 1, "Bug: Ambiguous grammar.");
        Ok(convert(trees.swap_remove(0)))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_basic_input() -> Result<(), String> {
        let parser = parser()?;
        let tokenized_input = [
            "FindRoot", "[", "Plus", "[", "x", ",", "2", "]", ",", "{", "x", ",", "2", "}", "]",
        ];
        let expected = Expr::Expr(
            "FindRoot".to_string(),
            vec![
                Expr::Expr(
                    "Plus".to_string(),
                    vec![Expr::Symbol("x".to_string()), Expr::Number(2.0)],
                ),
                Expr::Expr(
                    "List".to_string(),
                    vec![Expr::Symbol("x".to_string()), Expr::Number(2.0)],
                ),
            ],
        );
        assert_eq!(parser(tokenized_input.into_iter())?, expected);
        Ok(())
    }
}

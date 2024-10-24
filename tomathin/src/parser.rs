#![deny(warnings)]

// https://reference.wolfram.com/language/tutorial/Expressions.html
// https://reference.wolfram.com/language/tutorial/OperatorInputForms.html
// https://reference.wolfram.com/language/tutorial/InputSyntax.html

fn grammar_str() -> &'static str {
    r#"
    # full form grammar
    expr      := head '[' arglist ']' ;
    arglist   := arglist ',' arg | arg ;
    bracketed := '(' arglist ')' | '{' arglist '}' | '[' arglist ']' ;
    arg       := expr | bracketed | '"' string '"' | symbol | number ;
    "#
}

#[derive(Clone, Debug)]
pub enum T {
    Expr(String, Vec<T>),
    Head(String),
    Arglist(Vec<T>),
    Arg(Box<T>),
    Number(f64),
    Symbol(String),
    String(String),
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

pub fn parser<InputIter>() -> Result<impl Fn(InputIter) -> Result<Vec<T>, String>, String>
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
        let arglist = pull!(T::Arglist, args.remove(2));
        let head = pull!(T::Head, args.remove(0));
        T::Expr(head, arglist)
    });
    evaler.action("arglist -> arglist , arg", |mut args| {
        let arg = pull!(T::Arg, args.remove(2));
        let mut arglist = pull!(T::Arglist, args.remove(0));
        arglist.push(*arg);
        T::Arglist(arglist)
    });
    evaler.action("arglist -> arg", |mut args| {
        let arg = pull!(T::Arg, args.remove(0));
        T::Arglist(vec![*arg])
    });
    evaler.action("bracketed -> ( arglist )", |mut args| args.remove(1));
    evaler.action("bracketed -> { arglist }", |mut args| args.remove(1));
    evaler.action("bracketed -> [ arglist ]", |mut args| args.remove(1));
    evaler.action("arg -> expr", |mut args| T::Arg(Box::new(args.remove(0))));
    evaler.action("arg -> bracketed", |mut args| {
        T::Arg(Box::new(args.remove(0)))
    });
    evaler.action("arg -> \" string \"", |mut args| {
        T::Arg(Box::new(args.remove(1)))
    });
    evaler.action("arg -> symbol", |mut args| T::Arg(Box::new(args.remove(0))));
    evaler.action("arg -> number", |mut args| T::Arg(Box::new(args.remove(0))));

    let parser = earlgrey::EarleyParser::new(grammar);
    Ok(move |input| evaler.eval_all(&parser.parse(input)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_x() {
        // let input = [
        //     "FindRoot", "[", "75", "/", "(", "r", "+", "1", ")", "+", "75", "/", "(", "r", "+",
        //     "1", ")", "^", "2", "==", "0", ",", "{", "r", "=", "2", "}", "]",
        // ];
        // FindRooot[Sum[Divide[75, Sum[Var[r], 1]], Divide[75, Pow[Sum[Var[r], 1], 2]]]]
        let input = [
            "FindRoot", "[", "75", "/", "(", "r", "+", "1", ")", "+", "75", "/", "(", "r", "+",
            "1", ")", "^", "2", "]",
        ];
        let p = parser().unwrap();
        println!("{:?}", p(input.into_iter()).unwrap());
    }
}

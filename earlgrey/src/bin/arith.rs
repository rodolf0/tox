fn build_grammar() -> earlgrey::Grammar {
    use std::str::FromStr;
    earlgrey::GrammarBuilder::default()
        .nonterm("expr")
        .nonterm("term")
        .nonterm("factor")
        .nonterm("power")
        .nonterm("ufact")
        .nonterm("group")
        .nonterm("func")
        .nonterm("args")
        .terminal("[n]", |n| f64::from_str(n).is_ok())
        .terminal("+", |n| n == "+")
        .terminal("-", |n| n == "-")
        .terminal("*", |n| n == "*")
        .terminal("/", |n| n == "/")
        .terminal("%", |n| n == "%")
        .terminal("^", |n| n == "^")
        .terminal("!", |n| n == "!")
        .terminal("(", |n| n == "(")
        .terminal(")", |n| n == ")")
        .rule("expr", &["term"])
        .rule("expr", &["expr", "+", "term"])
        .rule("expr", &["expr", "-", "term"])
        .rule("term", &["factor"])
        .rule("term", &["term", "*", "factor"])
        .rule("term", &["term", "/", "factor"])
        .rule("term", &["term", "%", "factor"])
        .rule("factor", &["power"])
        .rule("factor", &["-", "factor"])
        .rule("power", &["ufact"])
        .rule("power", &["ufact", "^", "factor"])
        .rule("ufact", &["group"])
        .rule("ufact", &["ufact", "!"])
        .rule("group", &["[n]"])
        .rule("group", &["(", "expr", ")"])
        .into_grammar("expr")
        .expect("Bad Gramar")
}

struct Tokenizer<I: Iterator<Item = char>>(lexers::Scanner<I>);

impl<I: Iterator<Item = char>> Iterator for Tokenizer<I> {
    type Item = String;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.scan_whitespace();
        self.0
            .scan_math_op()
            .or_else(|| self.0.scan_number())
            .or_else(|| self.0.scan_identifier())
    }
}

fn tokenizer<I: Iterator<Item = char>>(input: I) -> Tokenizer<I> {
    Tokenizer(lexers::Scanner::new(input))
}

fn gamma(x: f64) -> f64 {
    #[link(name = "m")]
    extern "C" {
        fn tgamma(x: f64) -> f64;
    }
    unsafe { tgamma(x) }
}

fn semanter<'a>() -> earlgrey::EarleyForest<'a, f64> {
    use std::str::FromStr;
    let mut ev = earlgrey::EarleyForest::new(|symbol, token| match symbol {
        "[n]" => f64::from_str(token).unwrap(),
        _ => 0.0,
    });
    ev.action("expr -> term", |n| n[0]);
    ev.action("expr -> expr + term", |n| n[0] + n[2]);
    ev.action("expr -> expr - term", |n| n[0] - n[2]);
    ev.action("term -> factor", |n| n[0]);
    ev.action("term -> term * factor", |n| n[0] * n[2]);
    ev.action("term -> term / factor", |n| n[0] / n[2]);
    ev.action("term -> term % factor", |n| n[0] % n[2]);
    ev.action("factor -> power", |n| n[0]);
    ev.action("factor -> - factor", |n| -n[1]);
    ev.action("power -> ufact", |n| n[0]);
    ev.action("power -> ufact ^ factor", |n| n[0].powf(n[2]));
    ev.action("ufact -> group", |n| n[0]);
    ev.action("ufact -> ufact !", |n| gamma(n[0] + 1.0));
    ev.action("group -> [n]", |n| n[0]);
    ev.action("group -> ( expr )", |n| n[1]);
    ev
}

fn main() -> Result<(), String> {
    let grammar = build_grammar();
    let parser = earlgrey::EarleyParser::new(grammar.clone());
    let evaler = semanter();

    if std::env::args().len() > 1 {
        let input = std::env::args().skip(1).collect::<Vec<String>>().join(" ");
        match parser.parse(tokenizer(input.chars())) {
            Err(e) => println!("Parse err: {:?}", e),
            Ok(state) => println!("{:?}", evaler.eval(&state)),
        }
        return Ok(());
    }

    println!("{:?}", grammar);
    use rustyline::error::ReadlineError;

    let mut rl = rustyline::DefaultEditor::new().map_err(|e| e.to_string())?;
    loop {
        match rl.readline("~> ") {
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => return Ok(()),
            Err(e) => return Err(format!("Readline err: {:?}", e)),
            Ok(line) => match parser.parse(&mut tokenizer(line.chars())) {
                Err(e) => println!("Parse err: {:?}", e),
                Ok(state) => {
                    let _ = rl.add_history_entry(&line);
                    println!("{:?}", evaler.eval(&state));
                }
            },
        }
    }
}

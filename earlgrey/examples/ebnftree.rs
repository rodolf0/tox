#![deny(warnings)]

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

fn main() -> Result<(), String> {
    let grammar = r#"
        expr   := expr ('+'|'-') term | term ;
        term   := term ('*'|'/') factor | factor ;
        factor := '-' factor | power ;
        power  := ufact '^' factor | ufact ;
        ufact  := ufact '!' | group ;
        group  := num | '(' expr ')' ;
    "#;

    let input = std::env::args().skip(1).collect::<Vec<String>>().join(" ");

    use std::str::FromStr;
    let grammar = earlgrey::EbnfGrammarParser::new(grammar, "expr")
        .plug_terminal("num", |n| f64::from_str(n).is_ok())
        .into_grammar()?;
    let parser = earlgrey::sexpr_parser(grammar)?;

    for tree in parser(tokenizer(input.chars()))? {
        println!("{}", tree.print());
    }

    Ok(())
}

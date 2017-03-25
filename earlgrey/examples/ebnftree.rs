extern crate lexers;
extern crate earlgrey;

struct Tokenizer(lexers::Scanner<char>);

impl lexers::Nexter<String> for Tokenizer {
    fn get_item(&mut self) -> Option<String> {
        self.0.ignore_ws();
        lexers::scan_math_op(&mut self.0)
            .or_else(|| lexers::scan_number(&mut self.0))
            .or_else(|| lexers::scan_identifier(&mut self.0))
    }
}

impl Tokenizer {
    fn from_str(input: &str) -> lexers::Scanner<String> {
        lexers::Scanner::new(
            Box::new(Tokenizer(lexers::Scanner::from_str(&input))))
    }
}

fn main() {
    let grammar = r#"
        expr   := expr ('+'|'-') term | term ;
        term   := term ('*'|'/') factor | factor ;
        factor := '-' factor | power ;
        power  := ufact '^' factor | ufact ;
        ufact  := ufact '!' | group ;
        group  := num | '(' expr ')' ;
    "#;

    use std::str::FromStr;
    let parser = earlgrey::ParserBuilder::new()
        .plug_terminal("num", |n| f64::from_str(n).is_ok())
        .into_parser("expr", &grammar);

    let input = std::env::args().skip(1).
        collect::<Vec<String>>().join(" ");
    match parser.parse(&mut Tokenizer::from_str(&input)) {
        Ok(state) => {
            let trees = earlgrey::subtree_evaler(parser.g.clone())
                        .eval_all(&state);
            for t in trees {
                println!("================================");
                t[0].print();
            }
        },
        Err(e) => println!("Arit error: {:?}", e)
    }
    return;
}

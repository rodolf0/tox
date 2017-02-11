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

    let parser = earlgrey::ParserBuilder::new(&grammar)
        .plug_terminal("num", |n| n.chars().all(|c| c.is_numeric()))
        .into_parser("expr");

    let input = std::env::args().skip(1).
        collect::<Vec<String>>().join(" ");
    match parser.parse(&mut Tokenizer::from_str(&input)) {
        Ok(state) => {
            let trees = earlgrey::all_trees(parser.g.start(), &state);
            for t in trees {
                println!("================================");
                t.print();
            }
        },
        Err(e) => println!("Arit error: {:?}", e)
    }
    return;
}

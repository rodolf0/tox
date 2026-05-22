#![deny(warnings)]

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

    let parser = earlgrey::ParserBuilder::for_sexpr(grammar, "expr")
        .terminal("num", |n| Some(earlgrey::Sexpr::Atom(n.to_string())))
        .build()?;

    for tree in parser.parse_sexpr(
        lexers::StringTokenizer::from(input.as_str())
            .symbols(["+", "-", "*", "/", "%", "^", "!", "(", ")"]),
    )? {
        println!("{}", tree.print());
    }

    Ok(())
}

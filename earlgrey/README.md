# earlgrey
Use *Earley*'s algorithm to retrieve all possible parses of an input given a grammar.

Example:
```
fn main() {
    use std::str::FromStr;

    let grammar = r#"
        expr   := expr ('+'|'-') term | term ;
        term   := term ('*'|'/') factor | factor ;
        factor := '-' factor | power ;
        power  := ufact '^' factor | ufact ;
        ufact  := ufact '!' | group ;
        group  := num | '(' expr ')' ;
    "#;

    let input = "3.2 ^ 3 - (8 / 4)!";

    let parser = earlgrey::ParserBuilder::new()
        .plug_terminal("num", |n| f64::from_str(n).is_ok())
        .treeficator("expr", &grammar);

    for tree in parser(&mut Tokenizer::from_str(input)).unwrap() {
      tree[0].print();
    }
}
```


### earley references
* http://loup-vaillant.fr/tutorials/earley-parsing/
* https://user.phil-fak.uni-duesseldorf.de/~kallmeyer/Parsing/earley.pdf
* http://joshuagrams.github.io/pep/
* https://github.com/tomerfiliba/tau/blob/master/earley3.py

# Example

```rust
// Full code at examples/ebnftree.rs
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
  let grammar = earlgrey::EbnfGrammarParser::new(grammar, "expr")
      .plug_terminal("num", |n| f64::from_str(n).is_ok())
      .into_grammar()
      .unwrap();

  let parser = earlgrey::sexpr_parser(grammar).unwrap();

  for tree in parser(tokenizer(input.chars()))? {
      println!("{}", tree.print());
  }
}
```

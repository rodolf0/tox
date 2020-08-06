# Documentation

Abackus crate adds a layer on top of [earlgrey crate](https://crates.io/crates/earlgrey) to simplify writing a **grammar**. You can simply use an EBNF style String instead of manually adding rules.

You can describe your grammar like this:
```rust
let grammar = r#"
    S := S '+' N | N ;
    N := '[0-9]' ;
"#;

ParserBuilder::default()
  .plug_terminal("[0-9]", |n| "1234567890".contains(n))
  .plug_terminal("[+]", |c| c == "+")
  .into_parser("S")
```
Instead of the more verbose:
```rust
// Gramar:  S -> S + N | N;  N -> [0-9];
let g = earlgrey::GrammarBuilder::default()
  .nonterm("S")
  .nonterm("N")
  .terminal("[+]", |c| c == "+")
  .terminal("[0-9]", |n| "1234567890".contains(n))
  .rule("S", &["S", "[+]", "N"])
  .rule("S", &["N"])
  .rule("N", &["[0-9]"])
  .into_grammar("S")
  .unwrap();

earlgrey::EarleyParser::new(g)
```

### How it works

Underneath the covers an `earlgrey::EarleyParser` is used to build a parser for EBNF grammar. (For details you can check `earlgrey/ebnf.rs`). That parser is then used to build a final parser for the grammar provided by the user.

## Example

```rust
fn main() {
  let grammar = r#"
    expr   := expr ('+'|'-') term | term ;
    term   := term ('*'|'/') factor | factor ;
    factor := '-' factor | power ;
    power  := ufact '^' factor | ufact ;
    ufact  := ufact '!' | group ;
    group  := num | '(' expr ')' ;
  "#;

  // Build a parser for our grammar and while at it, plug in an
  // evaluator to extract the resulting tree as S-expressions.
  use std::str::FromStr;
  let trif = abackus::ParserBuilder::default()
      .plug_terminal("num", |n| f64::from_str(n).is_ok())
      .sexprificator(&grammar, "expr");

  // Read some input from command-line
  let input = std::env::args().skip(1).
      collect::<Vec<String>>().join(" ");

  // Print resulting parse trees
  // See example for tokenizer https://github.com/rodolf0/tox/blob/master/abackus/examples/ebnftree.rs
  match trif(&mut tokenizer(input.chars())) {
      Ok(trees) => for t in trees { println!("{}", t.print()); },
      Err(e) => println!("{:?}", e)
  }
}
```

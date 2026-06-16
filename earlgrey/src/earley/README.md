# The Earley Algorithm

The `earley` module implements the classic Earley parsing algorithm. 

Use `GrammarBuilder` to define a grammar, `EarleyParser` to produce parse trees, and `EarleyForest` to evaluate them.

## Example

A toy parser that understands sums:

```rust
fn main() {
    // Grammar:  S -> S + N | N;  N -> [0-9]
    let g = earlgrey::GrammarBuilder::default()
      .nonterm("S")
      .nonterm("N")
      .terminal("[+]", |c| c == "+")
      .terminal("[0-9]", |n| n.chars().all(|c| c.is_ascii_digit()))
      .rule("S", &["S", "[+]", "N"])
      .rule("S", &["N"])
      .rule("N", &["[0-9]"])
      .into_grammar("S")
      .unwrap();

    // Parse some input
    let input = "1 + 2 + 3".split_whitespace();
    let trees = earlgrey::EarleyParser::new(g)
        .parse(input)
        .unwrap();

    // Evaluate the results
    let mut ev = earlgrey::EarleyForest::new(
        |symbol, token| match symbol {
            "[0-9]" => token.parse().unwrap(),
            _ => 0.0,
        });

    ev.action("S -> S [+] N", |n| n[0] + n[2]);
    ev.action("S -> N", |n| n[0]);
    ev.action("N -> [0-9]", |n| n[0]);

    println!("{}", ev.eval(&trees).unwrap()); // 6
}
```

## References

* http://loup-vaillant.fr/tutorials/earley-parsing/
* https://user.phil-fak.uni-duesseldorf.de/~kallmeyer/Parsing/earley.pdf
* http://joshuagrams.github.io/pep/
* https://github.com/tomerfiliba/tau/blob/master/earley3.py

# Documentation

Earlgrey is a crate for building parsers that can understand context-free grammars.

## How to use it

Parsing stage:

- First you need to define a grammar using `GrammarBuilder` to define terminals and rules.
- Then build an `EarleyParser` for that grammar and call `parse` on some input.

Invoking the parser on some input returns an opaque type (list of Earley items) that encodes all possible trees. If the grammar is unambiguous this should represent a single tree.

Evaluating the result:

You need an `EarleyForest` that will walk through all resulting parse trees and act on them.
- To build this you provide a function that given a terminal produces an AST node.
- Then you define semantic actions to evaluate how to interpret each rule in the grammar.

## Example

A toy parser that can understand sums.

```rust
fn main() {
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

    // Parse some sum
    let input = "1 + 2 + 3".split_whitespace();
    let trees = earlgrey::EarleyParser::new(g)
        .parse(input)
        .unwrap();

    // Evaluate the results
    // Describe what to do when we find a Terminal
    let mut ev = earlgrey::EarleyForest::new(
        |symbol, token| match symbol {
            "[0-9]" => token.parse().unwrap(),
            _ => 0.0,
        });

    // Describe how to execute grammar rules
    ev.action("S -> S [+] N", |n| n[0] + n[2]);
    ev.action("S -> N", |n| n[0]);
    ev.action("N -> [0-9]", |n| n[0]);

    println!("{}", ev.eval(&trees).unwrap());
}
```


#### References for Earley's algorithm
* http://loup-vaillant.fr/tutorials/earley-parsing/
* https://user.phil-fak.uni-duesseldorf.de/~kallmeyer/Parsing/earley.pdf
* http://joshuagrams.github.io/pep/
* https://github.com/tomerfiliba/tau/blob/master/earley3.py

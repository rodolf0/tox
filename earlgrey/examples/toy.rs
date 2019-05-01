fn main() {
    // Gramar:  S -> S + N | N;  N -> [0-9];
    let grammar = earlgrey::GrammarBuilder::default()
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
    let trees = earlgrey::EarleyParser::new(grammar)
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

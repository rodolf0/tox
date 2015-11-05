use earley::{Terminal, NonTerminal, GrammarBuilder, Grammar};
use earley::{Lexer, EarleyParser};

#[cfg(test)]
fn build_grammar() -> Grammar {
    let mut gb = GrammarBuilder::new();
    // register some symbols
    gb.symbol(NonTerminal::new("Sum"))
      .symbol(NonTerminal::new("Product"))
      .symbol(NonTerminal::new("Factor"))
      .symbol(Terminal::new("Number", |n: &str| {
          n.chars().all(|c| "1234567890".contains(c))
        }))
      .symbol(Terminal::new("[+-]", |n: &str| {
          n.len() == 1 && "+-".contains(n)
        }))
      .symbol(Terminal::new("[*/]", |n: &str| {
          n.len() == 1 && "*/".contains(n)
        }))
      .symbol(Terminal::new("(", |n: &str| { n == "(" }))
      .symbol(Terminal::new(")", |n: &str| { n == ")" }));
    // add grammar rules
    gb.rule("Sum",     vec!["Sum", "[+-]", "Product"])
      .rule("Sum",     vec!["Product"])
      .rule("Product", vec!["Product", "[*/]", "Factor"])
      .rule("Product", vec!["Factor"])
      .rule("Factor",  vec!["(", "Sum", ")"])
      .rule("Factor",  vec!["Number"]);

    let g = gb.into_grammar("Sum");

    assert_eq!(g.start, g.symbols["Sum"]);
    assert_eq!(g.symbols.len(), 8);
    assert_eq!(g.rules("Sum").count(), 2);
    assert_eq!(g.rules("Product").count(), 2);
    assert_eq!(g.rules("Factor").count(), 2);

    return g;
}


#[test]
fn test1() {
    let g = build_grammar();
    let mut input = Lexer::from_str("1+(2*3+4)", "+*-/()");
    let p = EarleyParser::new(g);

    let state = p.build_state(&mut input).unwrap();
    for (idx, stateset) in state.iter().enumerate() {
        println!("=== {} ===", idx);
        for i in stateset.iter() {
            println!("{}|{}  {:?} -> {:?}", i.start, i.dot, i.rule.name, i.rule.spec);
        }
    }
}

#[test]
fn test2() {
    let g = build_grammar();
    let mut input = Lexer::from_str("1+", "+*-/()");
    let p = EarleyParser::new(g);
    assert!(p.build_state(&mut input).is_err());
}

#[test]
fn test3() {
    let mut gb = GrammarBuilder::new();
    // Build bogus grammar
    gb.symbol(NonTerminal::new("A"));
    gb.symbol(NonTerminal::new("B"));
    gb.rule("A", Vec::new());
    gb.rule("A", vec!["B"]);
    gb.rule("B", vec!["A"]);

    let g = gb.into_grammar("A");

    assert_eq!(g.start, g.symbols["A"]);
    assert_eq!(g.symbols.len(), 2);

    let mut input = Lexer::from_str("", "-");
    let p = EarleyParser::new(g);

    let state = p.build_state(&mut input).unwrap();
    for (idx, stateset) in state.iter().enumerate() {
        println!("=== {} ===", idx);
        for i in stateset.iter() {
            println!("{}|{}  {:?} -> {:?}", i.start, i.dot, i.rule.name, i.rule.spec);
        }
    }
}

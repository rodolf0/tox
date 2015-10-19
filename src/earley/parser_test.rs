use earley::{Terminal, NonTerminal, Grammar};
use earley::{Lexer, EarleyParser};

#[cfg(test)]
fn build_grammar() -> Grammar {
    let mut g = Grammar::new("Sum");

    // register some symbols
    g.set_sym("Sum", NonTerminal::new("Sum"));
    g.set_sym("Product", NonTerminal::new("Product"));
    g.set_sym("Factor", NonTerminal::new("Factor"));
    g.set_sym("Number", Terminal::new(|n: &str| {
        n.chars().all(|c| "1234567890".contains(c))
    }));
    g.set_sym("[+-]", Terminal::new(|n: &str| {
        n.len() == 1 && "+-".contains(n)
    }));
    g.set_sym("[*/]", Terminal::new(|n: &str| {
        n.len() == 1 && "*/".contains(n)
    }));
    g.set_sym("(", Terminal::new(|n: &str| { n == "(" }));
    g.set_sym(")", Terminal::new(|n: &str| { n == ")" }));

    // add grammar rules
    g.add_rule("Sum",     vec!["Sum", "[+-]", "Product"]);
    g.add_rule("Sum",     vec!["Product"]);
    g.add_rule("Product", vec!["Product", "[*/]", "Factor"]);
    g.add_rule("Product", vec!["Factor"]);
    g.add_rule("Factor",  vec!["(", "Sum", ")"]);
    g.add_rule("Factor",  vec!["Number"]);

    assert_eq!(g.start, "Sum");
    assert_eq!(g.symbols.len(), 8);
    assert_eq!(g.rules["Sum"].len(), 2);
    assert_eq!(g.rules["Product"].len(), 2);
    assert_eq!(g.rules["Factor"].len(), 2);

    g.build_nullable();
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
    let mut g = Grammar::new("A");
    // Build bogus grammar
    g.set_sym("A", NonTerminal::new("A"));
    g.set_sym("B", NonTerminal::new("B"));
    g.add_rule("A", Vec::new());
    g.add_rule("A", vec!["B"]);
    g.add_rule("B", vec!["A"]);
    // build nullable symbols
    g.build_nullable();

    assert_eq!(g.start, "A");
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


/*
 *
 * Pow := Num '^' Pow | Num
 *
 */

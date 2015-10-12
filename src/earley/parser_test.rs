use earley::{Terminal, NonTerminal, Grammar};
use earley::{Lexer, EarleyParser};

#[test]
fn test1() {
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

    let mut input = Lexer::from_str("1+(2*3+4)", "+*-/()");
    let p = EarleyParser::new(g);

    let state = p.build_state(&mut input).unwrap();

    for (idx, stateset) in state.iter().enumerate() {
        println!("=== {} ===", idx);
        for i in stateset.iter() {
            println!("{:?} -> {:?}", i.rule.name, i.rule.spec);
        }
    }
}

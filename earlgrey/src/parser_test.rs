#![deny(warnings)]

use grammar::{GrammarBuilder, Grammar};
use parser::EarleyParser;
use trees::EarleyForest;
use std::fmt;


#[derive(Debug,Clone,PartialEq)]
enum Tree {
    // ("[+-]", "+")
    Leaf(String, String),
    // ("E -> E [+-] E", [...])
    Node(String, Vec<Tree>),
}

fn tree_evaler<'a>(g: Grammar) -> EarleyForest<'a, Tree> {
    let mut evaler = EarleyForest::new(
        |sym, tok| Tree::Leaf(sym.to_string(), tok.to_string()));
    for rule in g.str_rules() {
        evaler.action(&rule.to_string(), move |nodes|
                      Tree::Node(rule.to_string(), nodes));
    }
    evaler
}

fn check_trees<T: fmt::Debug>(trees: &Vec<T>, expected: Vec<&str>) {
    use std::collections::HashSet;
    use std::iter::FromIterator;
    assert_eq!(trees.len(), expected.len());
    let mut expect = HashSet::<&str>::from_iter(expected);
    for t in trees {
        let debug_string = format!("{:?}", t);
        eprintln!("Removing {}", debug_string);
        assert!(expect.remove(debug_string.as_str()));
    }
    assert_eq!(0, expect.len());
}

mod math {
    use grammar::{Grammar, GrammarBuilder};

    fn grammar_math() -> Grammar {
        // Sum -> Sum + Mul | Mul
        // Mul -> Mul * Pow | Pow
        // Pow -> Num ^ Pow | Num
        // Num -> Number | ( Sum )
        GrammarBuilder::default()
          .nonterm("Sum")
          .nonterm("Mul")
          .nonterm("Pow")
          .nonterm("Num")
          .terminal("Number", |n| n.chars().all(|c| "1234567890".contains(c)))
          .terminal("[+-]", |n| n.len() == 1 && "+-".contains(n))
          .terminal("[*/]", |n| n.len() == 1 && "*/".contains(n))
          .terminal("[^]", |n| { n == "^" })
          .terminal("(", |n| { n == "(" })
          .terminal(")", |n| { n == ")" })
          .rule("Sum", &["Sum", "[+-]", "Mul"])
          .rule("Sum", &["Mul"])
          .rule("Mul", &["Mul", "[*/]", "Pow"])
          .rule("Mul", &["Pow"])
          .rule("Pow", &["Num", "[^]", "Pow"])
          .rule("Pow", &["Num"])
          .rule("Num", &["(", "Sum", ")"])
          .rule("Num", &["Number"])
          .into_grammar("Sum")
          .expect("Grammar is broken")
    }

    #[test]
    fn math_grammar_test() {
        use parser::EarleyParser;
        use super::{Tree, tree_evaler};
        fn node(rule: &str, subtree: Vec<Tree>) -> Tree {
            Tree::Node(rule.to_string(), subtree)
        }
        fn leaf(rule: &str, lexeme: &str) -> Tree {
            Tree::Leaf(rule.to_string(), lexeme.to_string())
        }
        fn leafify(rules: &[&str], subtree: Tree) -> Tree {
            if rules.len() == 0 { return subtree; }
            Tree::Node(rules[0].to_string(), vec![leafify(&rules[1..], subtree)])
        }

        let tree =
            node("Sum -> Sum [+-] Mul", vec![
                leafify(&["Sum -> Mul",
                          "Mul -> Pow",
                          "Pow -> Num",
                          "Num -> Number"], leaf("Number", "1")),
                leaf("[+-]", "+"),
                leafify(&["Mul -> Pow",
                          "Pow -> Num"], node("Num -> ( Sum )", vec![
                    leaf("(", "("),
                    node("Sum -> Sum [+-] Mul", vec![
                        leafify(&["Sum -> Mul"],
                                node("Mul -> Mul [*/] Pow", vec![
                            leafify(&["Mul -> Pow",
                                      "Pow -> Num",
                                      "Num -> Number"], leaf("Number", "2")),
                            leaf("[*/]", "*"),
                            leafify(&["Pow -> Num",
                                      "Num -> Number"], leaf("Number", "3")),
                        ])),
                        leaf("[+-]", "-"),
                        leafify(&["Mul -> Pow",
                                  "Pow -> Num",
                                  "Num -> Number"], leaf("Number", "4")),
                    ]),
                    leaf(")", ")")
                ]))
            ]);

        let grammar = grammar_math();
        let p = EarleyParser::new(grammar.clone());
        let pout = p.parse("1 + ( 2 * 3 - 4 )".split_whitespace()).unwrap();
        let trees = tree_evaler(grammar).eval_all(&pout).unwrap();
        assert_eq!(trees, vec![tree]);
    }
}

#[test]
fn grammar_ambiguous() {
    // Earley's corner case. We should only get 2 trees.
    // Broken parsers generate trees for bb and bbbb while parsing bbb.
    // S -> SS | b
    let grammar = GrammarBuilder::default()
      .nonterm("S")
      .terminal("b", |n| n == "b")
      .rule("S", &["S", "S"])
      .rule("S", &["b"])
      .into_grammar("S")
      .expect("Bad grammar");
    let p = EarleyParser::new(grammar.clone());
    let pout = p.parse("b b b".split_whitespace()).unwrap();
    let trees = tree_evaler(grammar).eval_all(&pout).unwrap();
    check_trees(&trees, vec![
        concat!(
            r#"Node("S -> S S", ["#,
                r#"Node("S -> S S", ["#,
                    r#"Node("S -> b", [Leaf("b", "b")]), "#,
                    r#"Node("S -> b", [Leaf("b", "b")])]), "#,
                r#"Node("S -> b", [Leaf("b", "b")])])"#),
        concat!(
            r#"Node("S -> S S", ["#,
                r#"Node("S -> b", [Leaf("b", "b")]), "#,
                r#"Node("S -> S S", ["#,
                    r#"Node("S -> b", [Leaf("b", "b")]), "#,
                    r#"Node("S -> b", [Leaf("b", "b")])])])"#)
    ]);
}

#[test]
fn grammar_ambiguous_epsilon() {
    // S -> SSX | b
    // X -> <e>
    let grammar = GrammarBuilder::default()
      .nonterm("S")
      .nonterm("X")
      .terminal("b", |n| n == "b")
      .rule("S", &["S", "S", "X"])
      .rule::<_, &str>("X", &[])
      .rule("S", &["b"])
      .into_grammar("S")
      .expect("Bad grammar");
    let p = EarleyParser::new(grammar.clone());
    let pout = p.parse("b b b".split_whitespace()).unwrap();
    let trees = tree_evaler(grammar).eval_all(&pout).unwrap();
    check_trees(&trees, vec![
        concat!(
            r#"Node("S -> S S X", ["#,
                r#"Node("S -> S S X", ["#,
                    r#"Node("S -> b", [Leaf("b", "b")]), "#,
                    r#"Node("S -> b", [Leaf("b", "b")]), "#,
                    r#"Node("X -> ", [])]), "#,
                r#"Node("S -> b", [Leaf("b", "b")]), "#,
                r#"Node("X -> ", [])])"#),
        concat!(
            r#"Node("S -> S S X", ["#,
                r#"Node("S -> b", [Leaf("b", "b")]), "#,
                r#"Node("S -> S S X", ["#,
                    r#"Node("S -> b", [Leaf("b", "b")]), "#,
                    r#"Node("S -> b", [Leaf("b", "b")]), "#,
                    r#"Node("X -> ", [])]), "#,
                r#"Node("X -> ", [])])"#)
    ]);
}


#[test]
fn left_recurse() {
    // S -> S + N | N
    // N -> [0-9]
    let grammar = GrammarBuilder::default()
      .nonterm("S")
      .nonterm("N")
      .terminal("[+]", |n| n == "+")
      .terminal("[0-9]", |n| "1234567890".contains(n))
      .rule("S", &["S", "[+]", "N"])
      .rule("S", &["N"])
      .rule("N", &["[0-9]"])
      .into_grammar("S")
      .expect("Bad grammar");
    let p = EarleyParser::new(grammar.clone());
    let pout = p.parse("1 + 2".split_whitespace()).unwrap();
    let tree = tree_evaler(grammar).eval(&pout).unwrap();
    check_trees(&vec![tree], vec![
        concat!(
            r#"Node("S -> S [+] N", ["#,
                r#"Node("S -> N", ["#,
                    r#"Node("N -> [0-9]", [Leaf("[0-9]", "1")])]), "#,
                r#"Leaf("[+]", "+"), "#,
                r#"Node("N -> [0-9]", [Leaf("[0-9]", "2")])])"#)
    ]);
}

#[test]
fn right_recurse() {
    // P -> N ^ P | N
    // N -> [0-9]
    let grammar = GrammarBuilder::default()
      .nonterm("P")
      .nonterm("N")
      .terminal("[^]", |n| n == "^")
      .terminal("[0-9]", |n| "1234567890".contains(n))
      .rule("P", &["N", "[^]", "P"])
      .rule("P", &["N"])
      .rule("N", &["[0-9]"])
      .into_grammar("P")
      .expect("Bad grammar");
    let p = EarleyParser::new(grammar.clone());
    let pout = p.parse("1 ^ 2".split_whitespace()).unwrap();
    let tree = tree_evaler(grammar).eval(&pout).unwrap();
    check_trees(&vec![tree], vec![
        concat!(
            r#"Node("P -> N [^] P", ["#,
                r#"Node("N -> [0-9]", [Leaf("[0-9]", "1")]), "#,
                r#"Leaf("[^]", "^"), "#,
                r#"Node("P -> N", ["#,
                    r#"Node("N -> [0-9]", [Leaf("[0-9]", "2")])])])"#)
    ]);
}

#[test]
fn grammar_example() {
    // Grammar for all words containing 'main'
    // Program   -> Letters 'm' 'a' 'i' 'n' Letters
    // Letters   -> oneletter Letters | <epsilon>
    // oneletter -> [a-zA-Z]
    let grammar = GrammarBuilder::default()
      .nonterm("Program")
      .nonterm("Letters")
      .terminal("oneletter", |l| l.len() == 1 &&
               l.chars().next().unwrap().is_alphabetic())
      .terminal("m", |l| l == "m")
      .terminal("a", |l| l == "a")
      .terminal("i", |l| l == "i")
      .terminal("n", |l| l == "n")
      .rule("Program", &["Letters", "m", "a", "i", "n", "Letters"])
      .rule("Letters", &["oneletter", "Letters"])
      .rule::<_, &str>("Letters", &[])
      .into_grammar("Program")
      .expect("Bad grammar");
    let p = EarleyParser::new(grammar);
    let input = "containsmainword".chars().map(|c| c.to_string());
    assert!(p.parse(input).is_ok());
}

#[test]
fn math_ambiguous_catalan() {
    // E -> E + E | n
    let grammar = GrammarBuilder::default()
      .nonterm("E")
      .terminal("+", |n| n == "+")
      .terminal("n", |n| "1234567890".contains(n))
      .rule("E", &["E", "+", "E"])
      .rule("E", &["n"])
      .into_grammar("E")
      .expect("Bad grammar");
    let p = EarleyParser::new(grammar.clone());
    let pout = p.parse("0 + 1 + 2 + 3 + 4 + 5".split_whitespace()).unwrap();
    let trees = tree_evaler(grammar).eval_all(&pout).unwrap();
    // number of trees here should match Catalan numbers
    // https://en.wikipedia.org/wiki/Catalan_number
    assert_eq!(trees.len(), 42);
}

#[test]
fn chained_terminals() {
    // E -> X + +  (and other variants)
    // X -> <epsilon>
    let rule_variants = vec![
        vec!["X", "+"],
        vec!["+", "X"],
        vec!["X", "+", "+"],
        vec!["+", "+", "X"],
        vec!["+", "X", "+"],
    ];
    for variant in rule_variants {
        let tokens = match variant.len() {
            2 => "+", 3 => "++", _ => unreachable!()
        };
        let g = GrammarBuilder::default()
          .nonterm("E")
          .nonterm("X")
          .terminal("+", |n| n == "+")
          .rule("E", &variant)
          .rule::<_, &str>("X", &[])
          .into_grammar("E")
          .expect("Bad grammar");
        let p = EarleyParser::new(g);
        assert!(p.parse(tokens.chars().map(|c| c.to_string())).is_ok());
    }
}

#[test]
fn natural_lang() {
    let grammar = GrammarBuilder::default()
      .terminal("N", |noun|
        vec!["flight", "banana", "time", "boy", "flies", "telescope"]
        .contains(&noun))
      .terminal("D", |det| vec!["the", "a", "an"].contains(&det))
      .terminal("V", |verb| vec!["book", "eat", "sleep", "saw"].contains(&verb))
      .terminal("P", |p| vec!["with", "in", "on", "at", "through"].contains(&p))
      .terminal("[name]", |name| vec!["john", "houston"].contains(&name))
      .nonterm("PP")
      .nonterm("NP")
      .nonterm("VP")
      .nonterm("S")
      .rule("NP", &["D", "N"])
      .rule("NP", &["[name]"])
      .rule("NP", &["NP", "PP"])
      .rule("PP", &["P", "NP"])
      .rule("VP", &["V", "NP"])
      .rule("VP", &["VP", "PP"])
      .rule("S", &["NP", "VP"])
      .rule("S", &["VP"])
      .into_grammar("S")
      .expect("Bad grammar");
    let p = EarleyParser::new(grammar);
    assert!(vec![
        "book the flight through houston",
        "john saw the boy with the telescope",
    ].iter().all(|input| p.parse(input.split_whitespace()).is_ok()));
}

mod small_math {
    use grammar::{Grammar, GrammarBuilder};
    use parser::EarleyParser;
    use trees::EarleyForest;
    use super::check_trees;

    fn small_math() -> Grammar {
        // E -> E + E | E * E | n
        GrammarBuilder::default()
          .nonterm("E")
          .terminal("+", |n| n == "+")
          .terminal("*", |n| n == "*")
          .terminal("n", |n| "1234567890".contains(n))
          .rule("E", &["E", "*", "E"])
          .rule("E", &["E", "+", "E"])
          .rule("E", &["n"])
          .into_grammar("E")
          .expect("Bad grammar")
    }

    #[test]
    fn eval_actions() {
        let mut ev = EarleyForest::new(|symbol, token| {
            match symbol {"n" => token.parse().unwrap(), _ => 0.0}
        });
        ev.action("E -> E + E", |nodes| nodes[0] + nodes[2]);
        ev.action("E -> E * E", |nodes| nodes[0] * nodes[2]);
        ev.action("E -> n", |nodes| nodes[0]);
        // parse 2 ambiguous results
        let input = "3 + 4 * 2".split_whitespace();
        let ps = EarleyParser::new(small_math()).parse(input).unwrap();
        let trees = ev.eval_all(&ps).unwrap();
        assert_eq!(trees.len(), 2);
        assert!(trees.contains(&11.0));
        assert!(trees.contains(&14.0));
    }

    #[test]
    fn build_ast() {
        #[derive(Clone, Debug)]
        enum AST { BinOP(Box<AST>, String, Box<AST>), Num(u64) }
        let mut ev = EarleyForest::new(|symbol, token| {
            match symbol {
                "n" => AST::Num(token.parse().unwrap()),
                _ => AST::Num(0)
            }
        });
        ev.action("E -> E * E", |nodes| AST::BinOP(
            Box::new(nodes[0].clone()), format!("*"), Box::new(nodes[2].clone())));
        ev.action("E -> E + E", |nodes| AST::BinOP(
            Box::new(nodes[0].clone()), format!("+"), Box::new(nodes[2].clone())));
        ev.action("E -> n", |nodes| nodes[0].clone());
        // check both possible parses
        let input = "3 + 4 * 2".split_whitespace();
        let ps = EarleyParser::new(small_math()).parse(input).unwrap();
        let trees = ev.eval_all(&ps).unwrap();
        check_trees(&trees, vec![
            r#"BinOP(BinOP(Num(3), "+", Num(4)), "*", Num(2))"#,
            r#"BinOP(Num(3), "+", BinOP(Num(4), "*", Num(2)))"#,
        ]);
    }

    #[test]
    fn build_sexpr() {
        #[derive(Clone,Debug)]
        pub enum Sexpr { Atom(String), List(Vec<Sexpr>) }
        let mut ev = EarleyForest::new(|_, tok| Sexpr::Atom(tok.to_string()));
        ev.action("E -> E + E", |nodes| Sexpr::List(nodes.clone()));
        ev.action("E -> E * E", |nodes| Sexpr::List(nodes.clone()));
        ev.action("E -> n", |nodes| nodes[0].clone());
        // check both trees
        let input = "3 + 4 * 2".split_whitespace();
        let output = EarleyParser::new(small_math()).parse(input).unwrap();
        let trees = ev.eval_all(&output).unwrap();
        check_trees(&trees, vec![
            r#"List([List([Atom("3"), Atom("+"), Atom("4")]), Atom("*"), Atom("2")])"#,
            r#"List([Atom("3"), Atom("+"), List([Atom("4"), Atom("*"), Atom("2")])])"#,
        ]);
    }
}

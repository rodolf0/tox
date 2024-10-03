#![deny(warnings)]

use crate::grammar::{GrammarBuilder, Grammar};
use crate::parser::EarleyParser;
use crate::trees::EarleyForest;
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
    for rule in g.rules {
        evaler.action(&rule.to_string(), move |nodes|
                      Tree::Node(rule.to_string(), nodes));
    }
    evaler
}

fn check_trees<T: fmt::Debug>(trees: &Vec<T>, expected: Vec<&str>) {
    use std::collections::HashSet;
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
    use crate::grammar::{Grammar, GrammarBuilder};

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
        use crate::parser::EarleyParser;
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

        let expected_tree =
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

        // Test evaluation of all possible parse trees (even though just 1)
        let evaler = tree_evaler(grammar);
        assert_eq!(evaler.eval_all_recursive(&pout).unwrap(), vec![expected_tree.clone()]);
        assert_eq!(evaler.eval_all(&pout).unwrap(), vec![expected_tree.clone()]);

        // Test evaluation of the unique parse tree.
        assert_eq!(evaler.eval(&pout).unwrap(), expected_tree);
        assert_eq!(evaler.eval_recursive(&pout).unwrap(), expected_tree);
    }
}

#[test]
fn grammar_ambiguous() {
    // Ambiguous, generates 2 trees
    // S -> A B, A -> A c | a, B -> c B | b, Input = w = a c b
    let grammar = GrammarBuilder::default()
      .nonterm("S")
      .nonterm("A")
      .nonterm("B")
      .terminal("a", |n| n == "a")
      .terminal("b", |n| n == "b")
      .terminal("c", |n| n == "c")
      .rule("S", &["A", "B"])
      .rule("A", &["A", "c"])
      .rule("A", &["a"])
      .rule("B", &["c", "B"])
      .rule("B", &["b"])
      .into_grammar("S")
      .expect("Bad grammar");
    let p = EarleyParser::new(grammar.clone());
    let pout = p.parse("a c b".split_whitespace()).unwrap();

    let expected_trees = vec![
        concat!(
            r#"Node("S -> A B", ["#,
                r#"Node("A -> A c", ["#,
                    r#"Node("A -> a", [Leaf("a", "a")]), "#,
                    r#"Leaf("c", "c")]), "#,
                r#"Node("B -> b", [Leaf("b", "b")])])"#),
        concat!(
            r#"Node("S -> A B", ["#,
                r#"Node("A -> a", [Leaf("a", "a")]), "#,
                r#"Node("B -> c B", ["#,
                    r#"Leaf("c", "c"), "#,
                    r#"Node("B -> b", [Leaf("b", "b")])])])"#)
    ];

    let evaler = tree_evaler(grammar);
    let trees = evaler.eval_all_recursive(&pout).unwrap();
    assert!(trees.len() == 2);
    check_trees(&trees, expected_trees.clone());

    let trees2 = evaler.eval_all(&pout).unwrap();
    assert!(trees2.len() == 2);
    check_trees(&trees2, expected_trees);
}

#[test]
fn earley_corner_case() {
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

    let expected_trees = vec![
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
    ];

    let evaler = tree_evaler(grammar);
    check_trees(&evaler.eval_all_recursive(&pout).unwrap(), expected_trees.clone());
    check_trees(&evaler.eval_all(&pout).unwrap(), expected_trees.clone());
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
      .rule("X", &[])
      .rule("S", &["b"])
      .into_grammar("S")
      .expect("Bad grammar");
    let p = EarleyParser::new(grammar.clone());
    let pout = p.parse("b b b".split_whitespace()).unwrap();

    let expected_trees = vec![
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
    ];

    let evaler = tree_evaler(grammar);
    check_trees(&evaler.eval_all_recursive(&pout).unwrap(), expected_trees.clone());
    check_trees(&evaler.eval_all(&pout).unwrap(), expected_trees.clone());
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
    let expected_tree = 
        concat!(
            r#"Node("S -> S [+] N", ["#,
                r#"Node("S -> N", ["#,
                    r#"Node("N -> [0-9]", [Leaf("[0-9]", "1")])]), "#,
                r#"Leaf("[+]", "+"), "#,
                r#"Node("N -> [0-9]", [Leaf("[0-9]", "2")])])"#);

    let evaler = tree_evaler(grammar);
    check_trees(&vec![evaler.eval(&pout).unwrap()], vec![expected_tree.clone()]);
    check_trees(&vec![evaler.eval_recursive(&pout).unwrap()], vec![expected_tree.clone()]);
    check_trees(&evaler.eval_all_recursive(&pout).unwrap(), vec![expected_tree.clone()]);
    check_trees(&evaler.eval_all(&pout).unwrap(), vec![expected_tree.clone()]);
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
    let expected_tree = 
        concat!(
            r#"Node("P -> N [^] P", ["#,
                r#"Node("N -> [0-9]", [Leaf("[0-9]", "1")]), "#,
                r#"Leaf("[^]", "^"), "#,
                r#"Node("P -> N", ["#,
                    r#"Node("N -> [0-9]", [Leaf("[0-9]", "2")])])])"#);

    let evaler = tree_evaler(grammar);
    check_trees(&vec![evaler.eval(&pout).unwrap()], vec![expected_tree.clone()]);
    check_trees(&vec![evaler.eval_recursive(&pout).unwrap()], vec![expected_tree.clone()]);
    check_trees(&evaler.eval_all_recursive(&pout).unwrap(), vec![expected_tree.clone()]);
    check_trees(&evaler.eval_all(&pout).unwrap(), vec![expected_tree.clone()]);
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
    let ef = tree_evaler(grammar);
    // number of trees here should match Catalan numbers
    // https://en.wikipedia.org/wiki/Catalan_number
    // https://en.wikipedia.org/wiki/Associahedron
    assert_eq!(ef.eval_all_recursive(&pout).unwrap().len(), 42);
    assert_eq!(ef.eval_all(&pout).unwrap().len(), 42);
}

#[test]
fn trigger_has_multiple_bp() {
    // E -> E + n | n + E | n
    // SpanSource::Completion has multiple sources.
    // This is a scenario that would trigger that but we're not asserting.
    let grammar = GrammarBuilder::default()
      .nonterm("E")
      .terminal("+", |n| n == "+")
      .terminal("n", |n| "1234567890".contains(n))
      .rule("E", &["E", "+", "n"])
      .rule("E", &["n", "+", "E"])
      .rule("E", &["n"])
      .into_grammar("E")
      .expect("Bad grammar");
    let p = EarleyParser::new(grammar.clone());
    let pout = p.parse("3 + 4 + 5 + 6".split_whitespace()).unwrap();
    let ef = tree_evaler(grammar);
    assert_eq!(ef.eval_all_recursive(&pout).unwrap().len(), 8);
    assert_eq!(ef.eval_all(&pout).unwrap().len(), 8);
}

mod small_math {
    use crate::grammar::{Grammar, GrammarBuilder};
    use crate::parser::EarleyParser;
    use crate::trees::EarleyForest;
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
        let trees = ev.eval_all_recursive(&ps).unwrap();
        let trees2 = ev.eval_all(&ps).unwrap();
        assert_eq!(trees.len(), 2);
        assert_eq!(trees2.len(), 2);
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
        check_trees(&ev.eval_all_recursive(&ps).unwrap(), vec![
            r#"BinOP(BinOP(Num(3), "+", Num(4)), "*", Num(2))"#,
            r#"BinOP(Num(3), "+", BinOP(Num(4), "*", Num(2)))"#,
        ]);
        check_trees(&ev.eval_all(&ps).unwrap(), vec![
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
        check_trees(&ev.eval_all_recursive(&output).unwrap(), vec![
            r#"List([List([Atom("3"), Atom("+"), Atom("4")]), Atom("*"), Atom("2")])"#,
            r#"List([Atom("3"), Atom("+"), List([Atom("4"), Atom("*"), Atom("2")])])"#,
        ]);
        check_trees(&ev.eval_all(&output).unwrap(), vec![
            r#"List([List([Atom("3"), Atom("+"), Atom("4")]), Atom("*"), Atom("2")])"#,
            r#"List([Atom("3"), Atom("+"), List([Atom("4"), Atom("*"), Atom("2")])])"#,
        ]);
    }
}


mod earley_recognizer {
    use crate::grammar::GrammarBuilder;
    use super::EarleyParser;

    fn good(parser: &EarleyParser, input: &str) {
        assert!(parser.parse(input.split_whitespace()).is_ok());
    }

    fn fail(parser: &EarleyParser, input: &str) {
        assert_eq!(parser.parse(input.split_whitespace()).unwrap_err(),
                   "Parse Error: No Rule completes");
    }

    #[test]
    fn partial_parse() {
        let grammar = GrammarBuilder::default()
            .nonterm("Start")
            .terminal("+", |n| n == "+")
            .rule("Start", &["+", "+"])
            .into_grammar("Start")
            .expect("Bad Grammar");
        let p = EarleyParser::new(grammar);
        fail(&p, "+ + +");
        good(&p, "+ +");

        let grammar = GrammarBuilder::default()
          .nonterm("Sum")
          .nonterm("Num")
          .terminal("Number", |n| n.chars().all(|c| "1234".contains(c)))
          .terminal("[+-]", |n| n.len() == 1 && "+-".contains(n))
          .rule("Sum", &["Sum", "[+-]", "Num"])
          .rule("Sum", &["Num"])
          .rule("Num", &["Number"])
          .into_grammar("Sum")
          .expect("Bad Grammar");
        let p = EarleyParser::new(grammar);
        fail(&p, "1 +");
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
        let p = EarleyParser::new(grammar);
        good(&p, "1 + 2");
        good(&p, "1 + 2 + 3");
        fail(&p, "1 2 + 3");
        fail(&p, "+ 3");
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
        let p = EarleyParser::new(grammar);
        good(&p, "1 ^ 2");
        fail(&p, "3 ^ ");
        good(&p, "1 ^ 2 ^ 4");
        good(&p, "1 ^ 2 ^ 4 ^ 5");
        fail(&p, "1 2 ^ 4");
    }

    #[test]
    fn bogus_empty() {
        // A -> <empty> | B
        // B -> A
        // http://loup-vaillant.fr/tutorials/earley-parsing/empty-rules
        let grammar = GrammarBuilder::default()
          .nonterm("A")
          .nonterm("B")
          .rule("A", &[])
          .rule("A", &vec!["B"])
          .rule("B", &vec!["A"])
          .into_grammar("A")
          .expect("Bad grammar");
        let p = EarleyParser::new(grammar);
        good(&p, "");
        good(&p, " ");
        fail(&p, "X");
    }

    #[test]
    fn epsilon_balanced() {
        // Grammar for balanced parenthesis
        // P  -> '(' P ')' | P P | <epsilon>
        let grammar = GrammarBuilder::default()
          .nonterm("P")
          .terminal("(", |l| l == "(")
          .terminal(")", |l| l == ")")
          .rule("P", &["(", "P", ")"])
          .rule("P", &["P", "P"])
          .rule("P", &[])
          .into_grammar("P")
          .expect("Bad grammar");
        let p = EarleyParser::new(grammar);
        good(&p, "");
        good(&p, "( )");
        good(&p, "( ( ) )");
        fail(&p, "( ) )");
        fail(&p, ")");
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
        good(&p, "book the flight through houston");
        good(&p, "john saw the boy with the telescope");
    }

    #[test]
    fn epsilon_variants() {
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
                2 => "+", 3 => "+ +", _ => unreachable!()
            };
            let g = GrammarBuilder::default()
              .nonterm("E")
              .nonterm("X")
              .terminal("+", |n| n == "+")
              .rule("E", &variant)
              .rule("X", &[])
              .into_grammar("E")
              .expect("Bad grammar");
            let p = EarleyParser::new(g);
            good(&p, tokens);
        }
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
          .rule("Letters", &[])
          .into_grammar("Program")
          .expect("Bad grammar");
        let p = EarleyParser::new(grammar);
        let input = "containsmainword".chars().map(|c| c.to_string());
        assert!(p.parse(input).is_ok());
    }
}

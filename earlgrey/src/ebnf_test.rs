#![deny(warnings)]

use crate::ebnf::EbnfGrammarParser;
use crate::earley::{EarleyForest, EarleyParser, Grammar};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Tree {
    // ("[+-]", "+")
    Leaf(String, String),
    // ("E -> E [+-] E", [...])
    Node(String, Vec<Tree>),
}

pub fn ast_parser<InputIter>(
    grammar: Grammar,
) -> Result<impl Fn(InputIter) -> Result<Vec<Tree>, String>, String>
where
    InputIter: Iterator,
    InputIter::Item: AsRef<str> + std::fmt::Debug,
{
    let mut tree_builder =
        EarleyForest::new(|sym, tok| Tree::Leaf(sym.to_string(), tok.to_string()));

    for rule in grammar.rules.iter().map(|r| r.to_string()) {
        tree_builder.action(&rule.clone(), move |nodes| Tree::Node(rule.clone(), nodes));
    }

    let parser = EarleyParser::new(grammar);
    Ok(move |tokenizer| tree_builder.eval_all(&parser.parse(tokenizer)?))
}

fn check_trees<T: fmt::Debug>(trees: &Vec<T>, expected: Vec<&str>) {
    use std::collections::HashSet;
    assert_eq!(trees.len(), expected.len());

    let strip_ws = |s: &str| -> String {
        let mut in_string = false;
        let mut escaped = false;
        s.chars().filter(|&c| {
            let is_quote = c == '"' && !escaped;
            if is_quote { in_string = !in_string; }
            escaped = c == '\\' && !escaped;
            in_string || is_quote || !c.is_whitespace()
        }).collect()
    };

    let mut expect: HashSet<String> = expected.into_iter().map(strip_ws).collect();
    for t in trees {
        let debug_string = format!("{:?}", t);
        let stripped = strip_ws(&debug_string);
        if !expect.contains(&stripped) {
            eprintln!("Trying to remove {}", debug_string);
            for items in expect.iter() {
                eprintln!("  possible item: {}", items);
            }
        }
        assert!(expect.remove(&stripped), "Missing from expected: {}", debug_string);
    }
    assert_eq!(0, expect.len());
}

#[test]
fn minimal_parser() {
    let g = r#" Number := "0" ; "#;
    let grammar = EbnfGrammarParser::new(&g, "Number").into_grammar().unwrap();
    let parser = ast_parser(grammar).unwrap();

    let trees = parser(["0"].iter()).unwrap();
    check_trees(&trees, vec![r#"
        Node("Number -> 0", [
            Leaf("0", "0")])
    "#]);
}

#[test]
fn arith_parser() {
    let g = r#"
        expr := Number
              | expr "+" Number ;

        Number := "0" | "1" | "2" | "3" ;
    "#;
    let grammar = EbnfGrammarParser::new(&g, "expr").into_grammar().unwrap();
    let parser = ast_parser(grammar).unwrap();

    let trees = parser("3 + 2 + 1".split_whitespace()).unwrap();
    check_trees(
        &trees,
        vec![r#"
                 Node("expr -> expr + Number", [
                     Node("expr -> expr + Number", [
                         Node("expr -> Number", [
                             Node("Number -> 3", [Leaf("3", "3")])]), 
                         Leaf("+", "+"), 
                         Node("Number -> 2", [Leaf("2", "2")])]), 
                     Leaf("+", "+"), 
                     Node("Number -> 1", [Leaf("1", "1")])])
             "#],
    );
}

#[test]
fn repetition() {
    let g = r#"
        arg := b { "," b } ;
        b := "0" | "1" ;
    "#;
    let grammar = EbnfGrammarParser::new(&g, "arg").into_grammar().unwrap();
    let parser = ast_parser(grammar).unwrap();

    let trees = parser("1 , 0 , 1".split_whitespace()).unwrap();
    check_trees(
        &trees,
        vec![r#"
            Node("arg -> b {,b}", [
                Node("b -> 1", [
                    Leaf("1", "1")]),
                Node("{,b} -> {,b} , b", [
                    Node("{,b} -> {,b} , b", [
                        Node("{,b} -> ", []),
                        Leaf(",", ","),
                        Node("b -> 0", [Leaf("0", "0")])
                    ]),
                    Leaf(",", ","),
                    Node("b -> 1", [Leaf("1", "1")])
                ])
            ])
        "#]
    );
}

#[test]
fn repetition_tagged() {
    let g = r#"
        arg := b { "," b } @x;
        b := "0" | "1" ;
    "#;
    let grammar = EbnfGrammarParser::new(&g, "arg").into_grammar().unwrap();
    let parser = ast_parser(grammar).unwrap();

    let trees = parser("1 , 0 , 1".split_whitespace()).unwrap();
    check_trees(
        &trees,
        vec![r#"
            Node("arg -> b @x", [
                Node("b -> 1", [Leaf("1", "1")]),
                Node("@x -> @x , b", [
                    Node("@x -> @x , b", [
                        Node("@x -> ", []),
                        Leaf(",", ","),
                        Node("b -> 0", [Leaf("0", "0")])
                    ]),
                    Leaf(",", ","),
                    Node("b -> 1", [Leaf("1", "1")])
                ])
            ])
        "#]
    );
}

#[test]
fn option() {
    let g = r#"
        complex := d [ "i" ];
        d := "0" | "1" | "2";
    "#;
    let grammar = EbnfGrammarParser::new(&g, "complex")
        .into_grammar()
        .unwrap();
    let parser = ast_parser(grammar).unwrap();

    let trees = parser(["1"].iter()).unwrap();
    check_trees(
        &trees,
        vec![r#"
                 Node("complex -> d [i]", [
                     Node("d -> 1", [Leaf("1", "1")]), 
                     Node("[i] -> ", [])])
             "#],
    );

    let trees = parser(["2", "i"].iter()).unwrap();
    check_trees(
        &trees,
        vec![r#"
                 Node("complex -> d [i]", [
                     Node("d -> 2", [Leaf("2", "2")]), 
                     Node("[i] -> i", [Leaf("i", "i")])])
             "#],
    );

    assert!(parser(["2", "i", "i"].iter()).is_err());
}

#[test]
fn option_tagged() {
    let g = r#"
        complex := d [ "i" ] @x;
        d := "0" | "1" | "2";
    "#;
    let grammar = EbnfGrammarParser::new(&g, "complex")
        .into_grammar()
        .unwrap();
    let parser = ast_parser(grammar).unwrap();

    let trees = parser(["1"].iter()).unwrap();
    check_trees(
        &trees,
        vec![r#"
                 Node("complex -> d @x", [
                     Node("d -> 1", [Leaf("1", "1")]), 
                     Node("@x -> ", [])])
             "#],
    );

    let trees = parser(["2", "i"].iter()).unwrap();
    check_trees(
        &trees,
        vec![r#"
                 Node("complex -> d @x", [
                     Node("d -> 2", [Leaf("2", "2")]), 
                     Node("@x -> i", [Leaf("i", "i")])])
             "#],
    );

    assert!(parser(["2", "i", "i"].iter()).is_err());
}

#[test]
fn grouping() {
    let g = r#"
        row := ("a" | "b") ("0" | "1") ;
    "#;
    let grammar = EbnfGrammarParser::new(&g, "row").into_grammar().unwrap();
    let parser = ast_parser(grammar).unwrap();

    let trees = parser(["b", "1"].iter()).unwrap();
    check_trees(
        &trees,
        vec![r#"
                 Node("row -> (a|b) (0|1)", [
                     Node("(a|b) -> b", [Leaf("b", "b")]), 
                     Node("(0|1) -> 1", [Leaf("1", "1")])])
             "#],
    );

    let trees = parser(["a", "0"].iter()).unwrap();
    check_trees(
        &trees,
        vec![r#"
                 Node("row -> (a|b) (0|1)", [
                     Node("(a|b) -> a", [Leaf("a", "a")]), 
                     Node("(0|1) -> 0", [Leaf("0", "0")])])
             "#],
    );

    assert!(parser(["a", "b"].iter()).is_err());
    assert!(parser(["0", "1"].iter()).is_err());
}

#[test]
fn grouping_tagged() {
    let g = r#"
        row := ("a" | "b") @x ("0" | "1") @y;
    "#;
    let grammar = EbnfGrammarParser::new(&g, "row").into_grammar().unwrap();
    let parser = ast_parser(grammar).unwrap();

    let trees = parser(["b", "1"].iter()).unwrap();
    check_trees(
        &trees,
        vec![r#"
                 Node("row -> @x @y", [
                     Node("@x -> b", [Leaf("b", "b")]), 
                     Node("@y -> 1", [Leaf("1", "1")])])
             "#],
    );

    let trees = parser(["a", "0"].iter()).unwrap();
    check_trees(
        &trees,
        vec![r#"
                 Node("row -> @x @y", [
                     Node("@x -> a", [Leaf("a", "a")]), 
                     Node("@y -> 0", [Leaf("0", "0")])])
             "#],
    );

    assert!(parser(["a", "b"].iter()).is_err());
    assert!(parser(["0", "1"].iter()).is_err());
}

#[test]
fn mixed() {
    let g = r#"
        row := "a" [ "b" ] ("0" | "1") [ "c" ];
    "#;
    let grammar = EbnfGrammarParser::new(&g, "row").into_grammar().unwrap();
    let parser = ast_parser(grammar).unwrap();

    let trees = parser(["a", "0"].iter()).unwrap();
    check_trees(
        &trees,
        vec![r#"
                 Node("row -> a [b] (0|1) [c]", [
                     Leaf("a", "a"), 
                     Node("[b] -> ", []), 
                     Node("(0|1) -> 0", [Leaf("0", "0")]), 
                     Node("[c] -> ", [])])
             "#],
    );

    let trees = parser(["a", "b", "1"].iter()).unwrap();
    check_trees(
        &trees,
        vec![r#"
                 Node("row -> a [b] (0|1) [c]", [
                     Leaf("a", "a"), 
                     Node("[b] -> b", [Leaf("b", "b")]), 
                     Node("(0|1) -> 1", [Leaf("1", "1")]), 
                     Node("[c] -> ", [])])
             "#],
    );

    let trees = parser(["a", "1", "c"].iter()).unwrap();
    check_trees(
        &trees,
        vec![r#"
                 Node("row -> a [b] (0|1) [c]", [
                     Leaf("a", "a"), 
                     Node("[b] -> ", []), 
                     Node("(0|1) -> 1", [Leaf("1", "1")]), 
                     Node("[c] -> c", [Leaf("c", "c")])])
             "#],
    );

    assert!(parser(["a", "b"].iter()).is_err());
    assert!(parser(["0", "1"].iter()).is_err());
    assert!(parser(["a", "b", "0", "d"].iter()).is_err());
    assert!(parser(["a", "b", "0"].iter()).is_ok());
}

#[test]
fn mixed_tagged() {
    let g = r#"
        row := "a" [ "b" ]@x ("0" | "1")@y [ "c" ]@z;
    "#;

    let grammar = EbnfGrammarParser::new(&g, "row").into_grammar().unwrap();
    let parser = ast_parser(grammar).unwrap();

    let trees = parser(["a", "0"].iter()).unwrap();
    check_trees(
        &trees,
        vec![r#"
                 Node("row -> a @x @y @z", [
                     Leaf("a", "a"), 
                     Node("@x -> ", []), 
                     Node("@y -> 0", [Leaf("0", "0")]), 
                     Node("@z -> ", [])])
             "#],
    );

    let trees = parser(["a", "b", "1"].iter()).unwrap();
    check_trees(
        &trees,
        vec![r#"
                 Node("row -> a @x @y @z", [
                     Leaf("a", "a"), 
                     Node("@x -> b", [Leaf("b", "b")]), 
                     Node("@y -> 1", [Leaf("1", "1")]), 
                     Node("@z -> ", [])])
             "#],
    );

    let trees = parser(["a", "1", "c"].iter()).unwrap();
    check_trees(
        &trees,
        vec![r#"
                 Node("row -> a @x @y @z", [
                     Leaf("a", "a"), 
                     Node("@x -> ", []), 
                     Node("@y -> 1", [Leaf("1", "1")]), 
                     Node("@z -> c", [Leaf("c", "c")])])
             "#],
    );

    assert!(parser(["a", "b"].iter()).is_err());
    assert!(parser(["0", "1"].iter()).is_err());
    assert!(parser(["a", "b", "0", "d"].iter()).is_err());
    assert!(parser(["a", "b", "0"].iter()).is_ok());
}

#[test]
fn plug_terminal() {
    use std::str::FromStr;
    let g = r#"
        expr := Number
              | expr "+" Number ;
    "#;
    let grammar = EbnfGrammarParser::new(&g, "expr")
        .plug_terminal("Number", |i| i8::from_str(i).is_ok())
        .into_grammar()
        .unwrap();

    let parser = ast_parser(grammar).unwrap();

    let trees = parser(["3", "+", "1"].iter()).unwrap();
    check_trees(
        &trees,
        vec![r#"
                 Node("expr -> expr + Number", [
                     Node("expr -> Number", [Leaf("Number", "3")]), 
                     Leaf("+", "+"), 
                     Leaf("Number", "1")])
             "#],
    );
}

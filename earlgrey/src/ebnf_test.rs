#![deny(warnings)]

use super::ebnf::EbnfGrammarParser;
use super::{Grammar, EarleyForest, EarleyParser};
use std::fmt;

#[derive(Debug,Clone,PartialEq)]
pub enum Tree {
    // ("[+-]", "+")
    Leaf(String, String),
    // ("E -> E [+-] E", [...])
    Node(String, Vec<Tree>),
}

pub fn ast_parser<InputIter>(grammar: Grammar)
    -> Result<impl Fn(InputIter) -> Result<Vec<Tree>, String>, String>
        where InputIter: Iterator, InputIter::Item: AsRef<str> + std::fmt::Debug
{
    let mut tree_builder = EarleyForest::new(
        |sym, tok| Tree::Leaf(sym.to_string(), tok.to_string()));

    for rule in grammar.rules.iter().map(|r| r.to_string()) {
        tree_builder.action(
            &rule.clone(), move |nodes| Tree::Node(rule.clone(), nodes));
    }

    let parser = EarleyParser::new(grammar);
    Ok(move |tokenizer| tree_builder.eval_all(&parser.parse(tokenizer)?))
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

#[test]
fn minimal_parser() {
    let g = r#" Number := "0" ; "#;
    let grammar = EbnfGrammarParser::new(&g, "Number")
        .into_grammar().unwrap();
    let parser = ast_parser(grammar).unwrap();

    let trees = parser(["0"].iter()).unwrap();
    check_trees(&trees, vec![r#"Node("Number -> 0", [Leaf("0", "0")])"#]);
}

#[test]
fn arith_parser() {
    let g = r#"
        expr := Number
              | expr "+" Number ;

        Number := "0" | "1" | "2" | "3" ;
    "#;
    let grammar = EbnfGrammarParser::new(&g, "expr")
        .into_grammar().unwrap();
    let parser = ast_parser(grammar).unwrap();

    let trees = parser("3 + 2 + 1".split_whitespace()).unwrap();
    check_trees(&trees, vec![
        concat!(
            r#"Node("expr -> expr + Number", ["#,
                r#"Node("expr -> expr + Number", ["#,
                    r#"Node("expr -> Number", ["#,
                        r#"Node("Number -> 3", [Leaf("3", "3")])]), "#,
                    r#"Leaf("+", "+"), "#,
                    r#"Node("Number -> 2", [Leaf("2", "2")])]), "#,
                r#"Leaf("+", "+"), "#,
                r#"Node("Number -> 1", [Leaf("1", "1")])])"#)
    ]);
}

#[test]
fn repetition() {
    let g = r#"
        arg := b { "," b } ;
        b := "0" | "1" ;
    "#;
    let grammar = EbnfGrammarParser::new(&g, "arg")
        .into_grammar().unwrap();
    let parser = ast_parser(grammar).unwrap();

    let trees = parser("1 , 0 , 1".split_whitespace()).unwrap();
    check_trees(&trees, vec![
        concat!(
            r#"Node("arg -> b <Uniq-4>", ["#,
                r#"Node("b -> 1", [Leaf("1", "1")]), "#,
                r#"Node("<Uniq-4> -> , b <Uniq-4>", ["#,
                    r#"Leaf(",", ","), "#,
                    r#"Node("b -> 0", [Leaf("0", "0")]), "#,
                    r#"Node("<Uniq-4> -> , b <Uniq-4>", ["#,
                        r#"Leaf(",", ","), "#,
                        r#"Node("b -> 1", [Leaf("1", "1")]), "#,
                        r#"Node("<Uniq-4> -> ", [])])])])"#)
    ]);
}

#[test]
fn repetition_tagged() {
    let g = r#"
        arg := b { "," b } @x;
        b := "0" | "1" ;
    "#;
    let grammar = EbnfGrammarParser::new(&g, "arg")
        .into_grammar().unwrap();
    let parser = ast_parser(grammar).unwrap();

    let trees = parser("1 , 0 , 1".split_whitespace()).unwrap();
    check_trees(&trees, vec![
        concat!(
            r#"Node("arg -> b @x", ["#,
                r#"Node("b -> 1", [Leaf("1", "1")]), "#,
                r#"Node("@x -> , b @x", ["#,
                    r#"Leaf(",", ","), "#,
                    r#"Node("b -> 0", [Leaf("0", "0")]), "#,
                    r#"Node("@x -> , b @x", ["#,
                        r#"Leaf(",", ","), "#,
                        r#"Node("b -> 1", [Leaf("1", "1")]), "#,
                        r#"Node("@x -> ", [])])])])"#)
    ]);
}

#[test]
fn option() {
    let g = r#"
        complex := d [ "i" ];
        d := "0" | "1" | "2";
    "#;
    let grammar = EbnfGrammarParser::new(&g, "complex")
        .into_grammar().unwrap();
    let parser = ast_parser(grammar).unwrap();

    let trees = parser(["1"].iter()).unwrap();
    check_trees(&trees, vec![
        concat!(
            r#"Node("complex -> d <Uniq-5>", ["#,
                r#"Node("d -> 1", [Leaf("1", "1")]), "#,
                r#"Node("<Uniq-5> -> ", [])])"#)
    ]);

    let trees = parser(["2", "i"].iter()).unwrap();
    check_trees(&trees, vec![
        concat!(
            r#"Node("complex -> d <Uniq-5>", ["#,
                r#"Node("d -> 2", [Leaf("2", "2")]), "#,
                r#"Node("<Uniq-5> -> i", [Leaf("i", "i")])])"#)
    ]);

    assert!(parser(["2", "i", "i"].iter()).is_err());
}

#[test]
fn option_tagged() {
    let g = r#"
        complex := d [ "i" ] @x;
        d := "0" | "1" | "2";
    "#;
    let grammar = EbnfGrammarParser::new(&g, "complex")
        .into_grammar().unwrap();
    let parser = ast_parser(grammar).unwrap();

    let trees = parser(["1"].iter()).unwrap();
    check_trees(&trees, vec![
        concat!(
            r#"Node("complex -> d @x", ["#,
                r#"Node("d -> 1", [Leaf("1", "1")]), "#,
                r#"Node("@x -> ", [])])"#)
    ]);

    let trees = parser(["2", "i"].iter()).unwrap();
    check_trees(&trees, vec![
        concat!(
            r#"Node("complex -> d @x", ["#,
                r#"Node("d -> 2", [Leaf("2", "2")]), "#,
                r#"Node("@x -> i", [Leaf("i", "i")])])"#)
    ]);

    assert!(parser(["2", "i", "i"].iter()).is_err());
}

#[test]
fn grouping() {
    let g = r#"
        row := ("a" | "b") ("0" | "1") ;
    "#;
    let grammar = EbnfGrammarParser::new(&g, "row")
        .into_grammar().unwrap();
    let parser = ast_parser(grammar).unwrap();

    let trees = parser(["b", "1"].iter()).unwrap();
    check_trees(&trees, vec![
        concat!(
            r#"Node("row -> <Uniq-5> <Uniq-2>", ["#,
                r#"Node("<Uniq-5> -> b", [Leaf("b", "b")]), "#,
                r#"Node("<Uniq-2> -> 1", [Leaf("1", "1")])])"#)
    ]);

    let trees = parser(["a", "0"].iter()).unwrap();
    check_trees(&trees, vec![
        concat!(
            r#"Node("row -> <Uniq-5> <Uniq-2>", ["#,
                r#"Node("<Uniq-5> -> a", [Leaf("a", "a")]), "#,
                r#"Node("<Uniq-2> -> 0", [Leaf("0", "0")])])"#)
    ]);

    assert!(parser(["a", "b"].iter()).is_err());
    assert!(parser(["0", "1"].iter()).is_err());
}

#[test]
fn grouping_tagged() {
    let g = r#"
        row := ("a" | "b") @x ("0" | "1") @y;
    "#;
    let grammar = EbnfGrammarParser::new(&g, "row")
        .into_grammar().unwrap();
    let parser = ast_parser(grammar).unwrap();

    let trees = parser(["b", "1"].iter()).unwrap();
    check_trees(&trees, vec![
        concat!(
            r#"Node("row -> @x @y", ["#,
                r#"Node("@x -> b", [Leaf("b", "b")]), "#,
                r#"Node("@y -> 1", [Leaf("1", "1")])])"#)
    ]);

    let trees = parser(["a", "0"].iter()).unwrap();
    check_trees(&trees, vec![
        concat!(
            r#"Node("row -> @x @y", ["#,
                r#"Node("@x -> a", [Leaf("a", "a")]), "#,
                r#"Node("@y -> 0", [Leaf("0", "0")])])"#)
    ]);

    assert!(parser(["a", "b"].iter()).is_err());
    assert!(parser(["0", "1"].iter()).is_err());
}

#[test]
fn mixed() {
    let g = r#"
        row := "a" [ "b" ] ("0" | "1") [ "c" ];
    "#;
    let grammar = EbnfGrammarParser::new(&g, "row")
        .into_grammar().unwrap();
    let parser = ast_parser(grammar).unwrap();

    let trees = parser(["a", "0"].iter()).unwrap();
    check_trees(&trees, vec![
        concat!(
            r#"Node("row -> a <Uniq-6> <Uniq-4> <Uniq-1>", ["#,
                r#"Leaf("a", "a"), "#,
                r#"Node("<Uniq-6> -> ", []), "#,
                r#"Node("<Uniq-4> -> 0", [Leaf("0", "0")]), "#,
                r#"Node("<Uniq-1> -> ", [])])"#)
    ]);

    let trees = parser(["a", "b", "1"].iter()).unwrap();
    check_trees(&trees, vec![
        concat!(
            r#"Node("row -> a <Uniq-6> <Uniq-4> <Uniq-1>", ["#,
                r#"Leaf("a", "a"), "#,
                r#"Node("<Uniq-6> -> b", [Leaf("b", "b")]), "#,
                r#"Node("<Uniq-4> -> 1", [Leaf("1", "1")]), "#,
                r#"Node("<Uniq-1> -> ", [])])"#)
    ]);

    let trees = parser(["a", "1", "c"].iter()).unwrap();
    check_trees(&trees, vec![
        concat!(
            r#"Node("row -> a <Uniq-6> <Uniq-4> <Uniq-1>", ["#,
                r#"Leaf("a", "a"), "#,
                r#"Node("<Uniq-6> -> ", []), "#,
                r#"Node("<Uniq-4> -> 1", [Leaf("1", "1")]), "#,
                r#"Node("<Uniq-1> -> c", [Leaf("c", "c")])])"#)
    ]);

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

    let grammar = EbnfGrammarParser::new(&g, "row")
        .into_grammar().unwrap();
    let parser = ast_parser(grammar).unwrap();

    let trees = parser(["a", "0"].iter()).unwrap();
    check_trees(&trees, vec![
        concat!(
            r#"Node("row -> a @x @y @z", ["#,
                r#"Leaf("a", "a"), "#,
                r#"Node("@x -> ", []), "#,
                r#"Node("@y -> 0", [Leaf("0", "0")]), "#,
                r#"Node("@z -> ", [])])"#)
    ]);

    let trees = parser(["a", "b", "1"].iter()).unwrap();
    check_trees(&trees, vec![
        concat!(
            r#"Node("row -> a @x @y @z", ["#,
                r#"Leaf("a", "a"), "#,
                r#"Node("@x -> b", [Leaf("b", "b")]), "#,
                r#"Node("@y -> 1", [Leaf("1", "1")]), "#,
                r#"Node("@z -> ", [])])"#)
    ]);

    let trees = parser(["a", "1", "c"].iter()).unwrap();
    check_trees(&trees, vec![
        concat!(
            r#"Node("row -> a @x @y @z", ["#,
                r#"Leaf("a", "a"), "#,
                r#"Node("@x -> ", []), "#,
                r#"Node("@y -> 1", [Leaf("1", "1")]), "#,
                r#"Node("@z -> c", [Leaf("c", "c")])])"#)
    ]);

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
        .into_grammar().unwrap();

    let parser = ast_parser(grammar).unwrap();

    let trees = parser(["3", "+", "1"].iter()).unwrap();
    check_trees(&trees, vec![
        concat!(
            r#"Node("expr -> expr + Number", ["#,
                r#"Node("expr -> Number", [Leaf("Number", "3")]), "#,
                r#"Leaf("+", "+"), "#,
                r#"Leaf("Number", "1")])"#)
    ]);
}

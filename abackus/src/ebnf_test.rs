#![deny(warnings)]

use ebnf::{ebnf_grammar, ParserBuilder};
use std::fmt;

#[test]
fn build_ebnf_grammar() {
    ebnf_grammar();
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


#[test]
fn minimal_parser() {
    let g = r#" Number := "0" ; "#;
    let parser = ParserBuilder::default().treeficator(g, "Number");
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
    let parser = ParserBuilder::default().treeficator(&g, "expr");
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
    let parser = ParserBuilder::default().treeficator(&g, "arg");
    let trees = parser("1 , 0 , 1".split_whitespace()).unwrap();
    check_trees(&trees, vec![
        concat!(
            r#"Node("arg -> b <Uniq-3>", ["#,
                r#"Node("b -> 1", [Leaf("1", "1")]), "#,
                r#"Node("<Uniq-3> -> , b <Uniq-3>", ["#,
                    r#"Leaf(",", ","), "#,
                    r#"Node("b -> 0", [Leaf("0", "0")]), "#,
                    r#"Node("<Uniq-3> -> , b <Uniq-3>", ["#,
                        r#"Leaf(",", ","), "#,
                        r#"Node("b -> 1", [Leaf("1", "1")]), "#,
                        r#"Node("<Uniq-3> -> ", [])])])])"#)
    ]);
}

#[test]
fn option() {
    let g = r#"
        complex := d [ "i" ];
        d := "0" | "1" | "2";
    "#;
    let parser = ParserBuilder::default().treeficator(&g, "complex");
    let trees = parser(["1"].iter()).unwrap();
    check_trees(&trees, vec![
        concat!(
            r#"Node("complex -> d <Uniq-3>", ["#,
                r#"Node("d -> 1", [Leaf("1", "1")]), "#,
                r#"Node("<Uniq-3> -> ", [])])"#)
    ]);

    let trees = parser(["2", "i"].iter()).unwrap();
    check_trees(&trees, vec![
        concat!(
            r#"Node("complex -> d <Uniq-3>", ["#,
                r#"Node("d -> 2", [Leaf("2", "2")]), "#,
                r#"Node("<Uniq-3> -> i", [Leaf("i", "i")])])"#)
    ]);

    assert!(parser(["2", "i", "i"].iter()).is_err());
}

#[test]
fn grouping() {
    let g = r#"
        row := ("a" | "b") ("0" | "1") ;
    "#;
    let parser = ParserBuilder::default().treeficator(&g, "row");
    let trees = parser(["b", "1"].iter()).unwrap();
    check_trees(&trees, vec![
        concat!(
            r#"Node("row -> <Uniq-3> <Uniq-6>", ["#,
                r#"Node("<Uniq-3> -> b", [Leaf("b", "b")]), "#,
                r#"Node("<Uniq-6> -> 1", [Leaf("1", "1")])])"#)
    ]);

    let trees = parser(["a", "0"].iter()).unwrap();
    check_trees(&trees, vec![
        concat!(
            r#"Node("row -> <Uniq-3> <Uniq-6>", ["#,
                r#"Node("<Uniq-3> -> a", [Leaf("a", "a")]), "#,
                r#"Node("<Uniq-6> -> 0", [Leaf("0", "0")])])"#)
    ]);

    assert!(parser(["a", "b"].iter()).is_err());
    assert!(parser(["0", "1"].iter()).is_err());
}

#[test]
fn mixed() {
    let g = r#"
        row := "a" [ "b" ] ("0" | "1") [ "c" ];
    "#;
    let parser = ParserBuilder::default().treeficator(&g, "row");
    let trees = parser(["a", "0"].iter()).unwrap();
    check_trees(&trees, vec![
        concat!(
            r#"Node("row -> a <Uniq-3> <Uniq-6> <Uniq-8>", ["#,
                r#"Leaf("a", "a"), "#,
                r#"Node("<Uniq-3> -> ", []), "#,
                r#"Node("<Uniq-6> -> 0", [Leaf("0", "0")]), "#,
                r#"Node("<Uniq-8> -> ", [])])"#)
    ]);

    let trees = parser(["a", "b", "1"].iter()).unwrap();
    check_trees(&trees, vec![
        concat!(
            r#"Node("row -> a <Uniq-3> <Uniq-6> <Uniq-8>", ["#,
                r#"Leaf("a", "a"), "#,
                r#"Node("<Uniq-3> -> b", [Leaf("b", "b")]), "#,
                r#"Node("<Uniq-6> -> 1", [Leaf("1", "1")]), "#,
                r#"Node("<Uniq-8> -> ", [])])"#)
    ]);

    let trees = parser(["a", "1", "c"].iter()).unwrap();
    check_trees(&trees, vec![
        concat!(
            r#"Node("row -> a <Uniq-3> <Uniq-6> <Uniq-8>", ["#,
                r#"Leaf("a", "a"), "#,
                r#"Node("<Uniq-3> -> ", []), "#,
                r#"Node("<Uniq-6> -> 1", [Leaf("1", "1")]), "#,
                r#"Node("<Uniq-8> -> c", [Leaf("c", "c")])])"#)
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
    let parser = ParserBuilder::default()
        .plug_terminal("Number", |i| i8::from_str(i).is_ok())
        .treeficator(&g, "expr");

    let trees = parser(["3", "+", "1"].iter()).unwrap();
    check_trees(&trees, vec![
        concat!(
            r#"Node("expr -> expr + Number", ["#,
                r#"Node("expr -> Number", [Leaf("Number", "3")]), "#,
                r#"Leaf("+", "+"), "#,
                r#"Leaf("Number", "1")])"#)
    ]);
}

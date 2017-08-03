#![deny(warnings)]

extern crate lexers;
use self::lexers::DelimTokenizer;
use ebnf::{ebnf_grammar, ParserBuilder};
use util::Tree;

#[test]
fn build_ebnf_grammar() {
    ebnf_grammar();
}

#[test]
fn minimal_parser() {
    let g = r#" Number := "0" ; "#;
    let p = ParserBuilder::new().into_parser("Number", &g).unwrap();
    let mut tok = DelimTokenizer::from_str("0", " ", true);
    let state = p.parse(&mut tok).unwrap();
    let trees = Tree::builder(p.g.clone()).eval_all(&state);
    println!("{:?}", trees);
    assert_eq!(format!("{:?}", trees.unwrap()),
               r#"[Node("Number -> 0", [Leaf("0", "0")])]"#);
}

#[test]
fn arith_parser() {
    let g = r#"
        expr := Number
              | expr "+" Number ;

        Number := "0" | "1" | "2" | "3" ;
    "#;
    let p = ParserBuilder::new().into_parser("expr", &g).unwrap();
    let mut tok = DelimTokenizer::from_str("3 + 2 + 1", " ", true);
    let state = p.parse(&mut tok).unwrap();
    let trees = Tree::builder(p.g.clone()).eval_all(&state);
    println!("{:?}", trees);
    assert_eq!(format!("{:?}", trees.unwrap()),
               r#"[Node("expr -> expr + Number", [Node("expr -> expr + Number", [Node("expr -> Number", [Node("Number -> 3", [Leaf("3", "3")])]), Leaf("+", "+"), Node("Number -> 2", [Leaf("2", "2")])]), Leaf("+", "+"), Node("Number -> 1", [Leaf("1", "1")])])]"#);
}

#[test]
fn repetition() {
    let g = r#"
        arg := b { "," b } ;
        b := "0" | "1" ;
    "#;
    let p = ParserBuilder::new().into_parser("arg", &g).unwrap();
    let mut tok = DelimTokenizer::from_str("1 , 0 , 1", " ", true);
    let state = p.parse(&mut tok).unwrap();
    let trees = Tree::builder(p.g.clone()).eval_all(&state);
    assert_eq!(format!("{:?}", trees.unwrap()),
               r#"[Node("arg -> b <Uniq-3>", [Node("b -> 1", [Leaf("1", "1")]), Node("<Uniq-3> -> , b <Uniq-3>", [Leaf(",", ","), Node("b -> 0", [Leaf("0", "0")]), Node("<Uniq-3> -> , b <Uniq-3>", [Leaf(",", ","), Node("b -> 1", [Leaf("1", "1")]), Node("<Uniq-3> -> ", [])])])])]"#);
}

#[test]
fn option() {
    let g = r#"
        complex := d [ "i" ];
        d := "0" | "1" | "2";
    "#;
    let p = ParserBuilder::new().into_parser("complex", &g).unwrap();
    let mut tok = DelimTokenizer::from_str("1", " ", true);
    let state = p.parse(&mut tok).unwrap();
    let trees = Tree::builder(p.g.clone()).eval_all(&state);
    assert_eq!(format!("{:?}", trees.unwrap()),
               r#"[Node("complex -> d <Uniq-3>", [Node("d -> 1", [Leaf("1", "1")]), Node("<Uniq-3> -> ", [])])]"#);
    let mut tok = DelimTokenizer::from_str("2 i", " ", true);
    assert!(p.parse(&mut tok).is_ok());
}

#[test]
fn grouping() {
    let g = r#"
        row := ("a" | "b") ("0" | "1") ;
    "#;
    let p = ParserBuilder::new().into_parser("row", &g).unwrap();
    let mut tok = DelimTokenizer::from_str("b 1", " ", true);
    let state = p.parse(&mut tok).unwrap();
    let trees = Tree::builder(p.g.clone()).eval_all(&state);
    assert_eq!(format!("{:?}", trees.unwrap()),
               r#"[Node("row -> <Uniq-3> <Uniq-6>", [Node("<Uniq-3> -> b", [Leaf("b", "b")]), Node("<Uniq-6> -> 1", [Leaf("1", "1")])])]"#);
    let mut tok = DelimTokenizer::from_str("a 0", " ", true);
    assert!(p.parse(&mut tok).is_ok());
}

#[test]
fn plug_terminal() {
    use std::str::FromStr;
    let g = r#"
        expr := Number
              | expr "+" Number ;
    "#;
    let p = ParserBuilder::new()
        .plug_terminal("Number", |i| i8::from_str(i).is_ok())
        .into_parser("expr", &g).unwrap();
    let mut tok = DelimTokenizer::from_str("3 + 1", " ", true);
    let state = p.parse(&mut tok).unwrap();
    let trees = Tree::builder(p.g.clone()).eval_all(&state);
    assert_eq!(format!("{:?}", trees.unwrap()),
               r#"[Node("expr -> expr + Number", [Node("expr -> Number", [Leaf("Number", "3")]), Leaf("+", "+"), Leaf("Number", "1")])]"#);
}

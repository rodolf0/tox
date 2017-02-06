use regex::Regex;
use earlgrey::{Grammar, GrammarBuilder, Subtree, EarleyParser, all_trees};
use lexer::EbnfTokenizer;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

// https://en.wikipedia.org/wiki/Extended_Backus%E2%80%93Naur_form
fn ebnf_grammar() -> Grammar {

    // TODO: get rid of regex dependency
    let id_re = Regex::new(r"^[A-Za-z_]+[A-Za-z0-9_]*$").unwrap();
    let gb = GrammarBuilder::new();

    gb.symbol("<Grammar>")
      .symbol(("<Id>", move |s: &str| id_re.is_match(s)))
      .symbol(("<Chars>", move |s: &str| s.chars().all(|c| !c.is_control())))
      .symbol((":=", |s: &str| s == ":="))
      .symbol((";", |s: &str| s == ";"))
      .symbol(("[", |s: &str| s == "["))
      .symbol(("]", |s: &str| s == "]"))
      .symbol(("{", |s: &str| s == "{"))
      .symbol(("}", |s: &str| s == "}"))
      .symbol(("(", |s: &str| s == "("))
      .symbol((")", |s: &str| s == ")"))
      .symbol(("|", |s: &str| s == "|"))
      .symbol(("'", |s: &str| s == "'"))
      .symbol(("\"", |s: &str| s == "\""))
      .symbol("<RuleList>")
      .symbol("<Rule>")
      .symbol("<Rhs>")
      .symbol("<Rhs1>")
      .symbol("<Rhs2>")

      .rule("<Grammar>", &["<RuleList>"])

      .rule("<RuleList>", &["<RuleList>", "<Rule>"])
      .rule("<RuleList>", &["<Rule>"])

      .rule("<Rule>", &["<Id>", ":=", "<Rhs>", ";"])

      .rule("<Rhs>", &["<Rhs>", "|", "<Rhs1>"])
      .rule("<Rhs>", &["<Rhs1>"])

      .rule("<Rhs1>", &["<Rhs1>", "<Rhs2>"])
      .rule("<Rhs1>", &["<Rhs2>"])

      .rule("<Rhs2>", &["<Id>"])
      .rule("<Rhs2>", &["'", "<Chars>", "'"])
      .rule("<Rhs2>", &["\"", "<Chars>", "\""])
      .rule("<Rhs2>", &["[", "<Rhs>", "]"])
      .rule("<Rhs2>", &["{", "<Rhs>", "}"])
      .rule("<Rhs2>", &["(", "<Rhs>", ")"])

      .into_grammar("<Grammar>")
}

macro_rules! xtract {
    ($p:path, $e:expr) => (match $e {
        &$p(ref x, ref y) => (x, y),
        _ => panic!("Bad xtract match={:?}", $e)
    })
}

fn parse_rhs(mut gb: GrammarBuilder, tree: &Subtree, lhs: &str) -> (GrammarBuilder, Vec<String>) {
    let (spec, subn) = xtract!(Subtree::Node, tree);
    println!("** spec: {:?}", spec);
    match spec.as_ref() {
        "<Rhs> -> <Rhs> | <Rhs1>" => {
            let (mut gb, rhs) = parse_rhs(gb, &subn[0], lhs);
            println!("Adding rule {:?} -> {:?}", lhs, rhs);
            gb = gb.rule(lhs, rhs.as_slice());
            parse_rhs(gb, &subn[2], lhs)
        },

        "<Rhs1> -> <Rhs1> <Rhs2>" => {
            let (gb, mut rhs1) = parse_rhs(gb, &subn[0], lhs);
            let (gb, mut rhs2) = parse_rhs(gb, &subn[1], lhs);
            rhs1.append(&mut rhs2);
            (gb, rhs1)
        },

        "<Rhs1> -> <Rhs2>" | "<Rhs> -> <Rhs1>" => parse_rhs(gb, &subn[0], lhs),

        "<Rhs2> -> <Id>" => {
            let (_, id) = xtract!(Subtree::Leaf, &subn[0]);
            println!("Adding symbol {:?}", id);
            gb = gb.symbol(id.as_ref());
            (gb, vec!(id.to_string()))
        },

        "<Rhs2> -> ' <Chars> '" | "<Rhs2> -> \" <Chars> \"" => {
            let (_, term) = xtract!(Subtree::Leaf, &subn[1]);
            let x = term.to_string();
            println!("Adding symbol {:?}", term);
            gb = gb.symbol((term.as_ref(), move |s: &str| s == x));
            (gb, vec!(term.to_string()))
        },

        "<Rhs2> -> { <Rhs> }" => {
            //  rx -> rhs rx | <e>
            //  rhs2 -> rx
            let (gb, mut rhs) = parse_rhs(gb, &subn[1], lhs);
            let mut h = DefaultHasher::new();
            rhs.hash(&mut h);
            let repsym = format!("<Rx-{}>", h.finish());
            rhs.push(repsym.clone());
            println!("Adding symbol {:?}", repsym);
            println!("Adding rule {:?} -> {:?}", repsym, rhs);
            println!("Adding rule {:?} -> []", repsym);
            let gb =
            gb.symbol(repsym.as_ref())
              .rule(repsym.clone(), rhs.as_slice())
              .rule::<_, String>(repsym.as_ref(), &[]);
            (gb, vec!(repsym))
        },

        "<Rhs2> -> [ <Rhs> ]" => {
            //  rx -> rhs | <e>
            //  rhs2 -> rx
            let (gb, rhs) = parse_rhs(gb, &subn[1], lhs);
            let mut h = DefaultHasher::new();
            rhs.hash(&mut h);
            let repsym = format!("<Rx-{}>", h.finish());
            println!("Adding symbol {:?}", repsym);
            println!("Adding rule {:?} -> {:?}", repsym, rhs);
            println!("Adding rule {:?} -> []", repsym);
            let gb =
            gb.symbol(repsym.as_ref())
              .rule(repsym.clone(), rhs.as_slice())
              .rule::<_, String>(repsym.as_ref(), &[]);
            (gb, vec!(repsym))
        },

        missing => unreachable!("EBNF: missed a rule (2): {}", missing)
    }
}

fn parse_rules(mut gb: GrammarBuilder, tree: &Subtree) -> GrammarBuilder {
    let (spec, subn) = xtract!(Subtree::Node, tree);
    match spec.as_ref() {
        "<RuleList> -> <Rule>" => parse_rules(gb, &subn[0]),
        "<RuleList> -> <RuleList> <Rule>" => {
            gb = parse_rules(gb, &subn[0]);
            parse_rules(gb, &subn[1])
        },
        "<Rule> -> <Id> := <Rhs> ;" => {
            let (_, lhs) = xtract!(Subtree::Leaf, &subn[0]);
            let (gb, rhs) = parse_rhs(gb, &subn[2], lhs.as_ref());
            println!("Adding rule {:?} -> {:?}", lhs, rhs);
            gb.rule(lhs.to_string(), rhs.as_slice())
        },
        missing => unreachable!("EBNF: missed a rule: {}", missing)
    }
}

fn build_grammar(start: &str, tree: &Subtree) -> Grammar {
    let (spec, subn) = xtract!(Subtree::Node, tree);
    let gb = GrammarBuilder::new();
    match spec.as_ref() {
        "<Grammar> -> <RuleList>" => parse_rules(gb, &subn[0]),
        _ => panic!("EBNF: What !!")
    }.symbol(start)
    .into_grammar(start)
}

pub fn build_parser(grammar: &str, start: &str) -> EarleyParser {
    let ebnf_parser = EarleyParser::new(ebnf_grammar());
    let mut tokenizer = EbnfTokenizer::from_str(grammar);
    let trees = match ebnf_parser.parse(&mut tokenizer) {
        Err(e) => panic!("Bad grammar: {:?}", e),
        Ok(state) => {
            let ts = all_trees(ebnf_parser.g.start(), &state);
            if ts.len() != 1 {
                for t in &ts {
                    t.print();
                }
                panic!("EBNF is ambiguous?");
            }
            ts
        }
    };
    EarleyParser::new(build_grammar(start, &trees[0]))
}


#[cfg(test)]
mod test {
    use super::ebnf_grammar;
    use super::build_parser;
    use lexers::DelimTokenizer;
    use earlgrey::all_trees;

    #[test]
    fn build_ebnf_grammar() {
        ebnf_grammar();
    }

    #[test]
    fn test_minimal_parser() {
        let g = r#" Number := "0" ; "#;
        let p = build_parser(&g, "Number");
        let mut tok = DelimTokenizer::from_str("0", " ", true);
        let state = p.parse(&mut tok).unwrap();
        let trees = all_trees(p.g.start(), &state);
        assert_eq!(format!("{:?}", trees),
                   r#"[Node("Number -> 0", [Leaf("0", "0")])]"#);
    }

    #[test]
    fn test_arith_parser() {
        let g = r#"
            expr := Number
                  | expr "+" Number ;

            Number := "0" | "1" | "2" | "3" ;
        "#;
        let p = build_parser(&g, "expr");
        let mut tok = DelimTokenizer::from_str("3 + 2 + 1", " ", true);
        let state = p.parse(&mut tok).unwrap();
        let trees = all_trees(p.g.start(), &state);
        assert_eq!(format!("{:?}", trees),
                   r#"[Node("expr -> expr + Number", [Node("expr -> expr + Number", [Node("expr -> Number", [Node("Number -> 3", [Leaf("3", "3")])]), Leaf("+", "+"), Node("Number -> 2", [Leaf("2", "2")])]), Leaf("+", "+"), Node("Number -> 1", [Leaf("1", "1")])])]"#);
    }

    #[test]
    fn test_repetition() {
        let g = r#"
            arg := b { "," b } ;
            b := "0" | "1" ;
        "#;
        let p = build_parser(&g, "arg");
        let mut tok = DelimTokenizer::from_str("1 , 0 , 1", " ", true);
        let state = p.parse(&mut tok).unwrap();
        let trees = all_trees(p.g.start(), &state);
        // NOTE: hash might make this test fail
        assert_eq!(format!("{:?}", trees),
                   r#"[Node("arg -> b <Rx-4288732297984548594>", [Node("b -> 1", [Leaf("1", "1")]), Node("<Rx-4288732297984548594> -> , b <Rx-4288732297984548594>", [Leaf(",", ","), Node("b -> 0", [Leaf("0", "0")]), Node("<Rx-4288732297984548594> -> , b <Rx-4288732297984548594>", [Leaf(",", ","), Node("b -> 1", [Leaf("1", "1")]), Node("<Rx-4288732297984548594> -> ", [])])])])]"#);
    }

    #[test]
    fn test_option() {
        let g = r#"
            complex := d [ "i" ];
            d := "0" | "1" | "2";
        "#;
        let p = build_parser(&g, "complex");
        let mut tok = DelimTokenizer::from_str("1", " ", true);
        let state = p.parse(&mut tok).unwrap();
        let trees = all_trees(p.g.start(), &state);
        // NOTE: hash might make this test fail
        assert_eq!(format!("{:?}", trees),
                   r#"[Node("complex -> d <Rx-12307430620962403152>", [Node("d -> 1", [Leaf("1", "1")]), Node("<Rx-12307430620962403152> -> ", [])])]"#);
        let mut tok = DelimTokenizer::from_str("2 i", " ", true);
        assert!(p.parse(&mut tok).is_ok());
    }
}

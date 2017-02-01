use regex::Regex;
use earlgrey::{Grammar, GrammarBuilder, Subtree, EarleyParser, all_trees};
use lexer::EbnfTokenizer;

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
      .symbol((",", |s: &str| s == ","))
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

      .rule("<Rhs1>", &["<Rhs1>", ",", "<Rhs2>"])
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

fn parse_rhs(mut gb: GrammarBuilder, tree: &Subtree) -> (GrammarBuilder, Vec<String>) {
    let (spec, subn) = xtract!(Subtree::Node, tree);
    match spec.as_ref() {
        "<Rhs> -> <Rhs> | <Rhs1>" => {
            let (gb, rhs) = parse_rhs(gb, &subn[0]);
            //println!("Adding rule {:?} -> {:?}", lexeme, rhs);
            //gb = gb.rule(lexeme.clone(), rhs.as_slice());
            parse_rhs(gb, &subn[2])
        },
        "<Rhs> -> <Rhs1>" => {
            parse_rhs(gb, &subn[0])
        },

        "<Rhs1> -> <Rhs1> , <Rhs2>" => {
            let (gb, _) = parse_rhs(gb, &subn[0]);
            parse_rhs(gb, &subn[2])
        },
        "<Rhs1> -> <Rhs2>" => {
            parse_rhs(gb, &subn[0])
        },

        "<Rhs2> -> <Id>" => {
            let (_, lexeme) = xtract!(Subtree::Leaf, &subn[0]);
            (gb.symbol(lexeme.as_ref()), vec!(lexeme.clone()))
        },

        "<Rhs2> -> ' <Chars> '" |
        "<Rhs2> -> \" <Chars> \"" => {
            let (_, lexeme) = xtract!(Subtree::Leaf, &subn[1]);
            let x = lexeme.to_string();
            println!("Adding symbol {:?}", lexeme);
            gb = gb.symbol((lexeme.as_ref(), move |s: &str| s == x));
            (gb, vec!(lexeme.clone()))
        }
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
            let (_, lexeme) = xtract!(Subtree::Leaf, &subn[0]);
            let (gb, rhs) = parse_rhs(gb, &subn[2]);
            println!("Adding rule {:?} -> {:?}", lexeme, rhs);
            gb.rule(lexeme.clone(), rhs.as_slice())
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
        let input = "0";
        let mut tok = DelimTokenizer::from_str(input, " ", true);
        let state = p.parse(&mut tok).unwrap();
        let trees = all_trees(p.g.start(), &state);
        assert_eq!(format!("{:?}", trees),
                   r#"[Node("Number -> 0", [Leaf("0", "0")])]"#);
    }

    #[test]
    fn test_arith_parser() {
        let g = r#"
            expr := Number
                  | expr, "+", Number ;

            Number := "0" | "1" | "2" | "3" ;
        "#;
        let p = build_parser(&g, "expr");
        let input = "3 + 3";
        let mut tok = DelimTokenizer::from_str(input, " ", true);
        let state = p.parse(&mut tok).unwrap();
        let trees = all_trees(p.g.start(), &state);
        println!("{:?}", trees);
        //assert_eq!(format!("{:?}", trees),
                   //r#"[Node("Number -> 0", [Leaf("0", "0")])]"#);
    }
}

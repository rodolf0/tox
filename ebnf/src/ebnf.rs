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
      .symbol("<Terminal>")


      .rule("<Grammar>", &["<RuleList>"])

      .rule("<RuleList>", &["<RuleList>", "<Rule>"])
      .rule("<RuleList>", &["<Rule>"])

      .rule("<Rule>", &["<Id>", ":=", "<Rhs>", ";"])

      .rule("<Rhs>", &["<Id>"])
      .rule("<Rhs>", &["<Terminal>"])
      .rule("<Rhs>", &["[", "<Rhs>", "]"])
      .rule("<Rhs>", &["{", "<Rhs>", "}"])
      .rule("<Rhs>", &["(", "<Rhs>", ")"])
      .rule("<Rhs>", &["<Rhs>", "|", "<Rhs>"])
      .rule("<Rhs>", &["<Rhs>", ",", "<Rhs>"])

      .rule("<Terminal>", &["'", "<Chars>", "'"])
      .rule("<Terminal>", &["\"", "<Chars>", "\""])

      .into_grammar("<Grammar>")
}

macro_rules! xtract {
    ($p:path, $e:expr) => (match $e {
        &$p(ref x, ref y) => (x, y),
        _ => panic!("Bad xtract match={:?}", $e)
    })
}

fn parse_rhs(gb: GrammarBuilder, tree: &Subtree) -> (GrammarBuilder, Vec<String>) {
    let (spec, subn) = xtract!(Subtree::Node, tree);
    match spec.as_ref() {
        //"<Rhs> -> <Id>" => {
            //let (_, lexeme) = xtract!(Subtree::Leaf, &subn[0]);
            //(gb.symbol(lexeme.as_ref()), vec!(lexeme.clone()))
        //},
        "<Rhs> -> <Terminal>" => parse_rhs(gb, &subn[0]),
        "<Terminal> -> ' <Chars> '" |
        "<Terminal> -> \" <Chars> \"" => {
            let (_, lexeme) = xtract!(Subtree::Leaf, &subn[1]);
            (gb.symbol(lexeme.as_ref()), vec!(lexeme.clone()))
        }
        _ => unreachable!("EBNF: missed a rule (2)!")
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
            gb.rule(lexeme.clone(), rhs.as_slice())
        },
        _ => unreachable!("EBNF: missed a rule!")
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
        p.parse(&mut tok).unwrap();
        //assert!(p.parse(&mut tok).is_ok());
    }
}

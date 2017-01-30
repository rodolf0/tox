use regex::Regex;
use lexers::{Scanner, Nexter, scan_identifier};
use earlgrey::{Grammar, GrammarBuilder, Subtree, EarleyParser};
use earlgrey;

// https://en.wikipedia.org/wiki/Extended_Backus%E2%80%93Naur_form
fn ebnf_grammar() -> Grammar {

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

struct EbnfTokenizer(Scanner<char>, Vec<String>);

impl EbnfTokenizer {
    fn from_str(src: &str) -> Scanner<String> {
        Scanner::new(Box::new(EbnfTokenizer(Scanner::from_str(src), vec!())))
    }
}

impl Nexter<String> for EbnfTokenizer {
    fn get_item(&mut self) -> Option<String> {
        // used for accumulating string parts
        if !self.1.is_empty() {
            return self.1.pop();
        }
        let mut s = &mut self.0;
        s.ignore_ws();
        if s.accept_any_char("[]{}()|,;").is_some() {
            return Some(s.extract_string());
        }
        let backtrack = s.pos();
        if s.accept_any_char(":").is_some() {
            if s.accept_any_char("=").is_some() {
                return Some(s.extract_string());
            }
            s.set_pos(backtrack);
        }
        let backtrack = s.pos();
        if let Some(q) = s.accept_any_char("\"'") {
            while let Some(n) = s.next() {
                if n == q {
                    // store closing quote
                    self.1.push(n.to_string());
                    // store string content
                    let v = s.extract_string();
                    self.1.push(v[1..v.len()-1].to_string());
                    // return opening quote
                    return Some(q.to_string());
                }
            }
            s.set_pos(backtrack);
        }
        if let Some(id) = scan_identifier(&mut s) {
            return Some(id);
        }
        return None;
    }
}

macro_rules! xtract {
    ($p:path, $e:expr) => (match $e {
        &$p(ref x, ref y) => (x, y),
        _ => panic!("Bad xtract match={:?}", $e)
    })
}

fn parse_rhs(tree: &Subtree) -> Vec<String> {
    let (spec, subn) = xtract!(Subtree::Node, tree);
    match spec.as_ref() {
        "<Rhs> -> <Terminal>" => parse_rhs(&subn[0]),
        "<Terminal> -> ' <Chars> '" |
        "<Terminal> -> \" <Chars> \"" => {
            let (_, lexeme) = xtract!(Subtree::Leaf, &subn[1]);
            vec!(lexeme.clone())
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
            let rhs = parse_rhs(&subn[2]);
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
    }.into_grammar(start)
}

pub fn build_parser(grammar: &str, start: &str) -> EarleyParser {
    let ebnf_parser = EarleyParser::new(ebnf_grammar());
    let mut tokenizer = EbnfTokenizer::from_str(grammar);
    let trees = match ebnf_parser.parse(&mut tokenizer) {
        Err(e) => panic!("Bad grammar: {:?}", e),
        Ok(state) => {
            let ts = earlgrey::all_trees(ebnf_parser.g.start(), &state);
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

    #[test]
    fn build_ebnf_grammar() {
        ebnf_grammar();
    }

    #[test]
    fn test_build_parser() {
        let g = r#"
            Number := "0" ;
        "#;

        build_parser(&g, "Number");
    }
}

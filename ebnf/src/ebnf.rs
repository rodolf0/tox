use regex::Regex;
use lexers::Scanner;
use earlgrey;

// https://en.wikipedia.org/wiki/Extended_Backus%E2%80%93Naur_form
fn ebnf_grammar() -> earlgrey::Grammar {

    let id_re = Regex::new(r"^[A-Za-z_]+[A-Za-z0-9_]*$").unwrap();
    let chars_re = Regex::new(r#"^[A-Za-z0-9_\[\]{}()<>'=|.,;"]*$"#).unwrap();
    let gb = earlgrey::GrammarBuilder::new();

    gb.symbol("<Grammar>")
      .symbol(("<Id>", move |s: &str| id_re.is_match(s)))
      .symbol(("<Chars>", move |s: &str| chars_re.is_match(s)))
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
      .rule("<Rhs>", &["[", "<Terminal>", "]"])
      .rule("<Rhs>", &["{", "<Terminal>", "}"])
      .rule("<Rhs>", &["(", "<Terminal>", ")"])
      .rule("<Rhs>", &["<Rhs>", "|", "<Rhs>"])
      .rule("<Rhs>", &["<Rhs>", ",", "<Rhs>"])

      .rule("<Terminal>", &["'", "<Chars>", "'"])
      .rule("<Terminal>", &["\"", "<Chars>", "\""])

      .into_grammar("<Grammar>")
}

struct EbnfTokenizer {}

impl EbnfTokenizer {
    fn from_str(src: &str) -> Scanner<String> {
    }
}

fn build_grammar(tree: earlgrey::Subtree) -> earlgrey::Grammar {
    let g = earlgrey::GrammarBuilder::new();
}

pub fn ParserFromEbnf(grammar: &str) -> earlgrey::EarleyParser {
    let ebnf_parser = earlgrey::EarleyParser::new(ebnf_grammar());
    let mut tokenizer = EbnfTokenizer::from_str(grammar);
    let tree = match ebnf_parser.parse(&mut tokenizer) {
        Err(e) => panic!("Bad grammar: {:?}", e),
        Ok(state) => {
            let ts = earlgrey::all_trees(ebnf_parser.g.start(), &state);
            if ts.len() != 1 {
                panic!("EBNF is ambiguous?");
            }
            ts[0]
        }
    };
    earlgrey::EarleyParser::new(build_grammar(tree))
}


#[cfg(test)]
mod test {
    use super::ebnf_grammar;

    #[test]
    fn build_ebnf_grammar() {
        ebnf_grammar();
    }
}

use types::{Grammar, GrammarBuilder};
use trees::{Subtree, all_trees};
use lexers::EbnfTokenizer;
use parser::EarleyParser;

const DEBUG_EBNF: bool = false;

// https://en.wikipedia.org/wiki/Extended_Backus%E2%80%93Naur_form
pub fn ebnf_grammar() -> Grammar {
    GrammarBuilder::new()
      .symbol("<Grammar>")
      .symbol(("<Id>", move |s: &str|  // in sync w lexers::scan_identifier
               s.chars().all(|c| c.is_alphanumeric() || c == '_')))
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

fn parse_rhs(gb: GrammarBuilder, tree: &Subtree, lhs: &str)
        -> (GrammarBuilder, Vec<String>) {
    let (spec, subn) = xtract!(Subtree::Node, tree);
    if DEBUG_EBNF { println!("** spec: {:?}", spec); }
    match spec.as_ref() {
        "<Rhs> -> <Rhs> | <Rhs1>" => {
            let (gb, rhs) = parse_rhs(gb, &subn[0], lhs);
            if DEBUG_EBNF { println!("Adding rule {:?} -> {:?}", lhs, rhs); }
            let gb = gb.rule(lhs, rhs.as_slice());
            // rule for 2nd branch is added in parent
            parse_rhs(gb, &subn[2], lhs)
        },

        "<Rhs1> -> <Rhs1> <Rhs2>" => {
            let (gb, mut rhs1) = parse_rhs(gb, &subn[0], lhs);
            let (gb, mut rhs2) = parse_rhs(gb, &subn[1], lhs);
            rhs1.append(&mut rhs2);
            (gb, rhs1)
        },

        "<Rhs1> -> <Rhs2>" | "<Rhs> -> <Rhs1>" => parse_rhs(gb, &subn[0], lhs),

        "<Rhs2> -> ( <Rhs> )" => {
            let auxsym = gb.unique_symbol_name();
            if DEBUG_EBNF { println!("Adding symbol {:?}", auxsym); }
            let gb = gb.symbol(auxsym.as_ref());
            let (gb, rhs) = parse_rhs(gb, &subn[1], auxsym.as_ref());
            if DEBUG_EBNF { println!("Adding rule {:?} -> {:?}", auxsym, rhs); }
            let gb = gb.rule(auxsym.clone(), rhs.as_slice());
            (gb, vec!(auxsym))
        },

        "<Rhs2> -> <Id>" => {
            let (_, id) = xtract!(Subtree::Leaf, &subn[0]);
            if DEBUG_EBNF { println!("Adding symbol {:?}", id); }
            let gb = gb.symbol_relaxed(id.as_ref());
            (gb, vec!(id.to_string()))
        },

        "<Rhs2> -> ' <Chars> '" | "<Rhs2> -> \" <Chars> \"" => {
            let (_, term) = xtract!(Subtree::Leaf, &subn[1]);
            let x = term.to_string();
            if DEBUG_EBNF { println!("Adding symbol {:?}", term); }
            let gb = gb.symbol_relaxed((term.as_ref(), move |s: &str| s == x));
            (gb, vec!(term.to_string()))
        },

        "<Rhs2> -> { <Rhs> }" => {
            // rhs -> auxsym
            // auxsym -> <e>
            // auxsym -> rhs auxsym
            let auxsym = gb.unique_symbol_name();
            if DEBUG_EBNF { println!("Adding symbol {:?}", auxsym); }
            let gb = gb.symbol(auxsym.as_ref());
            let (gb, mut rhs) = parse_rhs(gb, &subn[1], auxsym.as_ref());
            rhs.push(auxsym.clone());
            if DEBUG_EBNF {
                println!("Adding rule {:?} -> []", auxsym);
                println!("Adding rule {:?} -> {:?}", auxsym, rhs);
            }
            let gb =
            gb.rule::<_, String>(auxsym.as_ref(), &[])
              .rule(auxsym.clone(), rhs.as_slice());
            (gb, vec!(auxsym))
        },

        "<Rhs2> -> [ <Rhs> ]" => {
            // rhs -> auxsym
            // auxsym -> <e>
            // auxsym -> rhs
            let auxsym = gb.unique_symbol_name();
            if DEBUG_EBNF { println!("Adding symbol {:?}", auxsym); }
            let gb = gb.symbol(auxsym.as_ref());
            let (gb, rhs) = parse_rhs(gb, &subn[1], auxsym.as_ref());
            if DEBUG_EBNF {
                println!("Adding rule {:?} -> []", auxsym);
                println!("Adding rule {:?} -> {:?}", auxsym, rhs);
            }
            let gb = gb.rule::<_, String>(auxsym.as_ref(), &[])
                       .rule(auxsym.clone(), rhs.as_slice());
            (gb, vec!(auxsym))
        },

        missing => unreachable!("EBNF: missed a rule (2): {}", missing)
    }
}

fn parse_rules(gb: GrammarBuilder, tree: &Subtree) -> GrammarBuilder {
    let (spec, subn) = xtract!(Subtree::Node, tree);
    match spec.as_ref() {
        "<RuleList> -> <Rule>" => parse_rules(gb, &subn[0]),
        "<RuleList> -> <RuleList> <Rule>" => {
            let gb = parse_rules(gb, &subn[0]);
            parse_rules(gb, &subn[1])
        },
        "<Rule> -> <Id> := <Rhs> ;" => {
            let (_, lhs) = xtract!(Subtree::Leaf, &subn[0]);
            if DEBUG_EBNF { println!("Adding symbol {:?}", lhs); }
            let gb = gb.symbol_relaxed(lhs.to_string());
            let (gb, rhs) = parse_rhs(gb, &subn[2], lhs.as_ref());
            if DEBUG_EBNF { println!("Adding rule {:?} -> {:?}", lhs, rhs); }
            gb.rule(lhs.to_string(), rhs.as_slice())
        },
        missing => unreachable!("EBNF: missed a rule: {}", missing)
    }
}

pub struct ParserBuilder(GrammarBuilder, Subtree);

impl ParserBuilder {
    pub fn new(grammar: &str) -> ParserBuilder {
        let ebnf_parser = EarleyParser::new(ebnf_grammar());
        let mut tokenizer = EbnfTokenizer::from_str(grammar);
        let mut trees = match ebnf_parser.parse(&mut tokenizer) {
            Err(e) => panic!("ParserBuilder error: {:?}", e),
            Ok(state) => all_trees(ebnf_parser.g.start(), &state),
        };
        if trees.len() != 1 {
            for t in &trees { t.print(); }
            panic!("EBNF is ambiguous?");
        }
        ParserBuilder(GrammarBuilder::new(), trees.swap_remove(0))
    }

    pub fn plug_terminal<N, F>(self, name: N, pred: F) -> ParserBuilder
            where N: Into<String>, F: 'static + Fn(&str)->bool {
        ParserBuilder(self.0.symbol((name.into(), pred)), self.1)
    }

    pub fn into_parser(self, start: &str) -> EarleyParser {
        let (spec, subn) = xtract!(Subtree::Node, &self.1);
        let grammar = match spec.as_ref() {
            "<Grammar> -> <RuleList>" => parse_rules(self.0, &subn[0]),
            _ => panic!("EBNF: What !!")
        }.symbol_relaxed(start)
         .into_grammar(start);
        EarleyParser::new(grammar)
    }
}

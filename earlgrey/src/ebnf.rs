#![deny(warnings)]

use super::ebnf_tokenizer::EbnfTokenizer;
use crate::earley::{EarleyForest, EarleyParser, Grammar, GrammarBuilder};
use std::cell::RefCell;

macro_rules! debug {
    ($($args:tt)*) => (if cfg!(feature="debug") { eprintln!($($args)*); })
}

#[derive(Clone, Debug)]
enum G {
    VariantList(Vec<Vec<String>>),
    Variant(Vec<String>),
    Atom(String),
    Nop,
}

// use to destructure G enum into a specific alternative
macro_rules! pull {
    ($p:path, $e:expr) => {
        match $e {
            $p(value) => value,
            n => panic!("Bad pull match={:?}", n),
        }
    };
}

// https://en.wikipedia.org/wiki/Extended_Backus%E2%80%93Naur_form
fn ebnf_grammar() -> Grammar {
    GrammarBuilder::default()
        .terminal("<Id>", move |s| {
            s.chars().enumerate().all(|(i, c)| {
                i == 0 && c.is_alphabetic() || i > 0 && (c.is_alphanumeric() || c == '_')
            })
        })
        .terminal("<Chars>", move |s| s.chars().all(|c| !c.is_control()))
        .terminal("@<Tag>", move |s| {
            s.chars().enumerate().all(|(i, c)| {
                i == 0 && c == '@'
                    || i == 1 && c.is_alphabetic()
                    || i > 1 && (c.is_alphanumeric() || c == '_')
            })
        })
        .terminal(":=", |s| s == ":=")
        .terminal(";", |s| s == ";")
        .terminal("[", |s| s == "[")
        .terminal("]", |s| s == "]")
        .terminal("{", |s| s == "{")
        .terminal("}", |s| s == "}")
        .terminal("(", |s| s == "(")
        .terminal(")", |s| s == ")")
        .terminal("|", |s| s == "|")
        .terminal("'", |s| s == "'")
        .terminal("\"", |s| s == "\"")
        .nonterm("<RuleList>")
        .nonterm("<Rule>")
        .nonterm("<VariantList>")
        .nonterm("<Variant>")
        .nonterm("<Atom>")
        .rule("<RuleList>", &["<RuleList>", "<Rule>"])
        .rule("<RuleList>", &["<Rule>"])
        .rule("<Rule>", &["<Id>", ":=", "<VariantList>", ";"])
        .rule("<VariantList>", &["<VariantList>", "|", "<Variant>"])
        .rule("<VariantList>", &["<Variant>"])
        .rule("<Variant>", &["<Variant>", "<Atom>"])
        .rule("<Variant>", &["<Atom>"])
        .rule("<Atom>", &["<Id>"])
        .rule("<Atom>", &["'", "<Chars>", "'"])
        .rule("<Atom>", &["\"", "<Chars>", "\""])
        .rule("<Atom>", &["[", "<VariantList>", "]"])
        .rule("<Atom>", &["{", "<VariantList>", "}"])
        .rule("<Atom>", &["(", "<VariantList>", ")"])
        .rule("<Atom>", &["[", "<VariantList>", "]", "@<Tag>"])
        .rule("<Atom>", &["{", "<VariantList>", "}", "@<Tag>"])
        .rule("<Atom>", &["(", "<VariantList>", ")", "@<Tag>"])
        .into_grammar("<RuleList>")
        .expect("Bad EBNF Grammar")
}

fn ebnf_terminal_parser(
    user_grammar_builder: &RefCell<GrammarBuilder>,
) -> impl Fn(&str, &str) -> G + '_ {
    move |symbol, token| {
        match symbol {
            "<Id>" => {
                debug!("Adding non-term {:?}", token);
                user_grammar_builder.borrow_mut().nonterm_try(token);
            }
            "@<Tag>" => {
                debug!("Adding non-term {:?}", token);
                user_grammar_builder.borrow_mut().nonterm_try(token);
            }
            "<Chars>" => {
                debug!("Adding terminal {:?}", token);
                let tok = token.to_string();
                user_grammar_builder
                    .borrow_mut()
                    .terminal_try(token, move |s| s == tok);
            }
            _ => (),
        }
        G::Atom(token.to_string())
    }
}

fn ebnf_rule_action<'a>(ev: &mut EarleyForest<'a, G>, gb: &'a RefCell<GrammarBuilder>) {
    ev.action("<Rule> -> <Id> := <VariantList> ;", move |mut n| {
        let id = pull!(G::Atom, n.remove(0));
        let body = pull!(G::VariantList, n.remove(1));
        let mut t_gb = gb.borrow_mut();
        for rule in body {
            debug!("Adding rule {:?} -> {:?}", id, rule);
            t_gb.rule_try(&id, &rule.iter().map(|s| s.as_str()).collect::<Vec<&str>>());
        }
        G::Nop
    });
}

fn ebnf_variantlist_action(ev: &mut EarleyForest<'_, G>) {
    ev.action("<VariantList> -> <VariantList> | <Variant>", |mut n| {
        let mut body = pull!(G::VariantList, n.remove(0));
        body.push(pull!(G::Variant, n.remove(1)));
        G::VariantList(body)
    });
    ev.action("<VariantList> -> <Variant>", |mut n| {
        let part = pull!(G::Variant, n.remove(0));
        G::VariantList(vec![part])
    });
}

fn ebnf_variant_action(ev: &mut EarleyForest<'_, G>) {
    ev.action("<Variant> -> <Variant> <Atom>", |mut n| {
        let mut part = pull!(G::Variant, n.remove(0));
        part.push(pull!(G::Atom, n.remove(0)));
        G::Variant(part)
    });
    ev.action("<Variant> -> <Atom>", |mut n| {
        G::Variant(vec![pull!(G::Atom, n.remove(0))])
    });
}

fn ebnf_grouping_action<'a>(ev: &mut EarleyForest<'a, G>, gb: &'a RefCell<GrammarBuilder>) {
    ev.action("<Atom> -> ( <VariantList> )", move |mut n| {
        let aux = gb.borrow().unique_symbol_name();
        debug!("Adding non-term {:?}", aux);
        let mut t_gb = gb.borrow_mut();
        t_gb.nonterm_try(&aux);
        let body = pull!(G::VariantList, n.remove(1));
        for rule in body {
            debug!("Adding rule {:?} -> {:?}", aux, rule);
            t_gb.rule_try(
                &aux,
                &rule.iter().map(|s| s.as_str()).collect::<Vec<&str>>(),
            );
        }
        G::Atom(aux)
    });
    ev.action("<Atom> -> ( <VariantList> ) @<Tag>", move |mut n| {
        let aux = pull!(G::Atom, n.remove(3));
        debug!("Adding non-term {:?}", aux);
        let mut t_gb = gb.borrow_mut();
        t_gb.nonterm_try(&aux);
        let body = pull!(G::VariantList, n.remove(1));
        for rule in body {
            debug!("Adding rule {:?} -> {:?}", aux, rule);
            t_gb.rule_try(
                &aux,
                &rule.iter().map(|s| s.as_str()).collect::<Vec<&str>>(),
            );
        }
        G::Atom(aux)
    });
}

fn ebnf_optional_action<'a>(ev: &mut EarleyForest<'a, G>, gb: &'a RefCell<GrammarBuilder>) {
    ev.action("<Atom> -> [ <VariantList> ]", move |mut n| {
        // <Atom> -> aux ; aux -> <e> | <VariantList> ;
        let aux = gb.borrow().unique_symbol_name();
        debug!("Adding non-term {:?}", aux);
        let mut t_gb = gb.borrow_mut();
        t_gb.nonterm_try(&aux);
        let body = pull!(G::VariantList, n.remove(1));
        for rule in body {
            debug!("Adding rule {:?} -> {:?}", aux, rule);
            t_gb.rule_try(
                &aux,
                &rule.iter().map(|s| s.as_str()).collect::<Vec<&str>>(),
            );
            debug!("Adding rule {:?} -> []", aux);
            t_gb.rule_try(&aux, &[]);
        }
        G::Atom(aux)
    });
    ev.action("<Atom> -> [ <VariantList> ] @<Tag>", move |mut n| {
        let aux = pull!(G::Atom, n.remove(3));
        debug!("Adding non-term {:?}", aux);
        let mut t_gb = gb.borrow_mut();
        t_gb.nonterm_try(&aux);
        let body = pull!(G::VariantList, n.remove(1));
        for rule in body {
            debug!("Adding rule {:?} -> {:?}", aux, rule);
            t_gb.rule_try(
                &aux,
                &rule.iter().map(|s| s.as_str()).collect::<Vec<&str>>(),
            );
            debug!("Adding rule {:?} -> []", aux);
            t_gb.rule_try(&aux, &[]);
        }
        G::Atom(aux)
    });
}

fn ebnf_repeat_action<'a>(ev: &mut EarleyForest<'a, G>, gb: &'a RefCell<GrammarBuilder>) {
    ev.action("<Atom> -> { <VariantList> }", move |mut n| {
        // <Atom> -> aux ; aux -> <e> | <VariantList> aux ;
        let aux = gb.borrow().unique_symbol_name();
        debug!("Adding non-term {:?}", aux);
        let mut t_gb = gb.borrow_mut();
        t_gb.nonterm_try(&aux);
        let body = pull!(G::VariantList, n.remove(1));
        for mut rule in body {
            rule.push(aux.clone());
            debug!("Adding rule {:?} -> {:?}", aux, rule);
            t_gb.rule_try(
                &aux,
                &rule.iter().map(|s| s.as_str()).collect::<Vec<&str>>(),
            );
            debug!("Adding rule {:?} -> []", aux);
            t_gb.rule_try(&aux, &[]);
        }
        G::Atom(aux)
    });
    ev.action("<Atom> -> { <VariantList> } @<Tag>", move |mut n| {
        // <Atom> -> aux ; aux -> <e> | <VariantList> aux ;
        let aux = pull!(G::Atom, n.remove(3));
        debug!("Adding non-term {:?}", aux);
        let mut t_gb = gb.borrow_mut();
        t_gb.nonterm_try(&aux);
        let body = pull!(G::VariantList, n.remove(1));
        for mut rule in body {
            rule.push(aux.clone());
            debug!("Adding rule {:?} -> {:?}", aux, rule);
            t_gb.rule_try(
                &aux,
                &rule.iter().map(|s| s.as_str()).collect::<Vec<&str>>(),
            );
            debug!("Adding rule {:?} -> []", aux);
            t_gb.rule_try(&aux, &[]);
        }
        G::Atom(aux)
    });
}

pub struct EbnfGrammarParser {
    start: String,
    grammar: String,
    grammar_builder: GrammarBuilder,
}

impl EbnfGrammarParser {
    // Parse a user grammar into a builder where we can plug terminal matchers
    pub fn new(grammar: &str, start: &str) -> Self {
        Self {
            start: start.to_string(),
            grammar: grammar.to_string(),
            grammar_builder: GrammarBuilder::default(),
        }
    }

    // Plug-in functions that parse Terminals before we build the grammar
    pub fn plug_terminal(mut self, name: &str, pred: impl Fn(&str) -> bool + 'static) -> Self {
        debug!("Adding terminal {:?}", name);
        self.grammar_builder.terminal_try(name, pred);
        self
    }

    pub fn into_grammar(self) -> Result<Grammar, String> {
        // Need to move grammar_builder into a refcell because ebnf
        // semantic actions need mutable access to add rules and symbols.
        // These grammar-builder changes are executed while the ebnf-parser
        // is evaluating semantic actions, ie: at `eval_all` line.
        let grammar_builder = RefCell::new(self.grammar_builder);
        {
            let mut user_semanter = EarleyForest::new(ebnf_terminal_parser(&grammar_builder));
            user_semanter.action("<RuleList> -> <RuleList> <Rule>", |_| G::Nop);
            user_semanter.action("<RuleList> -> <Rule>", |_| G::Nop);
            ebnf_rule_action(&mut user_semanter, &grammar_builder);
            ebnf_variantlist_action(&mut user_semanter);
            ebnf_variant_action(&mut user_semanter);
            ebnf_grouping_action(&mut user_semanter, &grammar_builder);
            ebnf_optional_action(&mut user_semanter, &grammar_builder);
            ebnf_repeat_action(&mut user_semanter, &grammar_builder);
            user_semanter.action("<Atom> -> <Id>", |mut n| n.remove(0));
            user_semanter.action("<Atom> -> ' <Chars> '", |mut n| n.remove(1));
            user_semanter.action("<Atom> -> \" <Chars> \"", |mut n| n.remove(1));

            // Create a parser for EBNF which we'll use to parse input grammar
            let parsed_user_grammar = EarleyParser::new(ebnf_grammar())
                .parse(EbnfTokenizer::new(self.grammar.chars()))?;
            //
            if user_semanter.eval_all(&parsed_user_grammar)?.len() != 1 {
                panic!("BUG: EBNF grammar shouldn't be ambiguous!");
            }
        }
        grammar_builder.into_inner().into_grammar(&self.start)
    }
}

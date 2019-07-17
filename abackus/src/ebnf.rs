#![deny(warnings)]

use lexers::EbnfTokenizer;
use earlgrey::{
    Grammar, GrammarBuilder,
    EarleyParser, Error, EarleyForest,
};
use std::cell::RefCell;


// https://en.wikipedia.org/wiki/Extended_Backus%E2%80%93Naur_form
pub fn ebnf_grammar() -> Grammar {
    GrammarBuilder::default()
      .terminal("<Id>", move |s|
                s.chars().enumerate().all(|(i, c)|
                    i == 0 && c.is_alphabetic() ||
                    i > 0 && (c.is_alphanumeric() || c == '_')))
      .terminal("<Chars>", move |s| s.chars().all(|c| !c.is_control()))
      .terminal("@<Tag>", move |s|
                s.chars().enumerate().all(|(i, c)|
                    i == 0 && c == '@' ||
                    i == 1 && c.is_alphabetic() ||
                    i > 1 && (c.is_alphanumeric() || c == '_')))
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
      .nonterm("<Body>")
      .nonterm("<Part>")
      .nonterm("<Atom>")
      .rule("<RuleList>", &["<RuleList>", "<Rule>"])
      .rule("<RuleList>", &["<Rule>"])
      .rule("<Rule>", &["<Id>", ":=", "<Body>", ";"])
      .rule("<Body>", &["<Body>", "|", "<Part>"])
      .rule("<Body>", &["<Part>"])
      .rule("<Part>", &["<Part>", "<Atom>"])
      .rule("<Part>", &["<Atom>"])
      .rule("<Atom>", &["<Id>"])
      .rule("<Atom>", &["'", "<Chars>", "'"])
      .rule("<Atom>", &["\"", "<Chars>", "\""])
      .rule("<Atom>", &["[", "<Body>", "]"])
      .rule("<Atom>", &["{", "<Body>", "}"])
      .rule("<Atom>", &["(", "<Body>", ")"])
      .rule("<Atom>", &["[", "<Body>", "]", "@<Tag>"])
      .rule("<Atom>", &["{", "<Body>", "}", "@<Tag>"])
      .rule("<Atom>", &["(", "<Body>", ")", "@<Tag>"])
      .into_grammar("<RuleList>")
      .expect("Bad EBNF Grammar")
}

macro_rules! debug {
    ($($args:tt)*) => (if cfg!(feature="debug") { eprintln!($($args)*); })
}


#[derive(Default)]
pub struct ParserBuilder(pub GrammarBuilder);

#[derive(Clone,Debug)]
enum G {Body(Vec<Vec<String>>), Part(Vec<String>), Atom(String), Nop}

// use to destructure G enum into a specific alternative
macro_rules! pull {
    ($p:path, $e:expr) => (match $e {
        $p(value) => value,
        n => panic!("Bad pull match={:?}", n)
    })
}

impl ParserBuilder {
    // Parsing terminals / non-terminal leaf nodes
    fn evaler<'a>(gb: &'a RefCell<GrammarBuilder>) -> EarleyForest<'a, G> {
        EarleyForest::new(move |symbol, token| {
            match symbol {
                "<Id>" => {
                    debug!("Adding non-term {:?}", token);
                    gb.borrow_mut().add_nonterm(token, true);
                },
                "@<Tag>" => {
                    debug!("Adding non-term {:?}", token);
                    gb.borrow_mut().add_nonterm(token, true);
                },
                "<Chars>" => {
                    debug!("Adding terminal {:?}", token);
                    let tok = token.to_string();
                    gb.borrow_mut()
                        .add_terminal(token, move |s| s == tok, true);
                },
                _ => ()
            }
            G::Atom(token.to_string())
        })
    }

    fn action_rule<'a>(ev: &mut EarleyForest<'a, G>,
                       gb: &'a RefCell<GrammarBuilder>) {
        ev.action("<Rule> -> <Id> := <Body> ;", move |mut n| {
            let id = pull!(G::Atom, n.remove(0));
            let body = pull!(G::Body, n.remove(1));
            let mut t_gb = gb.borrow_mut();
            for rule in body {
                debug!("Adding rule {:?} -> {:?}", id, rule);
                t_gb.add_rule(&id, rule.as_slice(), false);
            }
            G::Nop
        });
    }

    fn action_body<'a>(ev: &mut EarleyForest<'a, G>) {
        ev.action("<Body> -> <Body> | <Part>", |mut n| {
            let mut body = pull!(G::Body, n.remove(0));
            body.push(pull!(G::Part, n.remove(1)));
            G::Body(body)
        });
        ev.action("<Body> -> <Part>", |mut n| {
            let part = pull!(G::Part, n.remove(0));
            G::Body(vec!(part))
        });
    }

    fn action_part<'a>(ev: &mut EarleyForest<'a, G>) {
        ev.action("<Part> -> <Part> <Atom>", |mut n| {
            let mut part = pull!(G::Part, n.remove(0));
            part.push(pull!(G::Atom, n.remove(0)));
            G::Part(part)
        });
        ev.action("<Part> -> <Atom>", |mut n| {
            G::Part(vec!(pull!(G::Atom, n.remove(0))))
        });
    }

    fn action_grouping<'a>(ev: &mut EarleyForest<'a, G>,
                           gb: &'a RefCell<GrammarBuilder>) {
        ev.action("<Atom> -> ( <Body> )", move |mut n| {
            let aux = gb.borrow().unique_symbol_name();
            debug!("Adding non-term {:?}", aux);
            let mut t_gb = gb.borrow_mut();
            t_gb.add_nonterm(&aux, false);
            let body = pull!(G::Body, n.remove(1));
            for rule in body {
                debug!("Adding rule {:?} -> {:?}", aux, rule);
                t_gb.add_rule(&aux, rule.as_slice(), false);
            }
            G::Atom(aux)
        });
        ev.action("<Atom> -> ( <Body> ) @<Tag>", move |mut n| {
            let aux = pull!(G::Atom, n.remove(3));
            debug!("Adding non-term {:?}", aux);
            let mut t_gb = gb.borrow_mut();
            t_gb.add_nonterm(&aux, true);
            let body = pull!(G::Body, n.remove(1));
            for rule in body {
                debug!("Adding rule {:?} -> {:?}", aux, rule);
                t_gb.add_rule(&aux, rule.as_slice(), true);
            }
            G::Atom(aux)
        });
    }

    fn action_optional<'a>(ev: &mut EarleyForest<'a, G>,
                           gb: &'a RefCell<GrammarBuilder>) {
        ev.action("<Atom> -> [ <Body> ]", move |mut n| {
            // <Atom> -> aux ; aux -> <e> | <Body> ;
            let aux = gb.borrow().unique_symbol_name();
            debug!("Adding non-term {:?}", aux);
            let mut t_gb = gb.borrow_mut();
            t_gb.add_nonterm(&aux, false);
            let body = pull!(G::Body, n.remove(1));
            for rule in body {
                debug!("Adding rule {:?} -> {:?}", aux, rule);
                t_gb.add_rule(&aux, rule.as_slice(), false);
                debug!("Adding rule {:?} -> []", aux);
                t_gb.add_rule::<_, String>(&aux, &[], false);
            }
            G::Atom(aux)
        });
        ev.action("<Atom> -> [ <Body> ] @<Tag>", move |mut n| {
            let aux = pull!(G::Atom, n.remove(3));
            debug!("Adding non-term {:?}", aux);
            let mut t_gb = gb.borrow_mut();
            t_gb.add_nonterm(&aux, true);
            let body = pull!(G::Body, n.remove(1));
            for rule in body {
                debug!("Adding rule {:?} -> {:?}", aux, rule);
                t_gb.add_rule(&aux, rule.as_slice(), true);
                debug!("Adding rule {:?} -> []", aux);
                t_gb.add_rule::<_, String>(&aux, &[], true);
            }
            G::Atom(aux)
        });
    }

    fn action_repeat<'a>(ev: &mut EarleyForest<'a, G>,
                         gb: &'a RefCell<GrammarBuilder>) {
        ev.action("<Atom> -> { <Body> }", move |mut n| {
            // <Atom> -> aux ; aux -> <e> | <Body> aux ;
            let aux = gb.borrow().unique_symbol_name();
            debug!("Adding non-term {:?}", aux);
            let mut t_gb = gb.borrow_mut();
            t_gb.add_nonterm(&aux, false);
            let body = pull!(G::Body, n.remove(1));
            for mut rule in body {
                rule.push(aux.clone());
                debug!("Adding rule {:?} -> {:?}", aux, rule);
                t_gb.add_rule(&aux, rule.as_slice(), false);
                debug!("Adding rule {:?} -> []", aux);
                t_gb.add_rule::<_, String>(&aux, &[], false);
            }
            G::Atom(aux)
        });
        ev.action("<Atom> -> { <Body> } @<Tag>", move |mut n| {
            // <Atom> -> aux ; aux -> <e> | <Body> aux ;
            let aux = pull!(G::Atom, n.remove(3));
            debug!("Adding non-term {:?}", aux);
            let mut t_gb = gb.borrow_mut();
            t_gb.add_nonterm(&aux, true);
            let body = pull!(G::Body, n.remove(1));
            for mut rule in body {
                rule.push(aux.clone());
                debug!("Adding rule {:?} -> {:?}", aux, rule);
                t_gb.add_rule(&aux, rule.as_slice(), true);
                debug!("Adding rule {:?} -> []", aux);
                t_gb.add_rule::<_, String>(&aux, &[], true);
            }
            G::Atom(aux)
        });
    }

    // Parse a user grammar into a builder where we can plug terminal matchers
    pub fn parse_grammar(gb: GrammarBuilder, user_grammar_spec: &str)
            -> Result<GrammarBuilder, Error> {
        let user_grammar_builder = RefCell::new(gb);
        {
            let mut ev = ParserBuilder::evaler(&user_grammar_builder);
            ev.action("<RuleList> -> <RuleList> <Rule>", |_| G::Nop);
            ev.action("<RuleList> -> <Rule>", |_| G::Nop);
            ParserBuilder::action_rule(&mut ev, &user_grammar_builder);
            ParserBuilder::action_body(&mut ev);
            ParserBuilder::action_part(&mut ev);
            ParserBuilder::action_grouping(&mut ev, &user_grammar_builder);
            ParserBuilder::action_optional(&mut ev, &user_grammar_builder);
            ParserBuilder::action_repeat(&mut ev, &user_grammar_builder);
            ev.action("<Atom> -> <Id>", |mut n| n.remove(0));
            ev.action("<Atom> -> ' <Chars> '", |mut n| n.remove(1));
            ev.action("<Atom> -> \" <Chars> \"", |mut n| n.remove(1));
            // Build parser for EBNF grammar
            let ebnf = EarleyParser::new(ebnf_grammar());
            // Use EBNF parser to parse the user provided grammar
            let state = ebnf
                .parse(EbnfTokenizer::new(user_grammar_spec.chars()))
                .unwrap_or_else(|_| panic!("Failed to parse user grammar. {}",
                                           user_grammar_spec));
            // Forge user's grammar builder by executing semantic actions
            if ev.eval_all(&state)?.len() != 1 {
                panic!("BUG: EBNF grammar shouldn't be ambiguous!");
            }
        }
        // User's GrammarBuilder has all rules and non-terminals from the spec
        Ok(user_grammar_builder.into_inner())
    }

    // Plug-in functions that parse Terminals before we build the grammar
    pub fn plug_terminal<N, F>(mut self, name: N, pred: F) -> Self
            where N: Into<String>, F: 'static + Fn(&str)->bool {
        self.0.add_terminal(&name.into(), pred, false);
        ParserBuilder(self.0)
    }

    // Build a parser for the provided grammar in EBNF syntax
    pub fn into_parser(self, start: &str, grammar: &str)
            -> Result<EarleyParser, Error> {
        let user_grammar =
            ParserBuilder::parse_grammar(self.0, grammar)?
                .into_grammar(start)?;
        Ok(EarleyParser::new(user_grammar))
    }
}

#![deny(warnings)]

extern crate lexers;
extern crate earlgrey;

use self::lexers::EbnfTokenizer;
use self::earlgrey::{
    Grammar, GrammarBuilder,
    EarleyParser, ParseError, EarleyForest,
};
use std::cell::RefCell;


#[derive(Debug)]
pub struct EbnfError(pub String);

// https://en.wikipedia.org/wiki/Extended_Backus%E2%80%93Naur_form
pub fn ebnf_grammar() -> Grammar {
    GrammarBuilder::new()
      .terminal("<Id>", move |s|  // in sync w lexers::scan_identifier
                s.chars().all(|c| c.is_alphanumeric() || c == '_'))
      .terminal("<Chars>", move |s| s.chars().all(|c| !c.is_control()))
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
      .into_grammar("<RuleList>")
      .expect("Bad EBNF Grammar")
}

pub struct ParserBuilder(pub GrammarBuilder);

macro_rules! pull {
    ($p:path, $e:expr) => (match $e {
        $p(value) => value,
        n @ _ => panic!("Bad pull match={:?}", n)
    })
}

impl ParserBuilder {
    pub fn new() -> ParserBuilder { ParserBuilder(GrammarBuilder::new()) }

    pub fn builder(gb: GrammarBuilder, grammar: &str, dbg: bool)
            -> Result<GrammarBuilder, ParseError> {
        let mut tokenizer = EbnfTokenizer::scanner(grammar);
        let ebnf_parser = EarleyParser::new(ebnf_grammar());
        let state = try!(ebnf_parser.parse(&mut tokenizer));

        #[derive(Clone,Debug)]
        enum G {Body(Vec<Vec<String>>), Part(Vec<String>), Atom(String), Nop}

        let gb = RefCell::new(gb);
        {
            let mut ev = EarleyForest::new(|symbol, token| {
                match symbol {
                    "<Id>" => {
                        if dbg {eprintln!("Adding non-term {:?}", token);}
                        gb.borrow_mut().add_nonterm(token, true);
                    }, "<Chars>" => {
                        if dbg {eprintln!("Adding terminal {:?}", token);}
                        let tok = token.to_string();
                        gb.borrow_mut()
                            .add_terminal(token, move |s| s == tok, true);
                    }, _ => ()
                }
                G::Atom(token.to_string())
            });
            ev.action("<RuleList> -> <RuleList> <Rule>", |_| G::Nop);
            ev.action("<RuleList> -> <Rule>", |_| G::Nop);
            ev.action("<Rule> -> <Id> := <Body> ;", |mut n| {
                let id = pull!(G::Atom, n.remove(0));
                let body = pull!(G::Body, n.remove(1));
                let mut t_gb = gb.borrow_mut();
                for rule in body {
                    if dbg {eprintln!("Adding rule {:?} -> {:?}", id, rule);}
                    t_gb.add_rule(id.as_ref(), rule.as_slice());
                }
                G::Nop
            });
            ev.action("<Body> -> <Body> | <Part>", |mut n| {
                let mut body = pull!(G::Body, n.remove(0));
                body.push(pull!(G::Part, n.remove(1)));
                G::Body(body)
            });
            ev.action("<Body> -> <Part>", |mut n| {
                let part = pull!(G::Part, n.remove(0));
                G::Body(vec!(part))
            });
            ev.action("<Part> -> <Part> <Atom>", |mut n| {
                let mut part = pull!(G::Part, n.remove(0));
                part.push(pull!(G::Atom, n.remove(0)));
                G::Part(part)
            });
            ev.action("<Part> -> <Atom>", |mut n| {
                G::Part(vec!(pull!(G::Atom, n.remove(0))))
            });
            ev.action("<Atom> -> <Id>", |mut n| n.remove(0));
            ev.action("<Atom> -> ' <Chars> '", |mut n| n.remove(1));
            ev.action("<Atom> -> \" <Chars> \"", |mut n| n.remove(1));
            ev.action("<Atom> -> ( <Body> )", |mut n| {
                let aux = gb.borrow().unique_symbol_name();
                let body = pull!(G::Body, n.remove(1));
                let mut t_gb = gb.borrow_mut();
                if dbg {eprintln!("Adding non-term {:?}", aux);}
                t_gb.add_nonterm(aux.as_ref(), false);
                for rule in body {
                    if dbg {eprintln!("Adding rule {:?} -> {:?}", aux, rule);}
                    t_gb.add_rule(aux.as_ref(), rule.as_slice());
                }
                G::Atom(aux)
            });
            ev.action("<Atom> -> [ <Body> ]", |mut n| {
                // <Atom> -> aux ; aux -> <e> | <Body> ;
                let aux = gb.borrow().unique_symbol_name();
                let body = pull!(G::Body, n.remove(1));
                let mut t_gb = gb.borrow_mut();
                if dbg {eprintln!("Adding non-term {:?}", aux);}
                t_gb.add_nonterm(aux.as_ref(), false);
                for rule in body {
                    if dbg {
                        eprintln!("Adding rule {:?} -> []", aux);
                        eprintln!("Adding rule {:?} -> {:?}", aux, rule);
                    }
                    t_gb.add_rule(aux.as_ref(), rule.as_slice());
                    t_gb.add_rule::<_, String>(aux.as_ref(), &[]);
                }
                G::Atom(aux)
            });
            ev.action("<Atom> -> { <Body> }", |mut n| {
                // <Atom> -> aux ; aux -> <e> | <Body> aux ;
                let aux = gb.borrow().unique_symbol_name();
                let body = pull!(G::Body, n.remove(1));
                let mut t_gb = gb.borrow_mut();
                if dbg {eprintln!("Adding non-term {:?}", aux);}
                t_gb.add_nonterm(aux.as_ref(), false);
                for mut rule in body {
                    if dbg {
                        eprintln!("Adding rule {:?} -> []", aux);
                        eprintln!("Adding rule {:?} -> {:?}", aux, rule);
                    }
                    rule.push(aux.clone());
                    t_gb.add_rule(aux.as_ref(), rule.as_slice());
                    t_gb.add_rule::<_, String>(aux.as_ref(), &[]);
                }
                G::Atom(aux)
            });
            if ev.eval_all(&state).expect("EBNF Bug").len() != 1 {
                panic!("EBNF grammar Bug: shouldn't be ambiguous!");
            }
        }
        Ok(gb.into_inner())
    }

    // Plug-in functions that parse Terminals before we build the grammar
    pub fn plug_terminal<N, F>(mut self, name: N, pred: F) -> Self
            where N: Into<String>, F: 'static + Fn(&str)->bool {
        self.0.add_terminal(name.into().as_ref(), pred, false);
        ParserBuilder(self.0)
    }

    // Build a parser for the provided grammar in EBNF syntax
    pub fn into_parser(self, start: &str, grammar: &str)
            -> Result<EarleyParser, EbnfError> {
        let grammar = ParserBuilder::builder(self.0, grammar, false)
                        .or_else(|e| Err(EbnfError(format!("{:?}", e))))?
                        .into_grammar(start)
                        .or_else(|e| Err(EbnfError(format!("{:?}", e))))?;
        Ok(EarleyParser::new(grammar))
    }
}

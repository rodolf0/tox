use lexers::{EbnfTokenizer, Scanner};
use parser::EarleyParser;
use trees::EarleyEvaler;
use types::{Grammar, GrammarBuilder};
use util::Sexpr;
use std::cell::RefCell;

// https://en.wikipedia.org/wiki/Extended_Backus%E2%80%93Naur_form
pub fn ebnf_grammar() -> Grammar {
    GrammarBuilder::new()
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
      .symbol("<Body>")
      .symbol("<Part>")
      .symbol("<Atom>")
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
}

pub struct ParserBuilder(GrammarBuilder);

macro_rules! pull {
    ($p:path, $e:expr) => (match $e {
        $p(value) => value,
        n @ _ => panic!("Bad pull match={:?}", n)
    })
}

impl ParserBuilder {
    pub fn new() -> ParserBuilder { ParserBuilder(GrammarBuilder::new()) }

    fn builder(gb: GrammarBuilder, grammar: &str, dbg: bool) -> GrammarBuilder {
        let ebnf_parser = EarleyParser::new(ebnf_grammar());
        let mut tokenizer = EbnfTokenizer::from_str(grammar);
        let state = ebnf_parser.parse(&mut tokenizer).unwrap_or_else(
            |e| panic!("ParserBuilder error: {:?}", e));

        #[derive(Clone, Debug)]
        enum G {Body(Vec<Vec<String>>), Part(Vec<String>), Atom(String), Nop}

        let gb = RefCell::new(gb);
        {
            let mut ev = EarleyEvaler::new(|symbol, token| {
                match symbol {
                    "<Id>" => {
                        if dbg {println!("Adding non-term {:?}", token);}
                        gb.borrow_mut().add_symbol(token, true);
                    }, "<Chars>" => {
                        if dbg {println!("Adding terminal {:?}", token);}
                        let tok = token.to_string();
                        gb.borrow_mut().add_symbol(
                            (token, move |s: &str| s == tok), true);
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
                    if dbg {println!("Adding rule {:?} -> {:?}", id, rule);}
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
                if dbg {println!("Adding non-term {:?}", aux);}
                t_gb.add_symbol(aux.as_ref(), false);
                for rule in body {
                    if dbg {println!("Adding rule {:?} -> {:?}", aux, rule);}
                    t_gb.add_rule(aux.as_ref(), rule.as_slice());
                }
                G::Atom(aux)
            });
            ev.action("<Atom> -> [ <Body> ]", |mut n| {
                // <Atom> -> aux ; aux -> <e> | <Body> ;
                let aux = gb.borrow().unique_symbol_name();
                let body = pull!(G::Body, n.remove(1));
                let mut t_gb = gb.borrow_mut();
                if dbg {println!("Adding non-term {:?}", aux);}
                t_gb.add_symbol(aux.as_ref(), false);
                for rule in body {
                    if dbg {
                        println!("Adding rule {:?} -> []", aux);
                        println!("Adding rule {:?} -> {:?}", aux, rule);
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
                if dbg {println!("Adding non-term {:?}", aux);}
                t_gb.add_symbol(aux.as_ref(), false);
                for mut rule in body {
                    if dbg {
                        println!("Adding rule {:?} -> []", aux);
                        println!("Adding rule {:?} -> {:?}", aux, rule);
                    }
                    rule.push(aux.clone());
                    t_gb.add_rule(aux.as_ref(), rule.as_slice());
                    t_gb.add_rule::<_, String>(aux.as_ref(), &[]);
                }
                G::Atom(aux)
            });
            if ev.eval_all(&state).len() != 1 {
                panic!("EBNF grammar shouldn't be ambiguous!");
            }
        }
        gb.into_inner()
    }

    pub fn into_parser(self, start: &str, grammar: &str) -> EarleyParser {
        let gb = ParserBuilder::builder(self.0, grammar, false);
        EarleyParser::new(gb.into_grammar(start))
    }

    pub fn treeficator<'a>(self, start: &str, grammar: &'a str)
            -> Box<Fn(&mut Scanner<String>)->Vec<Vec<Sexpr>> + 'a> {
        let grammar = ParserBuilder::builder(self.0, grammar, false)
            .into_grammar(start);
        // Add semantic actions that flatten the parse tree
        let mut ev = EarleyEvaler::new(|_, tok| Sexpr::Atom(tok.to_string()));
        for rule in grammar.rules() {
            ev.action(&rule.clone(), move |mut nodes| match nodes.len() {
                1 => nodes.swap_remove(0),
                _ => Sexpr::List(nodes),
            });
        }
        let parser = EarleyParser::new(grammar);
        Box::new(move |mut tokenizer| {
            match parser.parse(&mut tokenizer) {
                Ok(state) => ev.eval_all(&state),
                Err(e) => panic!("Parse error: {:?}", e)
            }
        })
    }

    pub fn plug_terminal<N, F>(mut self, name: N, pred: F) -> ParserBuilder
            where N: Into<String>, F: 'static + Fn(&str)->bool {
        self.0.add_symbol((name.into(), pred), false);
        ParserBuilder(self.0)
    }
}

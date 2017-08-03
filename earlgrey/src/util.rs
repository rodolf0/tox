#![deny(warnings)]

use grammar::Grammar;
use trees::EarleyEvaler;


#[derive(Debug,Clone,PartialEq)]
pub enum Tree {
    // ("[+-]", "+")
    Leaf(String, String),
    // ("E -> E [+-] E", [("n", "5"), ("[+-]", "+"), ("E -> E * E", [...])])
    Node(String, Vec<Tree>),
}

#[derive(Clone,Debug)]
pub enum Sexpr {
    Atom(String),
    List(Vec<Sexpr>),
}

///////////////////////////////////////////////////////////////////////////////

impl Tree {
    pub fn print(&self) {
        self.print_helper("")
    }

    fn print_helper(&self, level: &str) {
        match self {
            &Tree::Leaf(ref sym, ref lexeme) => {
                println!("{}`-- {:?} ==> {:?}", level, sym, lexeme);
            },
            &Tree::Node(ref spec, ref subn) => {
                println!("{}`-- {:?}", level, spec);
                if let Some((last, rest)) = subn.split_last() {
                    let l = format!("{}  |", level);
                    for n in rest { n.print_helper(&l); }
                    let l = format!("{}   ", level);
                    last.print_helper(&l);
                }
            }
        }
    }

    pub fn builder<'a>(g: Grammar) -> EarleyEvaler<'a, Tree> {
        let mut evaler = EarleyEvaler::new(
            |sym, tok| Tree::Leaf(sym.to_string(), tok.to_string())
        );
        for rule in g.str_rules() {
            evaler.action(&rule.to_string(), move |nodes|
                          Tree::Node(rule.to_string(), nodes));
        }
        evaler
    }
}

impl Sexpr {
    pub fn print(&self) {
        self.print_helper("")
    }

    fn print_helper(&self, level: &str) {
        match self {
            &Sexpr::Atom(ref lexeme) => {
                println!("{}`-- {:?}", level, lexeme);
            },
            &Sexpr::List(ref subn) => {
                println!("{}`--", level);
                if let Some((last, rest)) = subn.split_last() {
                    let l = format!("{}  |", level);
                    for n in rest { n.print_helper(&l); }
                    let l = format!("{}   ", level);
                    last.print_helper(&l);
                }
            }
        }
    }
}

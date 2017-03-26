use types::Grammar;
use trees::EarleyEvaler;

#[derive(Debug, Clone, PartialEq)]
pub enum Subtree {
    // ("[+-]", "+")
    Leaf(String, String),
    // ("E -> E [+-] E", [("n", "5"), ("[+-]", "+"), ("E -> E * E", [...])])
    Node(String, Vec<Subtree>),
}

impl Subtree {
    pub fn print(&self) {
        self.print_helper("")
    }
    fn print_helper(&self, level: &str) {
        match self {
            &Subtree::Leaf(ref sym, ref lexeme) => {
                println!("{}`-- {:?} ==> {:?}", level, sym, lexeme);
            },
            &Subtree::Node(ref spec, ref subn) => {
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
}

pub fn subtree_evaler<'a>(g: Grammar) -> EarleyEvaler<'a, Subtree> {
    let mut evaler = EarleyEvaler::new(
        |sym, tok| Subtree::Leaf(sym.to_string(), tok.to_string())
    );
    for rule in g.rules() {
        evaler.action(&rule.clone(), move |nodes|
                      Subtree::Node(rule.clone(), nodes));
    }
    evaler
}


#[derive(Clone, Debug)]
pub enum Sexpr {
    Atom(String),
    List(Vec<Sexpr>),
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

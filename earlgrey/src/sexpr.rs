#![deny(warnings)]

use std::fmt::Debug;

#[derive(Clone, Debug)]
pub enum Sexpr {
    Atom(String),
    List(Vec<Sexpr>),
}

impl Sexpr {
    pub fn print(&self) -> String {
        let mut out = String::new();
        self.print_helper("", &mut out);
        out
    }

    fn print_helper(&self, indent: &str, out: &mut String) {
        match *self {
            Sexpr::Atom(ref lexeme) => *out += &format!("\u{2500} {}\n", lexeme),
            Sexpr::List(ref subn) => {
                if let Some((first, rest)) = subn.split_first() {
                    if let Some((last, rest)) = rest.split_last() {
                        *out += "\u{252c}";
                        first.print_helper(&format!("{}\u{2502}", indent), out);
                        for mid in rest {
                            *out += &format!("{}\u{251c}", indent);
                            mid.print_helper(&format!("{}\u{2502}", indent), out);
                        }
                        *out += &format!("{}\u{2570}", indent);
                        last.print_helper(&format!("{} ", indent), out);
                    } else {
                        *out += "\u{2500} \u{03b5}\n";
                    }
                } else {
                    *out += "\u{2500} \u{03b5}\n";
                }
            }
        }
    }
}

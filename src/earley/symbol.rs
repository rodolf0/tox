use std::{fmt, hash, mem};

pub enum Symbol {
    NonTerm(String),
    Terminal(String, Box<Fn(&str)->bool>),
}

impl Symbol {
    pub fn nonterm<S: Into<String>>(s: S) -> Self { Symbol::NonTerm(s.into()) }

    pub fn terminal<S, F>(name: S, f: F) -> Self
    where S: Into<String>, F: 'static + Fn(&str)->bool {
        Symbol::Terminal(name.into(), Box::new(f))
    }

    pub fn name<'a>(&'a self) -> &'a str {
        match self {
            &Symbol::NonTerm(ref name) => name,
            &Symbol::Terminal(ref name, _) => name,
        }
    }

    pub fn term_match(&self, input: &str) -> bool {
        match self {
            &Symbol::Terminal(_, ref f) => f(input),
            &Symbol::NonTerm(_) => false,
        }
    }

    pub fn is_nonterm(&self) -> bool {
        match self { &Symbol::NonTerm(_) => true, _ => false }
    }

    pub fn is_term(&self) -> bool {
        match self { &Symbol::Terminal(_, _) => true, _ => false }
    }
}

impl fmt::Debug for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Symbol::NonTerm(ref name) => write!(f, "{}", name),
            &Symbol::Terminal(ref name, _) => write!(f, "'{}'", name),
        }
    }
}

impl hash::Hash for Symbol {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        match self {
            &Symbol::NonTerm(ref name) => name.hash(state),
            &Symbol::Terminal(ref name, ref f) => {
                name.hash(state);
                let (x, y) = unsafe { mem::transmute::<_, (usize, usize)>(&**f) };
                x.hash(state); y.hash(state);
            }
        }
    }
}

impl PartialEq for Symbol {
    fn eq(&self, other: &Symbol) -> bool {
        match (self, other) {
            (&Symbol::NonTerm(ref a), &Symbol::NonTerm(ref b)) => a == b,
            (&Symbol::Terminal(ref name_a, ref func_a),
             &Symbol::Terminal(ref name_b, ref func_b)) => {
                name_a == name_b && unsafe {
                    let a = mem::transmute::<_, (usize, usize)>(&**func_a);
                    let b = mem::transmute::<_, (usize, usize)>(&**func_b);
                    a == b
                }
            },
            _ => false,
        }
    }
}

impl Eq for Symbol {}

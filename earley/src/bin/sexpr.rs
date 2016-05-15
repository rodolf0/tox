#![cfg(not(test))]

extern crate linenoise;
extern crate regex;
extern crate lexers;
extern crate earley;

use earley::Subtree;
use std::collections::HashMap;
use std::fmt;

// expr     -> expr '[+-]' addpart | addpart
// addpart  -> addpart '[*%/]' uminus | uminus
// uminus   -> '-' uminus | mulpart
// mulpart  -> ufact '^' uminus | ufact
// ufact    -> ufact '!' | group
// group    -> num | id | '(' expr ')' | func
// func     -> id '(' args ')'
// args     -> args ',' expr | expr | <e>

fn build_grammar() -> earley::Grammar {
    use earley::Symbol;
    let num = regex::Regex::new(r"^-?\d+(?:\.\d+)?(?:[eE][-+]?\d+)?$").unwrap();
    let var = regex::Regex::new(r"^[A-Za-z_]+[A-Za-z0-9_]*$").unwrap();
    let mut gb = earley::GrammarBuilder::new();
    gb.symbol(Symbol::nonterm("expr"))
      .symbol(Symbol::nonterm("addpart"))
      .symbol(Symbol::nonterm("uminus"))
      .symbol(Symbol::nonterm("mulpart"))
      .symbol(Symbol::nonterm("ufact"))
      .symbol(Symbol::nonterm("group"))
      .symbol(Symbol::nonterm("func"))
      .symbol(Symbol::nonterm("args"))
      .symbol(Symbol::terminal("[n]", move |n: &str| num.is_match(n)))
      .symbol(Symbol::terminal("[v]", move |n: &str| var.is_match(n)))
      .symbol(Symbol::terminal("[+]", |n: &str| n == "+" || n == "-"))
      .symbol(Symbol::terminal("[*]", |n: &str| n == "*" || n == "/" || n == "%"))
      .symbol(Symbol::terminal("[-]", |n: &str| n == "-"))
      .symbol(Symbol::terminal("[^]", |n: &str| n == "^"))
      .symbol(Symbol::terminal("[!]", |n: &str| n == "!"))
      .symbol(Symbol::terminal("[,]", |n: &str| n == ","))
      .symbol(Symbol::terminal("[(]", |n: &str| n == "("))
      .symbol(Symbol::terminal("[)]", |n: &str| n == ")"))
      ;
    gb.rule("expr",    vec!["addpart"])
      .rule("expr",    vec!["expr", "[+]", "addpart"])
      .rule("addpart", vec!["uminus"])
      .rule("addpart", vec!["addpart", "[*]", "uminus"])
      .rule("uminus",  vec!["mulpart"])
      .rule("uminus",  vec!["[-]", "uminus"])
      .rule("mulpart", vec!["ufact"])
      .rule("mulpart", vec!["ufact", "[^]", "uminus"])
      .rule("ufact",   vec!["group"])
      .rule("ufact",   vec!["ufact", "[!]"])
      .rule("group",   vec!["[n]"])
      .rule("group",   vec!["[v]"])
      .rule("group",   vec!["[(]", "expr", "[)]"])
      .rule("group",   vec!["func"])
      .rule("func",    vec!["[v]", "[(]", "args", "[)]"])
      .rule("args",    vec!["expr"])
      .rule("args",    vec!["args", "[,]", "expr"])
      .rule("args",    vec![])
      ;
    gb.into_grammar("expr")
}


#[derive(Clone)]
enum Sexpr {
    S(String),
    List(Vec<Sexpr>),
}

impl Sexpr {
    fn to_string(&self) -> String {
        match self {
            &Sexpr::List(ref c) =>
                format!("({})", c.iter().map(|e| e.to_string()).collect::<Vec<_>>().join(" ")),
            &Sexpr::S(ref s) => s.clone(),
        }
    }
}

impl fmt::Debug for Sexpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

type Action<R> = Box<Fn(Vec<R>) -> R>;
type TokHandle<R> = for<'r> Fn(&'r str, &'r str) -> R;

fn semanter<R>(subtree: &Subtree,
               actions: &HashMap<String, Action<R>>,
               tokh: &TokHandle<R>) -> R {
    match subtree {
        &Subtree::Node(ref name, ref value) => tokh(name, value),
        &Subtree::SubT(ref rule, ref subtrees) => {
            let args = subtrees.iter().map(|t| semanter(t, actions, tokh)).collect();
            let action = &actions[rule.trim()];
            action(args)
        }
    }
}

// TODO: get rid of this
fn b<R, F: 'static + Fn(Vec<R>)->R>(f: F) -> Action<R> { Box::new(f) }

fn sexpr_actions() -> HashMap<String, Action<Sexpr>> {
    vec![
        ("expr -> addpart"               , b(|a: Vec<Sexpr>| a[0].clone())),
        ("expr -> expr [+] addpart"      , b(|a: Vec<Sexpr>| Sexpr::List(a))),
        ("addpart -> uminus"             , b(|a: Vec<Sexpr>| a[0].clone())),
        ("addpart -> addpart [*] uminus" , b(|a: Vec<Sexpr>| Sexpr::List(a))),
        ("uminus -> mulpart"             , b(|a: Vec<Sexpr>| a[0].clone())),
        ("uminus -> [-] uminus"          , b(|a: Vec<Sexpr>| Sexpr::List(a))),
        ("mulpart -> ufact"              , b(|a: Vec<Sexpr>| a[0].clone())),
        ("mulpart -> ufact [^] uminus"   , b(|a: Vec<Sexpr>| Sexpr::List(a))),
        ("ufact -> group"                , b(|a: Vec<Sexpr>| a[0].clone())),
        ("ufact -> ufact [!]"            , b(|a: Vec<Sexpr>| Sexpr::List(a))),
        ("group -> [n]"                  , b(|a: Vec<Sexpr>| a[0].clone())),
        ("group -> [v]"                  , b(|a: Vec<Sexpr>| a[0].clone())),
        ("group -> [(] expr [)]"         , b(|a: Vec<Sexpr>| a[1].clone())),
        ("group -> func"                 , b(|a: Vec<Sexpr>| a[0].clone())),
        ("func -> [v] [(] args [)]"      , b(|a: Vec<Sexpr>| Sexpr::List(vec![a[0].clone(), a[2].clone()]))),
        ("args -> expr"                  , b(|a: Vec<Sexpr>| a[0].clone())),
        ("args -> args [,] expr"         , b(|a: Vec<Sexpr>| Sexpr::List(a))),
        ("args ->"                       , b(|a: Vec<Sexpr>| a[0].clone())),
    ].into_iter().map(|(spec, func)| (spec.to_string(), func))
                 .collect::<HashMap<String, Action<Sexpr>>>()
}


fn dotprinter(node: &Subtree, n: usize) {
    match node {
        &Subtree::Node(ref term, ref value) => println!("  \"{}. {}\" -> \"{}. {}\"", n, term, n + 1, value),
        &Subtree::SubT(ref spec, ref childs) => for (nn, c) in childs.iter().enumerate() {
            let x = match c {
                &Subtree::Node(ref term, _) => term,
                &Subtree::SubT(ref sspec, _) => sspec,
            };
            println!("  \"{}. {}\" -> \"{}. {}\"", n, spec, n + nn + 100, x);
            dotprinter(&c, n + nn + 100);
        }
    }
}

struct Tokenizer(lexers::Scanner<char>);

impl lexers::Nexter<String> for Tokenizer {
    fn get_item(&mut self) -> Option<String> {
        self.0.ignore_ws();
        lexers::scan_number(&mut self.0)
            .or_else(|| lexers::scan_identifier(&mut self.0))
            .or_else(|| lexers::scan_math_op(&mut self.0))
    }
}

impl Tokenizer {
    fn from_str(input: &str) -> lexers::Scanner<String> {
        lexers::Scanner::new(
            Box::new(Tokenizer(lexers::Scanner::from_str(&input))))
    }
}

fn main() {
    let parser = earley::EarleyParser::new(build_grammar());

    if std::env::args().len() > 1 {
        let input = std::env::args().skip(1).
            collect::<Vec<String>>().join(" ");
        match parser.parse(&mut Tokenizer::from_str(&input)) {
            Ok(estate) => {
                let tree = earley::one_tree(parser.g.start(), &estate);
                println!("digraph x {{");
                dotprinter(&tree, 0);
                println!("}}");
            },
            Err(e) => println!("Parse err: {:?}", e)
        }
        return;
    }

    while let Some(input) = linenoise::input("~> ") {
        linenoise::history_add(&input[..]);
        match parser.parse(&mut Tokenizer::from_str(&input)) {
            Ok(estate) => {
                let tree = earley::one_tree(parser.g.start(), &estate);
                let s = semanter(&tree, &sexpr_actions(), &|_, value: &str| Sexpr::S(value.to_string()));
                println!("{:?}", s);
            },
            Err(e) => println!("Parse err: {:?}", e)
        }
    }
}

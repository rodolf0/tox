#![cfg(not(test))]

extern crate linenoise;
extern crate regex;
extern crate lexers;
extern crate earley;

use earley::Subtree;
use std::collections::HashMap;
use std::fmt;

// Grammar with unary-minus binding tighter than power
//
// expr     -> expr '[+-]' addpart | addpart
// addpart  -> addpart '[*%/]' mulpart | mulpart
// mulpart  -> uminus '^' mulpart | uminus
// uminus   -> '-' uminus | group
// group    -> num | id | '(' expr ')'

// Grammar with unary-minus binding less tight than power
//
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
    let sss = regex::Regex::new(r"^[A-Za-z_]+[A-Za-z0-9_]*$").unwrap();
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
      .symbol(Symbol::terminal("[v]", move |n: &str| sss.is_match(n)))
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
type TokHandle<R> = for<'r> FnMut(&'r str, &'r str) -> R;

fn semanter<R>(subtree: &Subtree,
               actions: &HashMap<String, Action<R>>,
               tokh: &mut TokHandle<R>) -> R {
    match subtree {
        &Subtree::Node(ref name, ref value) => tokh(name, value),
        &Subtree::SubT(ref rule, ref subtrees) => {
            let args = subtrees.iter().map(|t| semanter(t, actions, tokh)).collect();
            let action = &actions[rule.trim()];
            action(args)
        }
    }
}

/*
fn semanter2<R, TH>(subtree: &Subtree, actions: &HashMap<String, Action<R>>, tokh: &mut TH) -> R
        where TH: for<'r> FnMut(&'r str, &'r str) -> R {
    match subtree {
        &Subtree::Node(ref name, ref value) => tokh(name, value),
        &Subtree::SubT(ref rule, ref subtrees) => for t in subtrees {
            let action = &actions[rule.trim()];
            action(args);
            semanter2(t, actions, tokh)
        }
    }
}
*/

//fn printer(node: &Subtree, n: usize) {
    //match node {
        //&Subtree::Node(ref term, ref value) => println!("  \"{}. {}\" -> \"{}. {}\"", n, term, n + 1, value),
        //&Subtree::SubT(ref spec, ref childs) => for (nn, c) in childs.iter().enumerate() {
            //let x = match c {
                //&Subtree::Node(ref term, _) => term,
                //&Subtree::SubT(ref sspec, _) => sspec,
            //};
            //println!("  \"{}. {}\" -> \"{}. {}\"", n, spec, n + nn + 100, x);
            //printer(&c, n + nn + 100);
        //}
    //}
//};


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

fn prn_actions() -> HashMap<String, Action<usize>> {
    build_grammar().all_rules().cloned()
        .map(|rule| {
            let spec = format!("{} -> {}", rule.name(), rule.spec());
            let action = b(move |childs: Vec<usize>| {
                for (i, child) in rule.spec_parts().iter().enumerate() {
                    println!("\"{}. {}\" -> \"{}. {}\"", childs[i] + 1, rule.name(), childs[i], child);
                }
                childs.last().unwrap() + 1
            });
            (spec, action)
        })
        .collect::<HashMap<_, _>>()
}

fn main() {
    let parser = earley::EarleyParser::new(build_grammar());
    while let Some(input) = linenoise::input("~> ") {
        linenoise::history_add(&input[..]);
        let mut input = lexers::DelimTokenizer::from_str(&input, " ", true);
        match parser.parse(&mut input) {
            Ok(estate) => {
                let tree = earley::one_tree(parser.g.start(), &estate);

                //let s = semanter(&tree, &sexpr_actions(), &|_, value: &str| Sexpr::S(value.to_string()));
                //println!("{:?}", s);
                let mut nnn = 0;
                semanter(&tree, &prn_actions(), &mut move |terminal: &str, value: &str| {
                    nnn += 100;
                    println!("\"{}. {}\" -> \"{}. {}\"", nnn + 1, terminal, nnn, value);
                    nnn + 1
                });
            },
            Err(e) => println!("Parse err: {:?}", e)
        }
    }
}

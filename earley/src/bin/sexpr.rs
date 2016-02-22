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

fn build_parser() -> earley::EarleyParser {
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
    earley::EarleyParser::new(gb.into_grammar("expr"))
}


#[derive(Clone)]
enum Sexpr {
    Nil,
    S(String),
    List(Vec<Sexpr>),
}

impl Sexpr {
    fn to_string(&self) -> String {
        match self {
            &Sexpr::List(ref c) =>
                format!("({})", c.iter().map(|e| e.to_string()).collect::<Vec<_>>().join(" ")),
            &Sexpr::S(ref s) => s.clone(),
            _ => "nil".to_string(),
        }
    }
}

impl fmt::Debug for Sexpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

type Action = Box<Fn(Vec<Sexpr>) -> Sexpr>;

fn semanter<TH>(subtree: &Subtree, actions: &HashMap<&str, Action>, tokh: &TH) -> Sexpr
        where TH: for<'r> Fn(&'r str, &'r str) -> Sexpr {
    match subtree {
        &Subtree::Node(ref name, ref value) => tokh(name, value),
        &Subtree::SubT(ref rule, ref subtrees) => {
            let args = subtrees.iter().map(|t| semanter(t, actions, tokh)).collect();
            match actions.get(rule.trim()) { // trim -> as_str
                Some(action) => action(args),
                _ => Sexpr::Nil,
            }
        }
    }
}

// TODO: get rid of this
fn b<F: 'static + Fn(Vec<Sexpr>)->Sexpr>(f: F) -> Action { Box::new(f) }

fn sexpr_actions<'x>() -> HashMap<&'x str, Action> {
    let mut actions = HashMap::new();
    actions.insert("expr"                          , b(|args| Sexpr::List(args)));
    actions.insert("expr -> addpart"               , b(|args| args[0].clone()));
    actions.insert("expr -> expr [+] addpart"      , b(|args| Sexpr::List(args)));
    actions.insert("addpart -> uminus"             , b(|args| args[0].clone()));
    actions.insert("addpart -> addpart [*] uminus" , b(|args| Sexpr::List(args)));
    actions.insert("uminus -> mulpart"             , b(|args| args[0].clone()));
    actions.insert("uminus -> [-] uminus"          , b(|args| Sexpr::List(args)));
    actions.insert("mulpart -> ufact"              , b(|args| args[0].clone()));
    actions.insert("mulpart -> ufact [^] uminus"   , b(|args| Sexpr::List(args)));
    actions.insert("ufact -> group"                , b(|args| args[0].clone()));
    actions.insert("ufact -> ufact [!]"            , b(|args| Sexpr::List(args)));
    actions.insert("group -> [n]"                  , b(|args| args[0].clone()));
    actions.insert("group -> [v]"                  , b(|args| args[0].clone()));
    actions.insert("group -> [(] expr [)]"         , b(|args| args[1].clone())); // drop parens
    actions.insert("group -> func"                 , b(|args| args[0].clone()));
    actions.insert("func -> [v] [(] args [)]"      , b(|args| Sexpr::List(vec![args[0].clone(), args[2].clone()])));
    actions.insert("args -> expr"                  , b(|args| args[0].clone()));
    actions.insert("args -> args [,] expr"         , b(|args| Sexpr::List(args)));
    actions.insert("args ->"                       , b(|args| args[0].clone()));
    actions
}

fn prn_actions<'x>() -> HashMap<&'x str, Action> {
    let mut actions = HashMap::new();
    //actions.insert("expr"                          , b(|args| {println!("hola"); Sexpr::Nil}));
    //actions.insert("expr -> addpart"               , b(|args| args[0].clone()));
    //actions.insert("expr -> expr [+] addpart"      , b(|args| Sexpr::List(args)));
    //actions.insert("addpart -> uminus"             , b(|args| args[0].clone()));
    //actions.insert("addpart -> addpart [*] uminus" , b(|args| Sexpr::List(args)));
    //actions.insert("uminus -> mulpart"             , b(|args| args[0].clone()));
    //actions.insert("uminus -> [-] uminus"          , b(|args| Sexpr::List(args)));
    //actions.insert("mulpart -> ufact"              , b(|args| args[0].clone()));
    //actions.insert("mulpart -> ufact [^] uminus"   , b(|args| Sexpr::List(args)));
    //actions.insert("ufact -> group"                , b(|args| args[0].clone()));
    //actions.insert("ufact -> ufact [!]"            , b(|args| Sexpr::List(args)));
    //actions.insert("group -> [n]"                  , b(|args| args[0].clone()));
    //actions.insert("group -> [v]"                  , b(|args| args[0].clone()));
    //actions.insert("group -> [(] expr [)]"         , b(|args| args[1].clone())); // drop parens
    //actions.insert("group -> func"                 , b(|args| args[0].clone()));
    //actions.insert("func -> [v] [(] args [)]"      , b(|args| Sexpr::List(vec![args[0].clone(), args[2].clone()])));
    //actions.insert("args -> expr"                  , b(|args| args[0].clone()));
    //actions.insert("args -> args [,] expr"         , b(|args| Sexpr::List(args)));
    //actions.insert("args ->"                       , b(|args| args[0].clone()));
    actions
}

fn main() {
    let parser = build_parser();
    while let Some(input) = linenoise::input("~> ") {
        linenoise::history_add(&input[..]);
        let mut input = lexers::DelimTokenizer::from_str(&input, " ", true);
        match parser.parse(&mut input) {
            Ok(estate) => {
                let tree = earley::one_tree(parser.g.start(), &estate);

                let s = semanter(&tree, &prn_actions(), &|_, value: &str| Sexpr::S(value.to_string()));
                println!("{:?}", s);

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

                //println!("digraph arbol {{");
                //printer(&tree.unwrap(), 0);
                //println!("}}");
            },
            Err(e) => println!("Parse err: {:?}", e)
        }
    }
}

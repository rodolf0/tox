#![cfg(not(test))]

extern crate linenoise;
extern crate regex;
extern crate lexers;
extern crate earley;

use earley::Subtree;
use std::collections::HashMap;

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
      .symbol(Symbol::terminal("[+-]", |n: &str| n == "+" || n == "-"))
      .symbol(Symbol::terminal("[*/%]", |n: &str| n == "*" || n == "/" || n == "%"))
      .symbol(Symbol::terminal("[-]", |n: &str| n == "-"))
      .symbol(Symbol::terminal("[^]", |n: &str| n == "^"))
      .symbol(Symbol::terminal("[!]", |n: &str| n == "!"))
      .symbol(Symbol::terminal("[,]", |n: &str| n == ","))
      .symbol(Symbol::terminal("[(]", |n: &str| n == "("))
      .symbol(Symbol::terminal("[)]", |n: &str| n == ")"))
      ;
    gb.rule("expr",    vec!["addpart"])
      .rule("expr",    vec!["expr", "[+-]", "addpart"])
      .rule("addpart", vec!["uminus"])
      .rule("addpart", vec!["addpart", "[*/%]", "uminus"])
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

fn seval(n: &Subtree, ctx: &mut HashMap<String, f64>) -> f64 {
    use std::str::FromStr;
    use std::f64::consts;
    match n {
        &Subtree::Node(ref key, ref val) => match key.as_ref() {
            "[n]" => f64::from_str(&val).unwrap(),
            "[v]" => match val.as_ref() {
                "e" => consts::E,
                "pi" => consts::PI,
                x => ctx[x]
            },
            _ => unreachable!()
        },
        &Subtree::SubT(ref key, ref subn) => match key.as_ref() {
            "expr -> addpart" => seval(&subn[0], ctx),
            "expr -> expr [+-] addpart" => match &subn[1] {
                &Subtree::Node(_, ref op) if op == "+" => seval(&subn[0], ctx) + seval(&subn[2], ctx),
                &Subtree::Node(_, ref op) if op == "-" => seval(&subn[0], ctx) - seval(&subn[2], ctx),
                _ => unreachable!()
            },
            "addpart -> uminus" => seval(&subn[0], ctx),
            "addpart -> addpart [*/%] uminus" => match &subn[1] {
                &Subtree::Node(_, ref op) if op == "*" => seval(&subn[0], ctx) * seval(&subn[2], ctx),
                &Subtree::Node(_, ref op) if op == "-" => seval(&subn[0], ctx) / seval(&subn[2], ctx),
                &Subtree::Node(_, ref op) if op == "%" => seval(&subn[0], ctx) % seval(&subn[2], ctx),
                _ => unreachable!()
            },
            "uminus -> mulpart" => seval(&subn[0], ctx),
            "uminus -> [-] uminus" => - seval(&subn[1], ctx),
            "mulpart -> ufact" => seval(&subn[0], ctx),
            "mulpart -> ufact [^] uminus" => match &subn[1] {
                &Subtree::Node(_, ref op) if op == "^" => seval(&subn[0], ctx).powf(seval(&subn[2], ctx)),
                _ => unreachable!()
            },
            "ufact -> group" => seval(&subn[0], ctx),
            "ufact -> ufact [!]" => panic!(), // no gamma function?
            "group -> [n]" => seval(&subn[0], ctx),
            "group -> [v]" => seval(&subn[0], ctx),
            "group -> [(] expr [)]" => seval(&subn[1], ctx),
            "group -> func" => seval(&subn[0], ctx),
            "func -> [v] [(] args [)]" => match &subn[0] {
                &Subtree::Node(_, ref f) if f == "sin" => seval(&subn[2], ctx).sin(),
                &Subtree::Node(_, ref f) if f == "cos" => seval(&subn[2], ctx).cos(),
                _ => panic!()
            },
            "args -> expr" => seval(&subn[0], ctx),
            "args -> args [,] expr" => panic!(), // TODO: flatten args
            "args ->" => panic!(), // ????
            _ => unreachable!()
        }
    }
}

// TODO: build AST from Subtree

struct Tokenizer(lexers::Scanner<char>);

impl lexers::Nexter<String> for Tokenizer {
    fn get_item(&mut self) -> Option<String> {
        self.0.ignore_ws();
        lexers::scan_math_op(&mut self.0)
            .or_else(|| lexers::scan_number(&mut self.0))
            .or_else(|| lexers::scan_identifier(&mut self.0))
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
                let mut ctx = HashMap::new();
                println!("{:?}", seval(&tree, &mut ctx));
            },
            Err(e) => println!("Parse err: {:?}", e)
        }
    }
}

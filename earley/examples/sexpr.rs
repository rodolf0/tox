#![cfg(not(test))]

extern crate linenoise;
extern crate regex;
extern crate lexers;
extern crate toxearley as earley;

use earley::Subtree;
use std::collections::HashMap;

// assign -> id '=' expr | expr
// expr   -> expr '[+-]' term | term
// term   -> term '[*%/]' factor | factor
// factor -> '-' factor | power
// power  -> ufact '^' factor | ufact
// ufact  -> ufact '!' | group
// group  -> num | id | '(' expr ')' | func
// func   -> id '(' args ')'
// args   -> args ',' expr | expr | <e>

fn build_grammar() -> earley::Grammar {
    let num = regex::Regex::new(r"^-?\d+(?:\.\d+)?(?:[eE][-+]?\d+)?$").unwrap();
    let var = regex::Regex::new(r"^[A-Za-z_]+[A-Za-z0-9_]*$").unwrap();
    let mut gb = earley::GrammarBuilder::new();
    gb.symbol("assign")
      .symbol("expr")
      .symbol("term")
      .symbol("factor")
      .symbol("power")
      .symbol("ufact")
      .symbol("group")
      .symbol("func")
      .symbol("args")
      .symbol(("[n]", move |n: &str| num.is_match(n)))
      .symbol(("[v]", move |n: &str| var.is_match(n)))
      .symbol(("[+-]", |n: &str| n == "+" || n == "-"))
      .symbol(("[*/%]", |n: &str| n == "*" || n == "/" || n == "%"))
      .symbol(("[-]", |n: &str| n == "-"))
      .symbol(("[^]", |n: &str| n == "^"))
      .symbol(("[!]", |n: &str| n == "!"))
      .symbol(("[,]", |n: &str| n == ","))
      .symbol(("[(]", |n: &str| n == "("))
      .symbol(("[)]", |n: &str| n == ")"))
      .symbol(("[=]", |n: &str| n == "="));
    gb.rule("assign", vec!["expr"])
      .rule("assign", vec!["[v]", "[=]", "expr"])
      .rule("expr",   vec!["term"])
      .rule("expr",   vec!["expr", "[+-]", "term"])
      .rule("term",   vec!["factor"])
      .rule("term",   vec!["term", "[*/%]", "factor"])
      .rule("factor", vec!["power"])
      .rule("factor", vec!["[-]", "factor"])
      .rule("power",  vec!["ufact"])
      .rule("power",  vec!["ufact", "[^]", "factor"])
      .rule("ufact",  vec!["group"])
      .rule("ufact",  vec!["ufact", "[!]"])
      .rule("group",  vec!["[n]"])
      .rule("group",  vec!["[v]"])
      .rule("group",  vec!["[(]", "expr", "[)]"])
      .rule("group",  vec!["func"])
      .rule("func",   vec!["[v]", "[(]", "args", "[)]"])
      .rule("args",   vec!["expr"])
      .rule("args",   vec!["args", "[,]", "expr"])
      .rule("args",   vec![])
      ;
    gb.into_grammar("assign")
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

fn xeval(n: &Subtree, ctx: &mut HashMap<String, f64>) -> Vec<f64> {
    use std::str::FromStr;
    use std::f64::consts;

    macro_rules! eval0 {
        ($e:expr, $c:ident) => (xeval($e, $c)[0])
    }

    match n {
        &Subtree::Node(ref key, ref val) => match key.as_ref() {
            "[n]" => vec![f64::from_str(&val).unwrap()],
            "[v]" => match val.as_ref() {
                "e" => vec![consts::E],
                "pi" => vec![consts::PI],
                x => vec![ctx[x]]
            },
            _ => unreachable!()
        },
        &Subtree::SubT(ref key, ref subn) => match key.as_ref() {
            "assign -> expr" => xeval(&subn[0], ctx),
            "assign -> [v] [=] expr" => {
                let var = match &subn[0] {
                    &Subtree::Node(_, ref var) => var.clone(),
                    _ => unreachable!()
                };
                let val = xeval(&subn[2], ctx);
                ctx.insert(var, val[0]);
                val
            },
            "expr -> term" => xeval(&subn[0], ctx),
            "expr -> expr [+-] term" => match &subn[1] {
                &Subtree::Node(_, ref op) if op == "+" => vec![eval0!(&subn[0], ctx) + eval0!(&subn[2], ctx)],
                &Subtree::Node(_, ref op) if op == "-" => vec![eval0!(&subn[0], ctx) - eval0!(&subn[2], ctx)],
                _ => unreachable!()
            },
            "term -> factor" => xeval(&subn[0], ctx),
            "term -> term [*/%] factor" => match &subn[1] {
                &Subtree::Node(_, ref op) if op == "*" => vec![eval0!(&subn[0], ctx) * eval0!(&subn[2], ctx)],
                &Subtree::Node(_, ref op) if op == "/" => vec![eval0!(&subn[0], ctx) / eval0!(&subn[2], ctx)],
                &Subtree::Node(_, ref op) if op == "%" => vec![eval0!(&subn[0], ctx) % eval0!(&subn[2], ctx)],
                _ => unreachable!()
            },
            "factor -> power" => xeval(&subn[0], ctx),
            "factor -> [-] factor" => vec![- eval0!(&subn[1], ctx)],
            "power -> ufact" => xeval(&subn[0], ctx),
            "power -> ufact [^] factor" => match &subn[1] {
                &Subtree::Node(_, ref op) if op == "^" => vec![eval0!(&subn[0], ctx).powf(eval0!(&subn[2], ctx))],
                _ => unreachable!()
            },
            "ufact -> group" => xeval(&subn[0], ctx),
            "ufact -> ufact [!]" => panic!(), // no gamma function?
            "group -> [n]" => xeval(&subn[0], ctx),
            "group -> [v]" => xeval(&subn[0], ctx),
            "group -> [(] expr [)]" => xeval(&subn[1], ctx),
            "group -> func" => xeval(&subn[0], ctx),
            "func -> [v] [(] args [)]" => {
                let args = xeval(&subn[2], ctx);
                match &subn[0] {
                    &Subtree::Node(_, ref f) if f == "sin" => vec![args[0].sin()],
                    &Subtree::Node(_, ref f) if f == "cos" => vec![args[0].cos()],
                    &Subtree::Node(_, ref f) if f == "max" => match args.len() {
                        0 => vec![],
                        _ => vec![args.iter().cloned().fold(std::f64::NAN, f64::max)]
                    },
                    _ => panic!()
                }
            },
            "args -> expr" => xeval(&subn[0], ctx),
            "args -> args [,] expr" => {
                let mut a = xeval(&subn[0], ctx);
                a.push(eval0!(&subn[2], ctx));
                a
            }
            "args ->" => vec![],
            _ => unreachable!()
        }
    }
}

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

    let mut ctx = HashMap::new();
    while let Some(input) = linenoise::input("~> ") {
        linenoise::history_add(&input[..]);
        match parser.parse(&mut Tokenizer::from_str(&input)) {
            Ok(estate) => {
                let tree = earley::one_tree(parser.g.start(), &estate);
                let val = xeval(&tree, &mut ctx)[0];
                ctx.insert(format!["ans"], val);
                println!("{:?}", val);
            },
            Err(e) => println!("Parse err: {:?}", e)
        }
    }
}

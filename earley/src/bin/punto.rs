#![cfg(not(test))]

extern crate linenoise;
extern crate regex;
extern crate lexers;
extern crate earley;

use earley::Subtree;

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

fn main() {
    let parser = build_parser();
    while let Some(input) = linenoise::input("~> ") {
        linenoise::history_add(&input[..]);
        let mut input = lexers::DelimTokenizer::from_str(&input, " ", true);
        match parser.parse(&mut input) {
            Ok(estate) => {
                let tree = earley::build_tree(parser.g.start(), &estate);
                fn printer(node: &Subtree, n: usize) {
                    match node {
                        &Subtree::Node(ref term, ref value) => println!("\"{}. {}\" -> \"{}. {}\"", n, term, n + 1, value),
                        &Subtree::SubT(ref spec, ref childs) => for (nn, c) in childs.iter().enumerate() {
                            let x = match c {
                                &Subtree::Node(ref term, _) => term,
                                &Subtree::SubT(ref sspec, _) => sspec,
                            };
                            println!("\"{}. {}\" -> \"{}. {}\"", n, spec, n + nn + 100, x);
                            printer(&c, n + nn + 100);
                        }
                    }
                };

                println!("digraph arbol {{");
                printer(&tree.unwrap(), 0);
                println!("}}");
            },
            Err(e) => println!("Parse err: {:?}", e)
        }
    }
}

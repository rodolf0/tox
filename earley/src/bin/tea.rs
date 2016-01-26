extern crate linenoise;
extern crate regex;
extern crate lexers;
extern crate earley;

// Sum  -> Sum '+' Mul | Mul
// Mul  -> Mul '*' Pow | Pow | '-' Pow
// Pow  -> Fact '^' Pow | Fact
// Fact -> Num '!' | Num
// Num  -> 'num' | '(' Sum ')' | Fn '(' Args ')'
// Args -> Args ',' Sum | Sum
// Fn   -> 'string'

#[cfg(not(test))]
fn build_parser() -> earley::EarleyParser {
    use earley::Symbol;
    let num = regex::Regex::new(r"^-?\d+(?:\.\d+)?(?:[eE][-+]?\d+)?$").unwrap();
    let sss = regex::Regex::new(r"^[A-Za-z_]+[A-Za-z0-9_]*$").unwrap();
    let mut gb = earley::GrammarBuilder::new();
    gb.symbol(Symbol::nonterm("Sum"))
      .symbol(Symbol::nonterm("Mul"))
      .symbol(Symbol::nonterm("Pow"))
      .symbol(Symbol::nonterm("Fact"))
      .symbol(Symbol::nonterm("Num"))
      .symbol(Symbol::nonterm("Args"))
      .symbol(Symbol::terminal("[n]", move |n: &str| num.is_match(n)))
      .symbol(Symbol::terminal("[s]", move |n: &str| sss.is_match(n)))
      .symbol(Symbol::terminal("[+]", |n: &str| n == "+" || n == "-"))
      .symbol(Symbol::terminal("[*]", |n: &str| n == "*" || n == "/" || n == "%"))
      .symbol(Symbol::terminal("[-]", |n: &str| n == "-"))
      .symbol(Symbol::terminal("[^]", |n: &str| n == "^"))
      .symbol(Symbol::terminal("[!]", |n: &str| n == "!"))
      .symbol(Symbol::terminal("[,]", |n: &str| n == ","))
      .symbol(Symbol::terminal("[(]", |n: &str| n == "("))
      .symbol(Symbol::terminal("[)]", |n: &str| n == ")"));
    // add grammar rules
    gb.rule("Sum", vec!["Sum", "[+]", "Mul"]).rule("Sum", vec!["Mul"])
      .rule("Mul", vec!["Mul", "[*]", "Pow"]).rule("Mul", vec!["[-]", "Pow"]).rule("Mul", vec!["Pow"])
      .rule("Pow", vec!["Fact", "[^]", "Pow"]).rule("Pow", vec!["Fact"])
      .rule("Fact", vec!["Num", "[!]"]).rule("Fact", vec!["Num"])
      .rule("Num", vec!["[(]", "Sum", "[)]"]).rule("Num", vec!["[n]"])
      // TODO: add function calls
      ;
    earley::EarleyParser::new(gb.into_grammar("Sum"))
}

#[cfg(not(test))]
fn main() {
    let parser = build_parser();
    while let Some(input) = linenoise::input("~> ") {
        linenoise::history_add(&input[..]);
        // TODO: the tokenizer breaks exponent sign
        let mut input = lexers::DelimTokenizer::from_str(&input, "+-*/%^!(), ");
        if let Ok(pstate) = parser.parse(&mut input) {
            for t in earley::build_trees(&parser.g, &pstate) { println!("{:?}", t); }
        }
    }
}

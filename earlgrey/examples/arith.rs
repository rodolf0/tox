extern crate earlgrey;
extern crate lexers;
extern crate rustyline;
use std::cell::RefCell;

fn build_grammar() -> earlgrey::Grammar {
    use std::str::FromStr;;
    earlgrey::GrammarBuilder::new()
      .symbol("assign")
      .symbol("expr")
      .symbol("term")
      .symbol("factor")
      .symbol("power")
      .symbol("ufact")
      .symbol("group")
      .symbol("func")
      .symbol("args")
      .symbol(("[n]", |n: &str| f64::from_str(n).is_ok()))
      .symbol(("[v]", |n: &str| n.chars().all(|c| c.is_alphabetic() || c == '_')))
      .symbol(("+", |n: &str| n == "+"))
      .symbol(("-", |n: &str| n == "-"))
      .symbol(("*", |n: &str| n == "*"))
      .symbol(("/", |n: &str| n == "/"))
      .symbol(("%", |n: &str| n == "%"))
      .symbol(("^", |n: &str| n == "^"))
      .symbol(("!", |n: &str| n == "!"))
      .symbol((",", |n: &str| n == ","))
      .symbol(("(", |n: &str| n == "("))
      .symbol((")", |n: &str| n == ")"))
      .symbol(("=", |n: &str| n == "="))
      .rule("assign", &["expr"])
      //.rule("assign", &["[v]", "=", "expr"])
      .rule("expr",   &["term"])
      .rule("expr",   &["expr", "+", "term"])
      .rule("expr",   &["expr", "-", "term"])
      .rule("term",   &["factor"])
      .rule("term",   &["term", "*", "factor"])
      .rule("term",   &["term", "/", "factor"])
      .rule("term",   &["term", "%", "factor"])
      .rule("factor", &["power"])
      .rule("factor", &["-", "factor"])
      .rule("power",  &["ufact"])
      .rule("power",  &["ufact", "^", "factor"])
      .rule("ufact",  &["group"])
      .rule("ufact",  &["ufact", "!"])
      .rule("group",  &["[n]"])
      //.rule("group",  &["[v]"])
      .rule("group",  &["(", "expr", ")"])
      //.rule("group",  &["func"])
      //.rule("func",   &["[v]", "(", "args", ")"])
      //.rule("args",   &["expr"])
      //.rule("args",   &["args", ",", "expr"])
      //.rule::<_, &str>("args",   &[])
      .into_grammar("assign")
      .expect("Bad Gramar")
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

fn gamma(x: f64) -> f64 {
    #[link(name="m")]
    extern { fn tgamma(x: f64) -> f64; }
    unsafe { tgamma(x) }
}

fn semanter<'a>() -> earlgrey::EarleyEvaler<'a, f64> {
    use std::str::FromStr;
    let mut ev = earlgrey::EarleyEvaler::new(|symbol, token| {
        match symbol {"[n]" => f64::from_str(token).unwrap(), _ => 0.0}
    });
    ev.action("assign -> expr", |n| n[0]);
    //ev.action("assign -> [v] = expr", || {});
    ev.action("expr -> term", |n| n[0]);
    ev.action("expr -> expr + term", |n| n[0] + n[2]);
    ev.action("expr -> expr - term", |n| n[0] - n[2]);
    ev.action("term -> factor", |n| n[0]);
    ev.action("term -> term * factor", |n| n[0] * n[2]);
    ev.action("term -> term / factor", |n| n[0] / n[2]);
    ev.action("term -> term % factor", |n| n[0] % n[2]);
    ev.action("factor -> power", |n| n[0]);
    ev.action("factor -> - factor", |n| -n[1]);
    ev.action("power -> ufact", |n| n[0]);
    ev.action("power -> ufact ^ factor", |n| n[0].powf(n[2]));
    ev.action("ufact -> group", |n| n[0]);
    ev.action("ufact -> ufact !", |n| gamma(n[0]+1.0));
    ev.action("group -> [n]", |n| n[0]);
    //ev.action("group -> [v]", |n| {});
    ev.action("group -> ( expr )", |n| n[1]);
    //ev.action("group -> func", || {});
    //ev.action("func -> [v] ( args )", || {});
    //ev.action("args -> expr", || {});
    //ev.action("args -> args , expr", || {});
    //ev.action("args -> ", || {});
    ev
}

fn main() {
    let rl = RefCell::new(rustyline::Editor::<()>::new());
    let input: Box<Iterator<Item=_>> = if std::env::args().len() > 1 {
        Box::new((0..1).map(|_| std::env::args()
                            .skip(1).collect::<Vec<String>>().join(" ")))
    } else {
        Box::new((0..).map(|_| rl.borrow_mut().readline("~> "))
                 .take_while(|i| i.is_ok()).map(|i| i.unwrap()))
    };
    let parser = earlgrey::EarleyParser::new(build_grammar());
    let evaler = semanter();
    for expr in input {
        match parser.parse(&mut Tokenizer::from_str(&expr)) {
            Err(e) => println!("Parse err: {:?}", e),
            Ok(state) => {
                rl.borrow_mut().add_history_entry(&expr);
                let val = evaler.eval(&state);
                println!("{:?}", val);
            }
        }
    }
}

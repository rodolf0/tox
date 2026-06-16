mod grammar;
pub(crate) use grammar::{Grammar, GrammarBuilder};

mod parser;
mod spans;
pub(crate) use parser::EarleyParser;

mod trees;
pub(crate) use trees::EarleyForest;

#[cfg(test)]
mod parser_test;

#[cfg(test)]
mod minisum_test {
    use super::{EarleyForest, EarleyParser, GrammarBuilder};

    #[test]
    fn test_minisum() {
        let grammar = GrammarBuilder::default()
            .nonterm("S")
            .nonterm("N")
            .terminal("[+]", |c| c == "+")
            .terminal("[0-9]", |n| "1234567890".contains(n))
            .rule("S", &["S", "[+]", "N"])
            .rule("S", &["N"])
            .rule("N", &["[0-9]"])
            .into_grammar("S")
            .unwrap();

        let input = "1 + 2 + 3".split_whitespace();
        let trees = EarleyParser::new(grammar).parse(input).unwrap();

        let mut ev = EarleyForest::new(|symbol, token| match symbol {
            "[0-9]" => token.parse::<f64>().unwrap(),
            _ => 0.0,
        });
        ev.action("S -> S [+] N", |n| n[0] + n[2]);
        ev.action("S -> N", |n| n[0]);
        ev.action("N -> [0-9]", |n| n[0]);

        assert_eq!(ev.eval(&trees).unwrap(), 6.0);
    }
}

#[cfg(test)]
mod arith_test {
    use super::{EarleyForest, EarleyParser, Grammar, GrammarBuilder};
    use std::str::FromStr;

    fn build_grammar() -> Grammar {
        GrammarBuilder::default()
            .nonterm("expr")
            .nonterm("term")
            .nonterm("factor")
            .nonterm("power")
            .nonterm("ufact")
            .nonterm("group")
            .nonterm("func")
            .nonterm("args")
            .terminal("[n]", |n| f64::from_str(n).is_ok())
            .terminal("+", |n| n == "+")
            .terminal("-", |n| n == "-")
            .terminal("*", |n| n == "*")
            .terminal("/", |n| n == "/")
            .terminal("%", |n| n == "%")
            .terminal("^", |n| n == "^")
            .terminal("!", |n| n == "!")
            .terminal("(", |n| n == "(")
            .terminal(")", |n| n == ")")
            .rule("expr", &["term"])
            .rule("expr", &["expr", "+", "term"])
            .rule("expr", &["expr", "-", "term"])
            .rule("term", &["factor"])
            .rule("term", &["term", "*", "factor"])
            .rule("term", &["term", "/", "factor"])
            .rule("term", &["term", "%", "factor"])
            .rule("factor", &["power"])
            .rule("factor", &["-", "factor"])
            .rule("power", &["ufact"])
            .rule("power", &["ufact", "^", "factor"])
            .rule("ufact", &["group"])
            .rule("ufact", &["ufact", "!"])
            .rule("group", &["[n]"])
            .rule("group", &["(", "expr", ")"])
            .into_grammar("expr")
            .expect("Bad Gramar")
    }

    fn gamma(x: f64) -> f64 {
        #[link(name = "m")]
        unsafe extern "C" {
            fn tgamma(x: f64) -> f64;
        }
        unsafe { tgamma(x) }
    }

    fn semanter<'a>() -> EarleyForest<'a, f64> {
        let mut ev = EarleyForest::new(|symbol, token| match symbol {
            "[n]" => f64::from_str(token).unwrap(),
            _ => 0.0,
        });
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
        ev.action("ufact -> ufact !", |n| gamma(n[0] + 1.0));
        ev.action("group -> [n]", |n| n[0]);
        ev.action("group -> ( expr )", |n| n[1]);
        ev
    }

    #[test]
    fn test_arith() {
        let parser = EarleyParser::new(build_grammar());
        let ev = semanter();
        let symbols = ["+", "-", "*", "/", "%", "^", "!", "(", ")"];

        let state = parser
            .parse(lexers::StringTokenizer::from("1 + 2 * 3").split_on(symbols, false))
            .unwrap();
        assert_eq!(ev.eval(&state).unwrap(), 7.0);

        let state = parser
            .parse(lexers::StringTokenizer::from("( 1 + 2 ) * 3").split_on(symbols, false))
            .unwrap();
        assert_eq!(ev.eval(&state).unwrap(), 9.0);
    }
}

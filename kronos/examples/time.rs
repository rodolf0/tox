extern crate toxearley as earley;
extern crate lexers;
extern crate kronos;
extern crate chrono;

use chrono::naive::datetime::NaiveDateTime as DateTime;
use kronos::constants as k;
use std::str::FromStr;

// https://github.com/wit-ai/duckling/blob/master/resources/languages/en/rules/time.clj
fn build_grammar() -> earley::Grammar {
    static STOP_WORDS: &'static [&'static str] = &[
        "this", "next", "the", "last", "before", "after", "of", "on",
        "weekend", "summer", "autumn", "spring", "winter",
    ];
    let mut gb = earley::GrammarBuilder::new();
    for sw in STOP_WORDS { gb = gb.symbol((*sw, move |n: &str| n == *sw)); }

    gb.symbol("<time>")
      .symbol(("<day-of-week>", |d: &str| k::weekday(d).is_some()))
      .symbol(("<named-month>", |m: &str| k::month(m).is_some()))
      .symbol(("<ordinal>", |n: &str| k::ordinal(n).or(k::short_ordinal(n)).is_some()))
      .symbol(("<number>", |n: &str| i32::from_str(n).is_ok()))

      .rule("<time>", &["the", "<ordinal>"])                 // the 2nd
      .rule("<time>", &["<day-of-week>"])                    // thursday
      .rule("<time>", &["<named-month>"])                    // march
      .rule("<time>", &["weekend"])                          // weekend
      .rule("<time>", &["<named-month>", "<ordinal>"])
      .rule("<time>", &["<named-month>", "<number>"])
      .rule("<time>", &["<day-of-week>", "<ordinal>"])
      .rule("<time>", &["<day-of-week>", "<number>"])

      .rule("<time>", &["<time>", "<time>"])                 // intersect 2 times
      .rule("<time>", &["<ordinal>", "<time>", "of", "<time>"])
      .rule("<time>", &["the", "<ordinal>", "<time>", "of", "<time>"])

      // TODO: requires Seq-interval
      //.rule("<time>", &["summer"])
      //.rule("<time>", &["spring"])
      //.rule("<time>", &["autumn"])
      //.rule("<time>", &["winter"])

      //.rule("<grain/range>", &["weeks"]) | "days" ...
      //.rule("<time>", &["in" , "<number>", "<grain/range>"])

      //.rule("<time>", &["last", "<time>"])                   // last week | last sunday | last friday
      //.rule("<time>", &["<time>", "before", "last"])
      //.rule("<time>", &["<time>", "after", "next"])
      //.rule("<time>", &["<ordinal>", "<time>", "after", "<time>"])

      .into_grammar("<time>")
}


pub enum Tobj {
    //Duration(Duration),
    Seq(kronos::Seq),
    Range(kronos::Range),
    Num(i32),
}

macro_rules! xtract {
    ($p:path, $e:expr) => (match $e {$p(x) => x, _ => panic!()})
}

pub fn eval(reftime: DateTime, n: &earley::Subtree) -> Tobj {
    match n {
        &earley::Subtree::Node(ref sym, ref lexeme) => match sym.as_ref() {
            "<day-of-week>" => {
                let dow = k::weekday(lexeme).unwrap();
                Tobj::Seq(kronos::day_of_week(dow))
            },
            "<named-month>" => {
                let month = k::month(lexeme).unwrap();
                Tobj::Seq(kronos::month_of_year(month))
            },
            "<ordinal>" => {
                let num = k::ordinal(lexeme).or(k::short_ordinal(lexeme)).unwrap();
                Tobj::Num(num as i32)
            },
            "<number>" => Tobj::Num(i32::from_str(lexeme).unwrap()),
            _ => panic!("Unknown sym={:?} lexeme={:?}", sym, lexeme)
        },
        &earley::Subtree::SubT(ref spec, ref subn) => match spec.as_ref() {
            "<time> -> the <ordinal>" => {
                let n = xtract!(Tobj::Num, eval(reftime, &subn[1])) as usize;
                Tobj::Seq(kronos::nth(n, kronos::day(), kronos::month()))
            },
            "<time> -> <day-of-week>" |
            "<time> -> <named-month>" => eval(reftime, &subn[0]),
            "<time> -> <named-month> <ordinal>" |
            "<time> -> <named-month> <number>" |
            "<time> -> <day-of-week> <ordinal>" |
            "<time> -> <day-of-week> <number>" => {
                let m = xtract!(Tobj::Seq, eval(reftime, &subn[0]));
                let d = xtract!(Tobj::Num, eval(reftime, &subn[1])) as usize;
                Tobj::Seq(kronos::intersect(m, kronos::nth(d, kronos::day(), kronos::month())))
            },
            "<time> -> weekend" => Tobj::Seq(kronos::weekend()),
            "<time> -> <time> <time>" => {
                let s1 = xtract!(Tobj::Seq, eval(reftime, &subn[0]));
                let s2 = xtract!(Tobj::Seq, eval(reftime, &subn[1]));
                Tobj::Seq(kronos::intersect(s1, s2))
            },
            "<time> -> <ordinal> <time> of <time>" => {
                let n = xtract!(Tobj::Num, eval(reftime, &subn[0])) as usize;
                let s1 = xtract!(Tobj::Seq, eval(reftime, &subn[1]));
                let s2 = xtract!(Tobj::Seq, eval(reftime, &subn[3]));
                Tobj::Seq(kronos::nth(n, s1, s2))
            },
            _ => panic!()
        }
    }
}

fn main() {
    if std::env::args().len() < 1 {
        println!("usage: time <time-expr>");
        return;
    }

    let input = std::env::args().skip(1).collect::<Vec<String>>().join(" ");
    let parser = earley::EarleyParser::new(build_grammar());
    let mut tokenizer = lexers::DelimTokenizer::from_str(&input, " ", true);

    match parser.parse(&mut tokenizer) {
        Ok(state) => for tree in earley::all_trees(parser.g.start(), &state) {
            let reftime = chrono::Local::now().naive_local();
            let r = xtract!(Tobj::Seq, eval(reftime, &tree));
            // TODO: fuse after max-n elements
            println!("{:?}", r(reftime).next().unwrap());
        },
        Err(e) => println!("Parse err: {:?}", e)
    }
}
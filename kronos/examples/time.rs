extern crate toxearley as earley;
extern crate lexers;
extern crate kronos;
extern crate chrono;

use chrono::naive::datetime::NaiveDateTime as DateTime;
use kronos::constants as k;

// https://github.com/wit-ai/duckling/blob/master/resources/languages/en/rules/time.clj
fn build_grammar() -> earley::Grammar {
    earley::GrammarBuilder::new()
      .symbol("<time>")
      .symbol(("<day-of-week>", |d: &str| k::weekday(d).is_some()))
      .symbol(("<ordinal>", |n: &str| k::ordinal(n).or(k::short_ordinal(n)).is_some()))
      //.symbol(("<named-month>", |m: &str| month(m).is_some()))

      //.symbol(("this", |n: &str| n == "this"))
      //.symbol(("next", |n: &str| n == "next"))
      .symbol(("the", |n: &str| n == "the"))
      //.symbol(("last", |n: &str| n == "last"))
      //.symbol(("before", |n: &str| n == "before"))
      //.symbol(("after", |n: &str| n == "after"))
      //.symbol(("of", |n: &str| n == "of"))
      //.symbol(("now", |n: &str| n == "now"))
      //.symbol(("today", |n: &str| n == "today"))
      //.symbol(("tomorrow", |n: &str| n == "tomorrow"))
      //.symbol(("yesterday", |n: &str| n == "yesterday"))
      //.symbol(("year", |n: &str| n == "year"))

      .rule("<time>", &["<day-of-week>"])                    // thursday
      .rule("<time>", &["the", "<ordinal>"])                 // the 2nd
      //.rule("<time>", &["<time>", "<time>"])        // intersect 2 times
      //.rule("<time>", &["<named-month>"])                    // march
      //.rule("<time>", &["year"])                             // march
      //.rule("<time>", &["this", "<day-of-week>"])            // next tuesday
      //.rule("<time>", &["next", "<day-of-week>"])            // next tuesday
      //.rule("<time>", &["last", "<time>"])                   // last week | last sunday | last friday
      //.rule("<time>", &["next", "<time>"])                   // last week | last sunday | last friday
      //.rule("<time>", &["<named-month>", "<ordinal>"])
      //.rule("<time>", &["<ordinal>", "<time>", "of", "<time>"])
      //.rule("<time>", &["<time>", "before", "last"])
      //.rule("<time>", &["<time>", "after", "next"])
      //.rule("<time>", &["<ordinal>", "<time>", "after", "<time>"])
      //.rule("<time>", &["<ordinal>", "<time>", "of", "<time>"])
      //.rule("<time>", &["the", "<ordinal>", "<time>", "of", "<time>"])

      .into_grammar("<time>")
}


//#[derive(Debug)]
pub enum Tobj {
    //Duration(Duration),
    Seq(kronos::Seq),
    Range(kronos::Range),
    Num(i32),
}

pub fn eval(reftime: DateTime, n: &earley::Subtree) -> Tobj {
    match n {
        &earley::Subtree::Node(ref sym, ref lexeme) => match sym.as_ref() {
            "<day-of-week>" => {
                let dow = k::weekday(lexeme).unwrap();
                Tobj::Seq(kronos::day_of_week(dow))
            },
            "<ordinal>" => {
                let num = k::ordinal(lexeme).or(k::short_ordinal(lexeme)).unwrap();
                Tobj::Num(num as i32)
            },
            //"<named-month>" => {
                //let month = month(lexeme).unwrap();
                //Tobj::Seq(seq_month_of_year(month))
            //},
            _ => panic!()
        },
        &earley::Subtree::SubT(ref spec, ref subn) => match spec.as_ref() {
            "<time> -> <day-of-week>" => {
                // TODO: create macro to wrap all this crap
                match eval(reftime, &subn[0]) {
                    Tobj::Seq(s) => Tobj::Range(s(reftime).next().unwrap()),
                    _ => panic!(),
                }
            },
            "<time> -> the <ordinal>" => {
                let n = match eval(reftime, &subn[1]) {
                    Tobj::Num(n) => n,
                    _ => panic!(),
                } as usize;
                let s = kronos::nth(n, kronos::day(), kronos::month());
                Tobj::Range(s(reftime).next().unwrap())
            },
            //"<time> -> <named-month> <ordinal>" => {
                //let m = match eval(ctx, &subn[0]) {
                    //Tobj::Seq(s) => s,
                    //_ => panic!(),
                //};
                //let d = match eval(ctx, &subn[1]) {
                    //Tobj::Num(n) => n,
                    //_ => panic!(),
                //};
                //Tobj::Seq(intersect(m, seq_nth(d as usize, seq_day(), seq_month())))
            //},
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
            match eval(reftime, &tree) {
                Tobj::Range(r) => {
                    println!("{:?}", r);
                },
                _ => panic!()
            }
        },
        Err(e) => println!("Parse err: {:?}", e)
    }
}

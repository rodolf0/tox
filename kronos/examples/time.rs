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
        "the", "of", "a", "next", "this", "after", "weekend", "in", "to",
    ];
    let mut gb = earley::GrammarBuilder::new();
    for sw in STOP_WORDS { gb = gb.symbol((*sw, move |n: &str| n == *sw)); }

    gb.symbol(("<ordinal>", |n: &str| k::ordinal(n).or(k::short_ordinal(n)).is_some()))
      .symbol(("<number>", |n: &str| i32::from_str(n).is_ok()))

      //// Durations
      .symbol("<duration>")
      .symbol(("<dur-day>", |d: &str| d == "day" || d == "days"))
      .symbol(("<dur-week>", |d: &str| d == "week" || d == "weeks"))
      .symbol(("<dur-month>", |d: &str| d == "month" || d == "months"))
      .symbol(("<dur-quarter>", |d: &str| d == "quarter" || d == "quarters"))
      .symbol(("<dur-year>", |d: &str| d == "year" || d == "years"))
      .rule("<duration>", &["<dur-day>"])
      .rule("<duration>", &["<dur-week>"])
      .rule("<duration>", &["<dur-month>"])
      .rule("<duration>", &["<dur-quarter>"])
      .rule("<duration>", &["<dur-year>"])
      .rule("<duration>", &["a", "<duration>"]) // a week
      .rule("<duration>", &["<number>", "<duration>"]) // 2 days

      //// Sequences
      .symbol("<seq>")
      .symbol(("<day-of-week>", |d: &str| k::weekday(d).is_some()))
      .symbol(("<named-month>", |m: &str| k::month(m).is_some()))
      // basic sequences
      .rule("<seq>", &["<duration>"])
      .rule("<seq>", &["<named-month>"])
      .rule("<seq>", &["<day-of-week>"])
      .rule("<seq>", &["the", "<ordinal>"]) // (day of the month)
      .rule("<seq>", &["weekend"])
      // intersections
      .rule("<seq>", &["<named-month>", "<ordinal>"])
      .rule("<seq>", &["<named-month>", "<number>"])
      .rule("<seq>", &["<day-of-week>", "<ordinal>"])
      .rule("<seq>", &["<day-of-week>", "<number>"])
      .rule("<seq>", &["<seq>", "<seq>"])
      // nthofs
      .rule("<seq>", &["<ordinal>", "<seq>", "of", "<seq>"])
      .rule("<seq>", &["<seq>", "to", "<seq>"])

      //// Ranges
      .symbol("<time>")
      .rule("<time>", &["<number>"]) // year
      .rule("<time>", &["<named-month>", "<number>"]) // july 1994
      .rule("<time>", &["<seq>"]) // grab first item of seq
      .rule("<time>", &["this", "<seq>"])
      .rule("<time>", &["next", "<seq>"])
      .rule("<time>", &["next", "<number>", "<seq>"]) // next 3 weeks
      .rule("<time>", &["<seq>", "after", "next"])
      // ranges shifted by duration
      .rule("<time>", &["<duration>", "after", "<time>"]) // 2 days after xx
      .rule("<time>", &["in", "<duration>"])  // in a week, in 6 days

      // TODO
      // * the last week of november
      // * in 3 days / 3 days from now
      // * in 2 months (feb?) (when its 13th of june)
      //
      //.rule("<time>", &["last", "<time>"])                   // last week | last sunday | last friday
      //.rule("<time>", &["<time>", "before", "last"])


      .into_grammar("<time>")
}


// TODO: replace this and xtract macro with 3 functions directly
pub enum Tobj {
    //Duration(Duration),
    Seq(kronos::Seq),
    Range(kronos::Range),
    Num(i32),
}

macro_rules! xtract {
    ($p:path, $e:expr) => (match $e {$p(x) => x, _ => panic!()})
}

fn eval_terminal(n: &earley::Subtree) -> Tobj {
    if let &earley::Subtree::Node(ref sym, ref lexeme) = n {
        match sym.as_ref() {
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
        }
    } else {
        panic!("Couldn't evaluate terminal {:?}", n);
    }
}

fn duration_to_seq(reftime: DateTime, n: &earley::Subtree) -> kronos::Seq {
    if let &earley::Subtree::SubT(ref spec, ref subn) = n {
        println!("* {:?}", spec); // trace
        match spec.as_ref() {
            "<duration> -> <dur-day>" => kronos::day(),
            "<duration> -> <dur-week>" => kronos::week(),
            "<duration> -> <dur-month>" => kronos::month(),
            "<duration> -> <dur-quarter>" => kronos::quarter(),
            "<duration> -> <dur-year>" => kronos::year(),
            "<duration> -> a <duration>" => duration_to_seq(reftime, &subn[1]),
            "<duration> -> <number> <duration>" => {
                let n = xtract!(Tobj::Num, eval_terminal(&subn[0])) as usize;
                kronos::merge(n, duration_to_seq(reftime, &subn[1]))
            },
            _ => panic!("Unknown duration rule={:?}", spec)
        }
    } else {
        unreachable!("what!!")
    }
}

pub fn eval_seq(reftime: DateTime, n: &earley::Subtree) -> kronos::Seq {
    if let &earley::Subtree::SubT(ref spec, ref subn) = n {
        println!("* {:?}", spec); // trace
        match spec.as_ref() {
            "<seq> -> <duration>" => duration_to_seq(reftime, &subn[0]),
            "<seq> -> <named-month>" |
            "<seq> -> <day-of-week>" => xtract!(Tobj::Seq, eval_terminal(&subn[0])),
            "<seq> -> the <ordinal>" => {
                let n = xtract!(Tobj::Num, eval_terminal(&subn[1])) as usize;
                kronos::nthof(n, kronos::day(), kronos::month())
            },
            "<seq> -> weekend" => kronos::weekend(),
            "<seq> -> <named-month> <ordinal>" |
            "<seq> -> <named-month> <number>" |
            "<seq> -> <day-of-week> <ordinal>" |
            "<seq> -> <day-of-week> <number>" => {
                let m = xtract!(Tobj::Seq, eval_terminal(&subn[0]));
                let d = xtract!(Tobj::Num, eval_terminal(&subn[1])) as usize;
                kronos::intersect(m, kronos::nthof(d, kronos::day(), kronos::month()))
            },
            "<seq> -> <seq> <seq>" => {
                kronos::intersect(eval_seq(reftime, &subn[0]), eval_seq(reftime, &subn[1]))
            },
            "<seq> -> <ordinal> <seq> of <seq>" => {
                let n = xtract!(Tobj::Num, eval_terminal(&subn[0])) as usize;
                kronos::nthof(n, eval_seq(reftime, &subn[1]), eval_seq(reftime, &subn[3]))
            },
            "<seq> -> <seq> to <seq>" => {
                kronos::interval(eval_seq(reftime, &subn[0]), eval_seq(reftime, &subn[2]))
            },
            _ => panic!("Unknown [eval_seq] spec={:?}", spec)
        }
    } else {
        unreachable!("what!");
    }
}


pub fn eval(reftime: DateTime, n: &earley::Subtree) -> kronos::Range {
    if let &earley::Subtree::SubT(ref spec, ref subn) = n {
        println!("* {:?}", spec);
        match spec.as_ref() {
            "<time> -> <number>" => {
                let n = xtract!(Tobj::Num, eval_terminal(&subn[0])) as usize;
                kronos::a_year(n)
            },
            "<time> -> <named-month> <number>" => {
                let month = xtract!(Tobj::Seq, eval_terminal(&subn[0]));
                let n = xtract!(Tobj::Num, eval_terminal(&subn[1])) as usize;
                kronos::this(month, kronos::a_year(n).start)
            },
            "<time> -> <seq>" => {
                kronos::this(eval_seq(reftime, &subn[0]), reftime)
            },
            "<time> -> this <seq>" => {
                kronos::this(eval_seq(reftime, &subn[1]), reftime)
            },
            "<time> -> next <seq>" => {
                kronos::next(eval_seq(reftime, &subn[1]), 1, reftime)
            },
            "<time> -> next <number> <seq>" => {
                let n = xtract!(Tobj::Num, eval_terminal(&subn[1])) as usize;
                let r0 = kronos::next(eval_seq(reftime, &subn[2]), 1, reftime);
                let r1 = kronos::next(eval_seq(reftime, &subn[2]), n, reftime);
                kronos::Range{start: r0.start, end: r1.end, grain: r0.grain}
            },
            "<time> -> <seq> after next" => {
                kronos::next(eval_seq(reftime, &subn[0]), 2, reftime)
            },
            //"<time> -> in <duration>" => {
            //},
            _ => panic!("Unknown [eval] spec={:?}", spec)
        }
    } else {
        unreachable!("what!");
    }
}

fn main() {
    if std::env::args().len() < 1 {
        println!("usage: time <time-expr>");
        return;
    }

    let input = std::env::args().skip(1).collect::<Vec<String>>().join(" ");
    let parser = earley::EarleyParser::new(build_grammar());
    let mut tokenizer = lexers::DelimTokenizer::from_str(&input, ", ", true);

    match parser.parse(&mut tokenizer) {
        Ok(state) => for tree in earley::all_trees(parser.g.start(), &state) {
            let reftime = chrono::Local::now().naive_local();
            println!("{:?}", eval(reftime, &tree));
        },
        Err(e) => println!("Parse err: {:?}", e)
    }
}

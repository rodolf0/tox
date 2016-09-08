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
        "of", "a", "next", "this", "after",
        "weekend", "in", "to", "ago", "last",
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
      .rule("<seq>", &["<number>"]) // year
      .rule("<seq>", &["<named-month>"])
      .rule("<seq>", &["<day-of-week>"])
      .rule("<seq>", &["<ordinal>"]) // (day of the month)
      .rule("<seq>", &["<duration>"])
      .rule("<seq>", &["weekend"])
      // intersection, nthofs, interval
      .rule("<seq>", &["<seq>", "<seq>"])
      .rule("<seq>", &["<ordinal>", "<seq>", "of", "<seq>"])
      .rule("<seq>", &["last", "<seq>", "of", "<seq>"])
      .rule("<seq>", &["<seq>", "to", "<seq>"])

      //// Ranges
      .symbol("<time>")
      .rule("<time>", &["<seq>"]) // 1st range of seq
      .rule("<time>", &["this", "<seq>"])
      .rule("<time>", &["next", "<seq>"])
      .rule("<time>", &["next", "<number>", "<seq>"]) // next 3 weeks
      .rule("<time>", &["<seq>", "after", "next"])
      // ranges shifted by duration
      .rule("<time>", &["<duration>", "after", "<time>"]) // 2 days after xx
      .rule("<time>", &["in", "<duration>"])  // in a week, in 6 days
      .rule("<time>", &["<duration>", "ago"])  // 3 months ago

      // TODO
      // * the last week of november
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

fn duration_to_grain(n: &earley::Subtree) -> (kronos::Granularity, i32) {
    if let &earley::Subtree::SubT(ref spec, ref subn) = n {
        println!("* {:?}", spec); // trace
        match spec.as_ref() {
            "<duration> -> <dur-day>" => (kronos::Granularity::Day, 1),
            "<duration> -> <dur-week>" => (kronos::Granularity::Week, 1),
            "<duration> -> <dur-month>" => (kronos::Granularity::Month, 1),
            "<duration> -> <dur-quarter>" => (kronos::Granularity::Quarter, 1),
            "<duration> -> <dur-year>" => (kronos::Granularity::Year, 1),
            "<duration> -> a <duration>" => duration_to_grain(&subn[1]),
            "<duration> -> <number> <duration>" => {
                let n = xtract!(Tobj::Num, eval_terminal(&subn[0]));
                let (g, n2) = duration_to_grain(&subn[1]);
                (g, n * n2)
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
            "<seq> -> <ordinal>" => {
                let n = xtract!(Tobj::Num, eval_terminal(&subn[0])) as usize;
                kronos::nthof(n, kronos::day(), kronos::month())
            },
            "<seq> -> weekend" => kronos::weekend(),
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
            "<seq> -> last <seq> of <seq>" => {
                kronos::lastof(1, eval_seq(reftime, &subn[1]), eval_seq(reftime, &subn[3]))
            },
            "<seq> -> <number>" => {
                let n = xtract!(Tobj::Num, eval_terminal(&subn[0])) as usize;
                kronos::a_year(n)
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
            "<time> -> <duration> after <time>" => {
                let t = eval(reftime, &subn[2]);
                let (g, n) = duration_to_grain(&subn[0]);
                kronos::shift(t, n, g)
            },
            "<time> -> in <duration>" => {
                let r = kronos::Range{
                    start: reftime,
                    end: reftime + chrono::Duration::days(1),
                    grain: kronos::Granularity::Day
                };
                let (g, n) = duration_to_grain(&subn[1]);
                kronos::shift(r, n, g)
            },
            "<time> -> <duration> ago" => {
                let r = kronos::Range{
                    start: reftime,
                    end: reftime + chrono::Duration::days(1),
                    grain: kronos::Granularity::Day
                };
                let (g, n) = duration_to_grain(&subn[0]);
                kronos::shift(r, -n, g)
            },
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

#[cfg(test)]
mod tests {
    use chrono::naive::date::NaiveDate as Date;
    #[test]
    fn test_time_1() {
        let s = "july 2015";
        let s = "june 2014";
        let s = "last feb of 2013";
        let s = "july 23rd";
        let s = "1st thu of the month"; // TODO
        let s = "2nd month of 2012";
        let s = "3 days after mon feb 28th";
        let s = "feb next year";
    }
}

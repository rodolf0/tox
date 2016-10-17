extern crate earlgrey as earley;
extern crate lexers;
extern crate kronos;
extern crate chrono;

use chrono::naive::datetime::NaiveDateTime as DateTime;
use kronos::constants as k;
use std::str::FromStr;

fn build_grammar() -> earley::Grammar {
    // more terminals
    static STOP_WORDS: &'static [&'static str] = &[
        "the", "of", "a", "next", "this", "after",
        "weekend", "in", "to", "ago", "last",
    ];
    let mut gb = earley::GrammarBuilder::new();
    for sw in STOP_WORDS { gb = gb.symbol((*sw, move |n: &str| n == *sw)); }

    // terminals
    gb.symbol(("<ordinal>", |n: &str| k::ordinal(n).or(k::short_ordinal(n)).is_some()))
      .symbol(("<number>", |n: &str| i32::from_str(n).is_ok()))
      .symbol(("<day-of-week>", |d: &str| k::weekday(d).is_some()))
      .symbol(("<named-month>", |m: &str| k::month(m).is_some()))
      .symbol(("<day-of-month>", |n: &str| k::ordinal(n).or(k::short_ordinal(n)).is_some())) // also number ?
      .symbol(("<dur-day>", |d: &str| d == "day" || d == "days"))
      .symbol(("<dur-week>", |d: &str| d == "week" || d == "weeks"))
      .symbol(("<dur-month>", |d: &str| d == "month" || d == "months"))
      .symbol(("<dur-quarter>", |d: &str| d == "quarter" || d == "quarters"))
      .symbol(("<dur-year>", |d: &str| d == "year" || d == "years"))

      .symbol("<base_duration>")
      .rule("<base_duration>", &["<dur-day>"])
      .rule("<base_duration>", &["<dur-week>"])
      .rule("<base_duration>", &["<dur-month>"])
      .rule("<base_duration>", &["<dur-quarter>"])
      .rule("<base_duration>", &["<dur-year>"])
      .symbol("<duration>")
      .rule("<duration>", &["<base_duration>"]) // day
      .rule("<duration>", &["a", "<base_duration>"]) // a week
      .rule("<duration>", &["<number>", "<base_duration>"]) // 2 days

      .symbol("<base_seq>")
      .rule("<base_seq>", &["<named-month>"]) // may, june, ...
      .rule("<base_seq>", &["<day-of-week>"]) // monday, friday, ...
      .rule("<base_seq>", &["<day-of-month>"]) // 23rd, 5th, 1st, ... day of month
      .rule("<base_seq>", &["weekend"]) // seq of weekends (TODO: quarters)
      .rule("<base_seq>", &["<duration>"]) // days, weeks, months, ... 2 weeks (of june)

      .symbol("<seq>")
      .rule("<seq>", &["<ordinal>", "<base_seq>", "of", "the", "<seq>"]) // 2nd day of the 3rd week
      .rule("<seq>", &["<ordinal>", "<base_seq>", "of", "<seq>"]) // 3rd hour of june 18th
      .rule("<seq>", &["last", "<base_seq>", "of", "the", "<seq>"])
      .rule("<seq>", &["last", "<base_seq>", "of", "<seq>"])
      .rule("<seq>", &["<base_seq>", "of", "<seq>"]) // mondays of june (all, every year)
      .rule("<seq>", &["<base_seq>", "<seq>"]) // may 21st, thursday may 21st
      .rule("<seq>", &["<seq>", "to", "<seq>"]) // mon to fri, feb to jun EXPERIMENTAL
      .rule("<seq>", &["<base_seq>"])

      .symbol("<time>") // Ranges: things that resolve to a range
      .rule("<time>", &["<number>"]) // year
      .rule("<time>", &["this", "<seq>"]) // this monday
      .rule("<time>", &["next", "<seq>"]) // next monday
      .rule("<time>", &["<seq>"]) // 1st range of seq evaluated now
      .rule("<time>", &["<seq>", "after", "next"])
      // seqs anchored to times
      .rule("<time>", &["<seq>", "<time>"]) // EVAL SEQ ON <TIME>
      .rule("<time>", &["<seq>", "<number>"]) // tied to a year
      .rule("<time>", &["<seq>", "after", "<time>"]) // 2 days after monday 28th (could be a seq)
      .rule("<time>", &["<ordinal>", "<seq>", "of", "<number>"]) // tied to a year
      .rule("<time>", &["last", "<seq>", "of", "<number>"]) // tied to a year
      // ranges shifted by duration
      .rule("<time>", &["in", "<duration>"])  // in a week, in 6 days
      .rule("<time>", &["<duration>", "ago"])  // 3 months ago
      .rule("<time>", &["<duration>", "after", "<time>"]) // same but different than <seq> ...

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
    if let &earley::Subtree::Leaf(ref sym, ref lexeme) = n {
        match sym.as_ref() {
            "<day-of-week>" => {
                let dow = k::weekday(lexeme).unwrap();
                Tobj::Seq(kronos::day_of_week(dow))
            },
            "<named-month>" => {
                let month = k::month(lexeme).unwrap();
                Tobj::Seq(kronos::month_of_year(month))
            },
            "<day-of-month>" |
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
    if let &earley::Subtree::Node(ref spec, ref subn) = n {
        println!("* {:?}", spec); // trace
        match spec.as_ref() {
            "<base_duration> -> <dur-day>" => kronos::day(),
            "<base_duration> -> <dur-week>" => kronos::week(),
            "<base_duration> -> <dur-month>" => kronos::month(),
            "<base_duration> -> <dur-quarter>" => kronos::quarter(),
            "<base_duration> -> <dur-year>" => kronos::year(),
            "<duration> -> <base_duration>" => duration_to_seq(reftime, &subn[0]),
            "<duration> -> a <base_duration>" => duration_to_seq(reftime, &subn[1]),
            "<duration> -> <number> <base_duration>" => {
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
    if let &earley::Subtree::Node(ref spec, ref subn) = n {
        println!("* {:?}", spec); // trace
        match spec.as_ref() {
            "<base_duration> -> <dur-day>" => (kronos::Granularity::Day, 1),
            "<base_duration> -> <dur-week>" => (kronos::Granularity::Week, 1),
            "<base_duration> -> <dur-month>" => (kronos::Granularity::Month, 1),
            "<base_duration> -> <dur-quarter>" => (kronos::Granularity::Quarter, 1),
            "<base_duration> -> <dur-year>" => (kronos::Granularity::Year, 1),
            "<duration> -> <base_duration>" => duration_to_grain(&subn[0]),
            "<duration> -> a <base_duration>" => duration_to_grain(&subn[1]),
            "<duration> -> <number> <base_duration>" => {
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
    if let &earley::Subtree::Node(ref spec, ref subn) = n {
        println!("* {:?} ==> {:?}", spec,
                 subn.iter().map(|i| {
                     match i {
                         &earley::Subtree::Leaf(_, ref n) => n.to_string(),
                         &earley::Subtree::Node(ref n, _) => n.to_string(),
                     }
                 }).collect::<Vec<_>>().join(" | "));
        match spec.as_ref() {
            "<base_seq> -> <named-month>" |
            "<base_seq> -> <day-of-week>" => xtract!(Tobj::Seq, eval_terminal(&subn[0])),
            "<base_seq> -> <day-of-month>" => {
                let n = xtract!(Tobj::Num, eval_terminal(&subn[0])) as usize;
                kronos::nthof(n, kronos::day(), kronos::month())
            },
            "<base_seq> -> weekend" => kronos::weekend(),
            "<base_seq> -> <duration>" => duration_to_seq(reftime, &subn[0]),
            "<seq> -> <base_seq>" => eval_seq(reftime, &subn[0]),
            ////////////////////////////////////////////////////////////////////////////
            "<seq> -> <ordinal> <base_seq> of the <seq>" => {
                let n = xtract!(Tobj::Num, eval_terminal(&subn[0])) as usize;
                kronos::nthof(n, eval_seq(reftime, &subn[1]), eval_seq(reftime, &subn[4]))
            },
            "<seq> -> <ordinal> <base_seq> of <seq>" => {
                let n = xtract!(Tobj::Num, eval_terminal(&subn[0])) as usize;
                kronos::nthof(n, eval_seq(reftime, &subn[1]), eval_seq(reftime, &subn[3]))
            },
            "<seq> -> last <base_seq> of the <seq>" =>
                kronos::lastof(1, eval_seq(reftime, &subn[1]), eval_seq(reftime, &subn[4])),
            "<seq> -> last <base_seq> of <seq>" =>
                kronos::lastof(1, eval_seq(reftime, &subn[1]), eval_seq(reftime, &subn[3])),
            ////////////////////////////////////////////////////////////////////////////
            "<seq> -> <base_seq> of <seq>" =>
                kronos::intersect(eval_seq(reftime, &subn[0]), eval_seq(reftime, &subn[2])),
            "<seq> -> <base_seq> <seq>" =>
                kronos::intersect(eval_seq(reftime, &subn[0]), eval_seq(reftime, &subn[1])),
            "<seq> -> <seq> to <seq>" =>
                kronos::interval(eval_seq(reftime, &subn[0]), eval_seq(reftime, &subn[2])),
            ////////////////////////////////////////////////////////////////////////////
            _ => panic!("Unknown [eval_seq] spec={:?}", spec)
        }
    } else {
        unreachable!("what!");
    }
}



pub fn eval(reftime: DateTime, n: &earley::Subtree) -> kronos::Range {
    if let &earley::Subtree::Node(ref spec, ref subn) = n {
        println!("* {:?} ==> {:?}", spec,
                 subn.iter().map(|i| {
                     match i {
                         &earley::Subtree::Leaf(_, ref n) => n.to_string(),
                         &earley::Subtree::Node(ref n, _) => n.to_string(),
                     }
                 }).collect::<Vec<_>>().join(" | "));
        match spec.as_ref() {
            "<time> -> <number>" =>
                kronos::a_year(xtract!(Tobj::Num, eval_terminal(&subn[0]))),
            "<time> -> this <seq>" =>
                kronos::this(eval_seq(reftime, &subn[1]), reftime),
            "<time> -> next <seq>" =>
                kronos::next(eval_seq(reftime, &subn[1]), 1, reftime),
            "<time> -> <seq>" =>
                kronos::this(eval_seq(reftime, &subn[0]), reftime),
            "<time> -> <seq> after next" =>
                kronos::next(eval_seq(reftime, &subn[0]), 2, reftime),
            ////////////////////////////////////////////////////////////////////////////
            "<time> -> <seq> <time>" => { // HIGHLY ambiguous and not clearly needed
                let t = eval(reftime, &subn[1]);
                kronos::this(eval_seq(t.start, &subn[0]), t.start)
            },
            "<time> -> <seq> <number>" => {
                let y = xtract!(Tobj::Num, eval_terminal(&subn[1]));
                let y = kronos::a_year(y);
                kronos::this(eval_seq(y.start, &subn[0]), y.start)
            },
            "<time> -> <seq> after <time>" => {
                let t = eval(reftime, &subn[2]);
                kronos::this(eval_seq(t.start, &subn[0]), t.start)
            },
            "<time> -> <duration> after <time>" => {
                let (g, n) = duration_to_grain(&subn[0]);
                kronos::shift(eval(reftime, &subn[2]), n, g)
            },
            "<time> -> <ordinal> <seq> of <number>" => {
                let n = xtract!(Tobj::Num, eval_terminal(&subn[0])) as usize;
                let s = kronos::nthof(n, eval_seq(reftime, &subn[1]), kronos::year());
                let y = xtract!(Tobj::Num, eval_terminal(&subn[3]));
                let y = kronos::a_year(y);
                kronos::this(s, y.start)
            },
            "<time> -> last <seq> of <number>" => {
                let s = kronos::lastof(1, eval_seq(reftime, &subn[1]), kronos::year());
                let y = xtract!(Tobj::Num, eval_terminal(&subn[3]));
                let y = kronos::a_year(y);
                kronos::this(s, y.start)
            },
            ////////////////////////////////////////////////////////////////////////////
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
            ////////////////////////////////////////////////////////////////////////////
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
            println!("{:?}\n", eval(reftime, &tree));
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
        let s = "last day of feb 2013";
        let s = "july 23rd";
        let s = "last week of feb";
        let s = "2nd month of 2012";
        let s = "3rd day of june";
        let s = "last day of feb next year";
        let s = "mondays of june";

        let s = "mon feb 28th"; // slow
        let s = "2nd thu of sep 2016";
        let s = "3 days after mon feb 28th";

        let s = "1st thu of the month"; // OK, it's != this month
        let s = "1st thu of this month";

        let s = "feb next year"; // doesn't work
        let s = "4th day of next year"; // doesn't work
        let s = "the 2nd day, of the 3rd week, of february"; // some branches don't finish
    }
}

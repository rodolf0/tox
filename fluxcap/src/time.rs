use earlgrey;
use kronos;
use lexers;

use chrono::naive::datetime::NaiveDateTime as DateTime;
use earlgrey::Subtree;
use kronos::Granularity as g;
use kronos::constants as k;
use regex::Regex;
use std::str::FromStr;
use std::collections::HashMap;

fn build_grammar() -> earlgrey::Grammar {
    let mut gb = earlgrey::GrammarBuilder::new();

    lazy_static! {
        static ref STOP_WORDS: HashMap<String, Regex> = [
            "today", "tomorrow", "yesterday",
            "days?", "weeks?", "months?", "quarters?", "years?", "weekends?",
            "this", "next", "of", "the", "(of|in)", "before", "after", "last",
            "until", "from", "to", "and", "between", "in", "a", "ago",
        ].iter()
         .map(|s| (s.to_string(), Regex::new(&format!("^{}$", s)).unwrap()))
         .collect();
    }
    for (sw, rx) in STOP_WORDS.iter() {
        gb = gb.symbol((sw.as_str(), move |n: &str| rx.is_match(n)));
    }

    gb.symbol("<S>")
      // terminals
      .symbol(("<number>", |n: &str| i32::from_str(n).is_ok()))
      .symbol(("<ordinal>", |n: &str| k::ordinal(n).or(k::short_ordinal(n)).is_some()))
      .symbol(("<day-of-week>", |d: &str| k::weekday(d).is_some()))
      .symbol(("<day-of-month>", |n: &str| match k::ordinal(n).or(k::short_ordinal(n)) {
          Some(dom) => (0 < dom && dom < 32), _ => false,
      }))
      .symbol(("<named-month>", |m: &str| k::month(m).is_some()))
      .symbol(("<year>", |n: &str| match i32::from_str(n) {
          Ok(y) => (999 < y && y < 2101), _ => false,
      }))

      // optional prefix <the>
      .symbol("<the>")
      .rule("<the>", &[])
      .rule("<the>", &["the"])

      .symbol("<named-seq>")
      .rule("<named-seq>", &["<named-month>"])
      .rule("<named-seq>", &["<day-of-week>"])
      .rule("<named-seq>", &["<day-of-month>"])
      // TODO: add seasons

      .symbol("<duration>")
      .rule("<duration>", &["days?"])
      .rule("<duration>", &["weeks?"])
      .rule("<duration>", &["months?"])
      .rule("<duration>", &["quarters?"])
      .rule("<duration>", &["years?"])

      .symbol("<cycle>")
      .rule("<cycle>", &["weekends?"])
      .rule("<cycle>", &["<duration>"])
      .rule("<cycle>", &["<named-seq>"])

      .symbol("<range>")
      .rule("<range>", &["today"])
      .rule("<range>", &["tomorrow"])
      //.rule("<range>", &["yesterday"])
      .rule("<range>", &["<year>"])
      .rule("<range>", &["<named-seq>"])
      .rule("<range>", &["<the>", "<day-of-month>"])

      // this-next-last
      .rule("<range>", &["this", "<cycle>"])
      .rule("<range>", &["<the>", "next", "<cycle>"])
      //.rule("<range>", &["<the>", "last", "<cycle>"])
      .rule("<range>", &["<the>", "<cycle>", "after", "next"])
      //.rule("<range>", &["<the>", "<cycle>", "before", "last"])

      // nthofs
      .symbol("<nth>")
      .rule("<nth>", &["<the>", "<ordinal>", "<cycle>", "(of|in)"])
      .rule("<nth>", &["<the>", "last", "<cycle>", "(of|in)"])
      .rule("<range>", &["<nth>", "<range>"])
      .rule("<range>", &["<nth>", "<the>", "<duration>"])

      // intersections
      .symbol("<intersect>")
      .rule("<intersect>", &["<cycle>"])
      .rule("<intersect>", &["<intersect>", "<cycle>"])
      .rule("<range>", &["<intersect>", "<cycle>"])
      .rule("<range>", &["<intersect>", "<year>"])
      .rule("<range>", &["<the>", "<day-of-month>", "of", "<range>"])

      // TODO: intersects/nths  evaluated on specific range
      // july next year
      // 3rd day next month

      // shifts
      .symbol("<n-duration>")
      .rule("<n-duration>", &["a", "<duration>"])
      .rule("<n-duration>", &["<number>", "<duration>"])
      .rule("<range>", &["in", "<n-duration>"])
      .rule("<range>", &["<n-duration>", "ago"])
      .rule("<range>", &["<n-duration>", "after", "<range>"])
      .rule("<range>", &["<n-duration>", "before", "<range>"])

      // duration between times
      .symbol("<timediff>")
      .rule("<timediff>", &["<cycle>", "until", "<range>"])
      .rule("<timediff>", &["<cycle>", "between", "<range>", "and", "<range>"])
      .rule("<timediff>", &["<cycle>", "from", "<range>", "to", "<range>"])

      // start
      .rule("<S>", &["<range>"])
      .rule("<S>", &["<timediff>"])

      .into_grammar("<S>")
}



macro_rules! xtract {
    ($p:path, $e:expr) => (match $e {
        &$p(ref x, ref y) => (x, y),
        _ => panic!("Bad xtract match={:?}", $e)
    })
}

fn num(n: &Subtree) -> i32 {
    let (sym, lexeme) = xtract!(Subtree::Leaf, n);
    match sym.as_ref() {
        "<ordinal>" => (k::ordinal(lexeme)
                        .or(k::short_ordinal(lexeme)).unwrap() as i32),
        "<year>" | "<number>" => i32::from_str(lexeme).unwrap(),
        _ => panic!("Unknown sym={:?} lexeme={:?}", sym, lexeme)
    }
}

fn semi_seq(aseq: kronos::Seq, n: &Subtree) -> kronos::Seq {
    let (spec, subn) = xtract!(Subtree::Node, n);
    match spec.as_ref() {
        "<nth> -> <the> <ordinal> <cycle> (of|in)" => {
            let n = num(&subn[1]) as usize;
            kronos::nthof(n, seq(&subn[2]), aseq)
        },
        "<nth> -> <the> last <cycle> (of|in)" => {
            kronos::lastof(1, seq(&subn[2]), aseq)
        },
        _ => panic!("Unknown [semi_seq] spec={:?}", spec)
    }
}

fn seq(n: &Subtree) -> kronos::Seq {
match n {
    &Subtree::Leaf(ref sym, ref lexeme) => match sym.as_ref() {
        "<day-of-week>" => kronos::day_of_week(k::weekday(lexeme).unwrap()),
        "<named-month>" => kronos::month_of_year(k::month(lexeme).unwrap()),
        "<day-of-month>" => {
            let n = k::ordinal(lexeme).or(k::short_ordinal(lexeme)).unwrap();
            kronos::nthof(n, kronos::day(), kronos::month())
        },
        _ => panic!("Unknown sym={:?} lexeme={:?}", sym, lexeme)
    },
    &Subtree::Node(ref spec, ref subn) => match spec.as_ref() {
        "<named-seq> -> <day-of-week>" => seq(&subn[0]),
        "<named-seq> -> <day-of-month>" => seq(&subn[0]),
        "<named-seq> -> <named-month>" => seq(&subn[0]),
        "<duration> -> days?" => kronos::day(),
        "<duration> -> weeks?" => kronos::week(),
        "<duration> -> months?" => kronos::month(),
        "<duration> -> quarters?" => kronos::quarter(),
        "<duration> -> years?" => kronos::year(),
        "<cycle> -> weekends?" => kronos::weekend(),
        "<cycle> -> <named-seq>" => seq(&subn[0]),
        "<cycle> -> <duration>" => seq(&subn[0]),
        //////////////////////////////////////////////////////////////////////
        "<intersect> -> <cycle>" => seq(&subn[0]),
        "<intersect> -> <intersect> <cycle>" => {
            kronos::intersect(seq(&subn[0]), seq(&subn[1]))
        },
        //////////////////////////////////////////////////////////////////////
        _ => panic!("Unknown [seq] spec={:?}", spec)
    }
}
}

fn seq_from_grain(g: kronos::Granularity) -> kronos::Seq {
    match g {
        g::Day => kronos::day(),
        g::Week => kronos::week(),
        g::Month => kronos::month(),
        g::Quarter => kronos::quarter(),
        g::Year => kronos::year(),
    }
}

fn calc_duration(reftime: DateTime, n: &Subtree) -> (i32, kronos::Granularity) {
    let (spec, subn) = xtract!(Subtree::Node, n);
    match spec.as_ref() {
        "<n-duration> -> a <duration>" => {
            let s = kronos::this(seq(&subn[1]), reftime);
            (1, s.grain)
        }
        "<n-duration> -> <number> <duration>" => {
            let n = num(&subn[0]);
            let s = kronos::this(seq(&subn[1]), reftime);
            (n, s.grain)
        }
        _ => panic!("Unknown [n-duration] spec={:?}", spec)
    }
}

fn eval_range(reftime: DateTime, n: &Subtree) -> kronos::Range {
    let (spec, subn) = xtract!(Subtree::Node, n);
    match spec.as_ref() {
        "<range> -> today" => kronos::this(kronos::day(), reftime),
        "<range> -> tomorrow" => kronos::next(kronos::day(), 1, reftime),
        "<range> -> <year>" => kronos::a_year(num(&subn[0])),
        "<range> -> <named-seq>" => kronos::this(seq(&subn[0]), reftime),
        "<range> -> <the> <day-of-month>" => kronos::this(seq(&subn[1]), reftime),
        "<range> -> this <cycle>" => kronos::this(seq(&subn[1]), reftime),
        "<range> -> <the> next <cycle>" => kronos::next(seq(&subn[2]), 1, reftime),
        "<range> -> <the> <cycle> after next" => kronos::next(seq(&subn[1]), 2, reftime),
        ///////////// Intersect ////////////////////////////////
        "<range> -> <intersect> <year>" => {
            let y = kronos::a_year(num(&subn[1]));
            kronos::this(seq(&subn[0]), y.start)
        },
        "<range> -> <intersect> <cycle>" => {
            let i = kronos::intersect(seq(&subn[0]), seq(&subn[1]));
            kronos::this(i, reftime)
        },
        "<range> -> <the> <day-of-month> of <range>" => {
            let reftime = eval_range(reftime, &subn[3]);
            kronos::this(seq(&subn[1]), reftime.start)
        },
        ///////////// Shifts ///////////////////////////////////
        "<range> -> in <n-duration>" => {
            let (n, grain) = calc_duration(reftime, &subn[1]);
            let today = kronos::this(kronos::day(), reftime);
            kronos::shift(today, n, grain)
        },
        "<range> -> <n-duration> ago" => {
            let (n, grain) = calc_duration(reftime, &subn[0]);
            let today = kronos::this(kronos::day(), reftime);
            kronos::shift(today, -n, grain)
        },
        "<range> -> <n-duration> after <range>" => {
            let (n, grain) = calc_duration(reftime, &subn[0]);
            let reftime = eval_range(reftime, &subn[2]);
            let basetime = kronos::this(kronos::day(), reftime.start);
            kronos::shift(basetime, n, grain)
        },
        "<range> -> <n-duration> before <range>" => {
            let (n, grain) = calc_duration(reftime, &subn[0]);
            let reftime = eval_range(reftime, &subn[2]);
            let basetime = kronos::this(kronos::day(), reftime.start);
            kronos::shift(basetime, -n, grain)
        },
        //////////// Nths //////////////////////////////////////
        "<range> -> <nth> <range>" => {
            let reftime = eval_range(reftime, &subn[1]);
            let s = semi_seq(seq_from_grain(reftime.grain), &subn[0]);
            kronos::this(s, reftime.start)
        },
        "<range> -> <nth> <the> <duration>" => {
            let s = semi_seq(seq(&subn[2]), &subn[0]);
            kronos::this(s, reftime)
        },
        ////////////////////////////////////////////////////////////////////////////
        _ => panic!("Unknown [eval] spec={:?}", spec)
    }
}

fn eval_timediff(reftime: DateTime, n: &Subtree) -> usize {
    let (spec, subn) = xtract!(Subtree::Node, n);
    match spec.as_ref() {
        "<timediff> -> <cycle> until <range>" => {
            let target = eval_range(reftime, &subn[2]);
            seq(&subn[0])(reftime)
                .skip_while(|x| x.start < reftime)
                .take_while(|x| x.start < target.start)
                .count()
        },
        "<timediff> -> <cycle> from <range> to <range>" |
        "<timediff> -> <cycle> between <range> and <range>" => {
            let t0 = eval_range(reftime, &subn[2]);
            let t1 = eval_range(reftime, &subn[4]);
            seq(&subn[0])(t0.start)
                .skip_while(|x| x.start < t0.start)
                .take_while(|x| x.start < t1.start)
                .count()
        },
        ////////////////////////////////////////////////////////////////////////////
        _ => panic!("Unknown [timediff] spec={:?}", spec)
    }
}

pub struct TimeMachine {
    parser: earlgrey::EarleyParser,
}

impl TimeMachine {
    pub fn new() -> TimeMachine {
        TimeMachine{parser: earlgrey::EarleyParser::new(build_grammar())}
    }

    fn parse(&self, t: &str) -> Vec<Subtree> {
        let mut tokenizer = lexers::DelimTokenizer::from_str(t, ", ", true);
        match self.parser.parse(&mut tokenizer) {
            Err(_) => Vec::new(),
            Ok(state) => earlgrey::all_trees(self.parser.g.start(), &state)
        }
    }

    pub fn parse_time(&self, reftime: DateTime, t: &str) -> Option<kronos::Range> {
        let trees = self.parse(t);
        //for t in &trees { t.print(); }
        assert_eq!(trees.len(), 1); // just evaluate 1st option
        let (spec, subn) = xtract!(Subtree::Node, &trees[0]);
        match spec.as_ref() {
            "<S> -> <range>" => Some(eval_range(reftime, &subn[0])),
            _ => None
        }
    }

    pub fn time_diff(&self, reftime: DateTime, t: &str) -> Option<usize> {
        let trees = self.parse(t);
        assert_eq!(trees.len(), 1); // just evaluate 1st option
        let (spec, subn) = xtract!(Subtree::Node, &trees[0]);
        match spec.as_ref() {
            "<S> -> <timediff>" => Some(eval_timediff(reftime, &subn[0])),
            _ => None
        }
    }

    pub fn print_trees(&self, t: &str) {
        for tree in self.parse(t) { tree.print(); }
    }
}


#[cfg(test)]
mod tests {
    use chrono::naive::datetime::NaiveDateTime as DateTime;
    use super::TimeMachine;
    use kronos::Granularity as g;
    use kronos;

    fn d(year: i32, month: u32, day: u32) -> DateTime {
        use chrono::naive::date::NaiveDate as Date;
        Date::from_ymd(year, month, day).and_hms(0, 0, 0)
    }
    fn r(s: DateTime, e: DateTime, gr: kronos::Granularity) -> kronos::Range {
        kronos::Range{start: s, end: e, grain: gr}
    }

    #[test]
    fn t_thisnext() {
        let tm = TimeMachine::new();
        let x = r(d(2016, 9, 12), d(2016, 9, 13), g::Day);
        assert_eq!(tm.parse_time(d(2016, 9, 5), "next monday"), Some(x));
        let x = r(d(2016, 9, 5), d(2016, 9, 6), g::Day);
        assert_eq!(tm.parse_time(d(2016, 9, 5), "this monday"), Some(x));
        let x = r(d(2017, 3, 1), d(2017, 4, 1), g::Month);
        assert_eq!(tm.parse_time(d(2016, 9, 5), "next march"), Some(x));
        assert_eq!(tm.parse_time(d(2016, 9, 5), "this march"), Some(x));
        let x = r(d(2016, 3, 1), d(2016, 4, 1), g::Month);
        assert_eq!(tm.parse_time(d(2016, 3, 5), "this march"), Some(x));
        let x = r(d(2017, 1, 1), d(2018, 1, 1), g::Year);
        assert_eq!(tm.parse_time(d(2016, 3, 5), "next year"), Some(x));
        let x = r(d(2016, 3, 6), d(2016, 3, 13), g::Week);
        assert_eq!(tm.parse_time(d(2016, 3, 5), "next week"), Some(x));
        let x = r(d(2016, 10, 1), d(2016, 11, 1), g::Month);
        assert_eq!(tm.parse_time(d(2016, 9, 5), "next month"), Some(x));
        let x = r(d(2016, 9, 13), d(2016, 9, 14), g::Day);
        assert_eq!(tm.parse_time(d(2016, 9, 5), "tue after next"), Some(x));
    }
    #[test]
    fn t_direct() {
        let tm = TimeMachine::new();
        let x = r(d(2002, 1, 1), d(2003, 1, 1), g::Year);
        assert_eq!(tm.parse_time(d(2016, 9, 5), "2002"), Some(x));
        let x = r(d(2016, 10, 31), d(2016, 11, 1), g::Day);
        assert_eq!(tm.parse_time(d(2016, 10, 26), "monday"), Some(x));
        let x = r(d(2016, 10, 26), d(2016, 10, 27), g::Day);
        assert_eq!(tm.parse_time(d(2016, 10, 26), "today"), Some(x));
        assert_eq!(tm.parse_time(d(2016, 10, 25), "tomorrow"), Some(x));
        let x = r(d(2016, 9, 12), d(2016, 9, 13), g::Day);
        assert_eq!(tm.parse_time(d(2016, 9, 5), "the 12th"), Some(x));
        assert_eq!(tm.parse_time(d(2016, 9, 12), "the 12th"), Some(x));
    }
    #[test]
    fn t_nthof() {
        let tm = TimeMachine::new();
        let x = r(d(2017, 6, 19), d(2017, 6, 20), g::Day);
        assert_eq!(tm.parse_time(d(2016, 9, 5), "the 3rd mon of june"), Some(x));
        let x = r(d(2016, 9, 3), d(2016, 9, 4), g::Day);
        assert_eq!(tm.parse_time(d(2016, 9, 5), "3rd day of the month"), Some(x));
        let x = r(d(2017, 8, 6), d(2017, 8, 13), g::Week);
        assert_eq!(tm.parse_time(d(2016, 9, 5), "2nd week in august"), Some(x));
        let x = r(d(2017, 2, 24), d(2017, 2, 25), g::Day);
        assert_eq!(tm.parse_time(d(2017, 1, 1), "8th fri of the year"), Some(x));
        let x = r(d(2020, 2, 29), d(2020, 3, 1), g::Day);
        assert_eq!(tm.parse_time(d(2020, 1, 1), "last day of feb"), Some(x));
        let x = r(d(2017, 5, 9), d(2017, 5, 10), g::Day);
        assert_eq!(tm.parse_time(d(2016, 9, 5), "the 3rd day of the 2nd week of may"), Some(x));
        let x = r(d(2014, 6, 2), d(2014, 6, 3), g::Day);
        assert_eq!(tm.parse_time(d(2016, 9, 5), "2nd day of june 2014"), Some(x));
        let x = r(d(2014, 9, 11), d(2014, 9, 12), g::Day);
        assert_eq!(tm.parse_time(d(2016, 9, 5), "2nd thu of sep 2014"), Some(x));
    }
    #[test]
    fn t_intersect() {
        let tm = TimeMachine::new();
        let x = r(d(1984, 2, 27), d(1984, 2, 28), g::Day);
        assert_eq!(tm.parse_time(d(2016, 9, 5), "27th feb 1984"), Some(x));
        let x = r(d(2022, 2, 28), d(2022, 3, 1), g::Day);
        assert_eq!(tm.parse_time(d(2017, 9, 5), "mon feb 28th"), Some(x));
        let x = r(d(2016, 11, 18), d(2016, 11, 19), g::Day);
        assert_eq!(tm.parse_time(d(2016, 10, 24), "friday 18th"), Some(x));
        let x = r(d(2017, 6, 18), d(2017, 6, 19), g::Day);
        assert_eq!(tm.parse_time(d(2016, 10, 24), "18th of june"), Some(x));
        let x = r(d(2017, 2, 27), d(2017, 2, 28), g::Day);
        assert_eq!(tm.parse_time(d(2016, 10, 24), "feb 27th"), Some(x));
    }
    #[test]
    fn t_seqrange() {
        let tm = TimeMachine::new();
        let x = r(d(1984, 3, 4), d(1984, 3, 11), g::Week);
        assert_eq!(tm.parse_time(d(2016, 9, 5), "10th week of 1984"), Some(x));
        let x = r(d(2016, 11, 15), d(2016, 11, 16), g::Day);
        assert_eq!(tm.parse_time(d(2016, 9, 5),
                    "third tuesday of the month after next"), Some(x));
        let x = r(d(1987, 1, 12), d(1987, 1, 13), g::Day);
        assert_eq!(tm.parse_time(d(2016, 9, 5),
                    "the 2nd day of the 3rd week of 1987"), Some(x));
        //let x = r(d(2016, 10, 2), d(2016, 10, 9), g::Week);
        //assert_eq!(tm.parse_time(d(2016, 9, 5), "2nd week next month"), Some(x));
        //let ex = kronos::range{
            //start: d(2017, 1, 4), end: d(2017, 1, 5), grain: g::Day};
        //assert_eq!(parse_time("4th day next year",
                              //d(2016, 9, 5)), Some(ex));
        //let ex = kronos::range{
            //start: d(2017, 7, 1), end: d(2017, 8, 1), grain: g::Month};
        //assert_eq!(parse_time("july next year", d(2016, 9, 5)), Some(ex));
    }
    #[test]
    fn t_timediff() {
        let tm = TimeMachine::new();
        assert_eq!(tm.time_diff(d(2016, 9, 5), "days until tomorrow"), Some(1));
        assert_eq!(tm.time_diff(d(2016, 9, 5), "months until 2018"), Some(15));
        assert_eq!(tm.time_diff(d(2016, 9, 5), "weeks until dec"), Some(12));
        assert_eq!(tm.time_diff(d(2016, 10, 25), "mon until nov 14th"), Some(2));
        assert_eq!(tm.time_diff(d(2016, 10, 25), "weekends until jan"), Some(10));
    }
    #[test]
    fn t_shifts() {
        let tm = TimeMachine::new();
        let x = r(d(2016, 10, 12), d(2016, 10, 13), g::Day);
        assert_eq!(tm.parse_time(d(2016, 10, 26), "2 weeks ago"), Some(x));
        let x = r(d(2017, 2, 21), d(2017, 2, 22), g::Day);
        assert_eq!(tm.parse_time(d(2016, 10, 26), "a week after feb 14th"), Some(x));
        let x = r(d(2017, 2, 21), d(2017, 2, 22), g::Day);
        assert_eq!(tm.parse_time(d(2016, 10, 26), "a week before feb 28th"), Some(x));
        let x = r(d(2017, 10, 26), d(2017, 10, 27), g::Day);
        assert_eq!(tm.parse_time(d(2016, 10, 26), "in a year"), Some(x));
    }
}

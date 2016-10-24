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

pub fn build_grammar() -> earlgrey::Grammar {
    let mut gb = earlgrey::GrammarBuilder::new();

    lazy_static! {
        static ref STOP_WORDS: HashMap<String, Regex> = [
            "today", "tomorrow", "yesterday",
            "days?", "weeks?", "months?", "quarters?", "years?", "weekend",
            "this", "next", "of", "the", "(of|in)", "before", "after", "last",
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

      .symbol("<cycle>")
      .rule("<cycle>", &["days?"])
      .rule("<cycle>", &["weeks?"])
      .rule("<cycle>", &["months?"])
      .rule("<cycle>", &["quarters?"])
      .rule("<cycle>", &["years?"])
      .rule("<cycle>", &["weekend"])
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

      // 2nd tuesday in march
      // 3rd day of the month
      // 2nd week in august
      // 1st friday of the year
      // 2nd day of the 3rd week of june
      .symbol("<nth>")
      .symbol("<cycle-nth>")
      .rule("<cycle-nth>", &["<nth>"])
      .rule("<cycle-nth>", &["<the>", "<cycle>"])
      .rule("<nth>", &["<the>", "<ordinal>", "<cycle>", "(of|in)", "<cycle-nth>"])
      .rule("<nth>", &["<the>", "last", "<cycle>", "(of|in)", "<cycle-nth>"])
      //.rule("<nth>", &["<the>", "<ordinal>", "<cycle>", "after", "<cycle-nth>"])
      .rule("<range>", &["<nth>"])
      .rule("<range>", &["<nth>", "<year>"])

      // intersections
      // friday 18th
      // 18th of june
      // feb 18th
      // feb 18th 2014
      // 18th 2018
      .symbol("<intersect>")
      .rule("<intersect>", &["<named-seq>"])
      .rule("<intersect>", &["<named-seq>", "<intersect>"])
      .rule("<range>", &["<named-seq>", "<intersect>"])
      .rule("<range>", &["<intersect>", "<year>"])
      .rule("<range>", &["<the>", "<day-of-month>", "of", "<range>"])


      // the 10th week of 1948
      // the 2nd day of the 3rd week of 1987
      // 3rd day next month
      // 2nd month of <2018>
      // 1st tuesday of <last summer>
      // Grab grain of <range>, create a sequence, then evaluate on <range>

      //.symbol("<duration>")
      //.rule("<duration>", &["days?"])
      //.rule("<duration>", &["<number>", "<duration>"])
      //.rule("<S>", &["<duration>", "after", "<range>"])
      //.rule("<number>", &["<cycle>", "until", "<range>"]) // seconds until feb 24th, mondays until next year

      // start
      .rule("<S>", &["<range>"])
      //.rule("<S>", &["<timediff>"])

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
        "<cycle> -> days?" => kronos::day(),
        "<cycle> -> weeks?" => kronos::week(),
        "<cycle> -> months?" => kronos::month(),
        "<cycle> -> quarters?" => kronos::quarter(),
        "<cycle> -> years?" => kronos::year(),
        "<cycle> -> <named-seq>" => seq(&subn[0]),
        "<cycle> -> weekend" => kronos::weekend(),
        ////////////////////////////////////////////////////////////////////////////
        "<cycle-nth> -> <nth>" => seq(&subn[0]),
        "<cycle-nth> -> <the> <cycle>" => seq(&subn[1]),
        "<nth> -> <the> <ordinal> <cycle> (of|in) <cycle-nth>" => {
            let n = num(&subn[1]) as usize;
            kronos::nthof(n, seq(&subn[2]), seq(&subn[4]))
        },
        "<nth> -> <the> last <cycle> (of|in) <cycle-nth>" => {
            kronos::lastof(1, seq(&subn[2]), seq(&subn[4]))
        },
        ////////////////////////////////////////////////////////////////////////////
        "<intersect> -> <named-seq> <intersect>" => {
            kronos::intersect(seq(&subn[0]), seq(&subn[1]))
        },
        "<intersect> -> <named-seq>" => seq(&subn[0]),
        "<intersect> -> <year>" => panic!("TODO"),
        ////////////////////////////////////////////////////////////////////////////
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

pub fn eval_range(reftime: DateTime, n: &Subtree) -> kronos::Range {
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
        "<range> -> <nth>" => kronos::this(seq(&subn[0]), reftime),
        "<range> -> <nth> <year>" => {
            let y = kronos::a_year(num(&subn[1]));
            kronos::this(seq(&subn[0]), y.start)
        },
        "<range> -> <named-seq> <intersect>" => {
            let i = kronos::intersect(seq(&subn[0]), seq(&subn[1]));
            kronos::this(i, reftime)
        },
        "<range> -> <intersect> <year>" => {
            let y = kronos::a_year(num(&subn[1]));
            kronos::this(seq(&subn[0]), y.start)
        },
        "<range> -> <the> <day-of-month> of <range>" => {
            let reftime = eval_range(reftime, &subn[3]);
            kronos::this(seq(&subn[1]), reftime.start)
        },

        /////////// testing, START HERE 2nd week next month
        //"<range> -> <the> <ordinal> <cycle> <range>" => {
            //let reftime = eval_range(reftime, &subn[3]);
            //let n = num(&subn[1]) as usize;
            //let s = seq_from_grain(reftime.grain);
            //println!("{:?}", reftime);
            //kronos::this(kronos::nthof(n, seq(&subn[2]), s), reftime.start)
        //},
        ////////////////////////////////////////////////////////////////////////////
        _ => panic!("Unknown [eval] spec={:?}", spec)
    }
}


pub fn parse_time(t: &str, reftime: DateTime) -> Option<kronos::Range> {
    let parser = earlgrey::EarleyParser::new(build_grammar());
    let mut tokenizer = lexers::DelimTokenizer::from_str(t, ", ", true);
    match parser.parse(&mut tokenizer) {
        Ok(state) => {
            let trees = earlgrey::all_trees(parser.g.start(), &state);
            let mut x = kronos::a_year(2012);
            // TODO: yuck
            for t in &trees {
                t.print();
                let (spec, subn) = xtract!(Subtree::Node, t);
                x = match spec.as_ref() {
                    "<S> -> <range>" => eval_range(reftime, &subn[0]),
                    _ => panic!("Unknown [eval] spec={:?}", spec)
                };
                println!("{:?}", x);
            }
            assert_eq!(trees.len(), 1); // don't allow ambiguity
            Some(x)
        },
        Err(_) => None
    }
}



#[cfg(test)]
mod tests {
    use chrono::naive::date::NaiveDate as Date;
    use chrono::naive::datetime::NaiveDateTime as DateTime;
    use super::parse_time;
    use kronos;
    use kronos::Granularity as g;

    fn d(year: i32, month: u32, day: u32) -> DateTime {
        Date::from_ymd(year, month, day).and_hms(0, 0, 0)
    }
    #[test]
    fn t_thisnext() {
        let ex = kronos::Range{
            start: d(2016, 9, 12), end: d(2016, 9, 13), grain: g::Day};
        assert_eq!(parse_time("next monday", d(2016, 9, 5)), Some(ex));
        let ex = kronos::Range{
            start: d(2016, 9, 5), end: d(2016, 9, 6), grain: g::Day};
        assert_eq!(parse_time("this monday", d(2016, 9, 5)), Some(ex));
        let ex = kronos::Range{
            start: d(2017, 3, 1), end: d(2017, 4, 1), grain: g::Month};
        assert_eq!(parse_time("next march", d(2016, 9, 5)), Some(ex));
        assert_eq!(parse_time("this march", d(2016, 9, 5)), Some(ex));
        let ex = kronos::Range{
            start: d(2016, 3, 1), end: d(2016, 4, 1), grain: g::Month};
        assert_eq!(parse_time("this march", d(2016, 3, 5)), Some(ex));
        let ex = kronos::Range{
            start: d(2017, 1, 1), end: d(2018, 1, 1), grain: g::Year};
        assert_eq!(parse_time("next year", d(2016, 3, 5)), Some(ex));
        let ex = kronos::Range{
            start: d(2016, 3, 6), end: d(2016, 3, 13), grain: g::Week};
        assert_eq!(parse_time("next week", d(2016, 3, 5)), Some(ex));
        let ex = kronos::Range{
            start: d(2016, 10, 1), end: d(2016, 11, 1), grain: g::Month};
        assert_eq!(parse_time("next month", d(2016, 9, 5)), Some(ex));
    }
    #[test]
    fn t_thedom() {
        let ex = kronos::Range{
            start: d(2016, 9, 12), end: d(2016, 9, 13), grain: g::Day};
        assert_eq!(parse_time("the 12th", d(2016, 9, 5)), Some(ex));
        assert_eq!(parse_time("the 12th", d(2016, 9, 12)), Some(ex));
    }
    #[test]
    fn t_afternext() {
        let ex = kronos::Range{
            start: d(2016, 9, 13), end: d(2016, 9, 14), grain: g::Day};
        assert_eq!(parse_time("tue after next", d(2016, 9, 5)), Some(ex));
    }
    #[test]
    fn t_year() {
        let ex = kronos::Range{
            start: d(2002, 1, 1), end: d(2003, 1, 1), grain: g::Year};
        assert_eq!(parse_time("2002", d(2016, 9, 5)), Some(ex));
    }
    #[test]
    fn t_nthseqofseq() {
        let ex = kronos::Range{
            start: d(2017, 6, 19), end: d(2017, 6, 20), grain: g::Day};
        assert_eq!(parse_time("the 3rd mon of june", d(2016, 9, 5)), Some(ex));
        let ex = kronos::Range{
            start: d(2016, 9, 3), end: d(2016, 9, 4), grain: g::Day};
        assert_eq!(parse_time("3rd day of the month", d(2016, 9, 5)), Some(ex));
        let ex = kronos::Range{
            start: d(2017, 8, 6), end: d(2017, 8, 13), grain: g::Week};
        assert_eq!(parse_time("2nd week in august", d(2016, 9, 5)), Some(ex));
        let ex = kronos::Range{
            start: d(2017, 2, 24), end: d(2017, 2, 25), grain: g::Day};
        assert_eq!(parse_time("8th fri of the year", d(2017, 1, 1)), Some(ex));
        let ex = kronos::Range{
            start: d(2020, 2, 29), end: d(2020, 3, 1), grain: g::Day};
        assert_eq!(parse_time("last day of feb", d(2020, 1, 1)), Some(ex));
        let ex = kronos::Range{
            start: d(2017, 5, 9), end: d(2017, 5, 10), grain: g::Day};
        assert_eq!(parse_time("the 3rd day of the 2nd week of may",
                              d(2016, 9, 5)), Some(ex));
        let ex = kronos::Range{
            start: d(2014, 6, 2), end: d(2014, 6, 3), grain: g::Day};
        assert_eq!(parse_time("2nd day of june 2014", d(2016, 9, 5)), Some(ex));
        let ex = kronos::Range{
            start: d(2014, 9, 11), end: d(2014, 9, 12), grain: g::Day};
        assert_eq!(parse_time("2nd thu of sep 2014", d(2016, 9, 5)), Some(ex));
    }
    #[test]
    fn t_intersect() {
        let ex = kronos::Range{
            start: d(1984, 2, 27), end: d(1984, 2, 28), grain: g::Day};
        assert_eq!(parse_time("27th feb 1984", d(2016, 9, 5)), Some(ex));
        let ex = kronos::Range{
            start: d(2022, 2, 28), end: d(2022, 3, 1), grain: g::Day};
        assert_eq!(parse_time("mon feb 28th", d(2017, 9, 5)), Some(ex));
        let ex = kronos::Range{
            start: d(2016, 11, 18), end: d(2016, 11, 19), grain: g::Day};
        assert_eq!(parse_time("friday 18th", d(2016, 10, 24)), Some(ex));
        let ex = kronos::Range{
            start: d(2017, 6, 18), end: d(2017, 6, 19), grain: g::Day};
        assert_eq!(parse_time("18th of june", d(2016, 10, 24)), Some(ex));
        let ex = kronos::Range{
            start: d(2017, 2, 27), end: d(2017, 2, 28), grain: g::Day};
        assert_eq!(parse_time("feb 27th", d(2016, 10, 24)), Some(ex));
    }
    #[test]
    fn t_seqrange() {
        //let ex = kronos::Range{
            //start: d(2016, 10, 2), end: d(2016, 10, 9), grain: g::Week};
        //assert_eq!(parse_time("2nd week next month", d(2016, 9, 5)), Some(ex));
        //let ex = kronos::Range{
            //start: d(2017, 1, 4), end: d(2017, 1, 5), grain: g::Day};
        //assert_eq!(parse_time("4th day next year",
                              //d(2016, 9, 5)), Some(ex));
        //let ex = kronos::Range{
            //start: d(2017, 7, 1), end: d(2017, 8, 1), grain: g::Month};
        //assert_eq!(parse_time("july next year", d(2016, 9, 5)), Some(ex));
    }

    // durations
        //let ex = kronos::Range{
            //start: d(2017, 3, 3), end: d(2017, 3, 4), grain: g::Day};
        //assert_eq!(parse_time("3 days after mon feb 28th", d(2016, 9, 5)), Some(ex));
}

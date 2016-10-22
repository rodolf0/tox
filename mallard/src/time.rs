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
            "days?", "weeks?", "months?", "quarters?", "years?",
            "this", "next", "of", "the", "(of|in)", "(of|in|of the)",
            "before", "after", "weekend", "last",
            "today", "tomorrow", "yesterday",
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

      .symbol("<cycle>")
      .rule("<cycle>", &["days?"])
      .rule("<cycle>", &["weeks?"])
      .rule("<cycle>", &["months?"])
      .rule("<cycle>", &["quarters?"])
      .rule("<cycle>", &["years?"])
      .rule("<cycle>", &["<named-seq>"])

      .symbol("<range>")
      .rule("<range>", &["today"])
      .rule("<range>", &["tomorrow"])
      .rule("<range>", &["yesterday"])
      .rule("<range>", &["<year>"])
      .rule("<range>", &["<named-seq>"])

      // this-next-last
      .rule("<range>", &["this", "<cycle>"])
      .rule("<range>", &["<the>", "next", "<cycle>"])
      .rule("<range>", &["<the>", "last", "<cycle>"])
      .rule("<range>", &["<the>", "<cycle>", "after", "next"])
      .rule("<range>", &["<the>", "<cycle>", "before", "last"])

      // 3rd day of the month
      // 2nd week in august
      // 2nd tuesday in march
      // 1st friday of the year
      // 2nd day of the 3rd week of june
      .symbol("<nth>")
      .symbol("<cycle-nth>")
      .rule("<cycle-nth>", &["<nth>"])
      .rule("<cycle-nth>", &["<cycle>"])
      .rule("<nth>", &["<the>", "<ordinal>", "<cycle>", "(of|in|of the)", "<cycle-nth>"])
      .rule("<nth>", &["<the>", "last", "<cycle>", "(of|in|of the)", "<cycle-nth>"])
      .rule("<nth>", &["<the>", "<ordinal>", "<cycle>", "after", "<cycle-nth>"])
      .rule("<range>", &["<nth>"])

      // Grab grain of <range>, create a sequence, then evaluate on <range>
      // 2nd month of <2018>, 1st tuesday of <last summer>
      // last day of <feb 2019>
      // after: INFER next grain for seq? 3rd day after tomorrow (in month)
      .rule("<range>", &["<the>", "<ordinal>", "<cycle>", "(of|in)", "<range>"])
      .rule("<range>", &["<the>", "last", "<cycle>", "(of|in)", "<range>"])
      .rule("<range>", &["<the>", "<ordinal>", "<cycle>", "after", "<range>"])

      // intersections
      //3rd day next month
      .rule("<range>", &["<the>", "<day-of-month>"]) // the 12th
      .rule("<range>", &["<the>", "<day-of-month>", "of", "<named-month>"]) // 18th of august
      .rule("<range>", &["<day-of-week>", "<day-of-month>"]) // friday 18th
      .rule("<range>", &["<named-month>", "<day-of-month>"]) // march 18th
      .rule("<range>", &["<named-month>", "<day-of-month>", "<year>"])

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
        "<seq> -> <day-of-week>" => seq(&subn[0]),
        "<seq> -> <day-of-month>" => seq(&subn[0]),
        "<seq> -> <named-month>" => seq(&subn[0]),
        //"<seq> -> <year>" => seq(&subn[0]),
        "<seq> -> <duration>" => seq(&subn[0]),
        "<duration> -> day" => kronos::day(),
        "<duration> -> week" => kronos::week(),
        "<duration> -> month" => kronos::month(),
        "<duration> -> quarter" => kronos::quarter(),
        "<duration> -> year" => kronos::year(),
        "<duration> -> weekend" => kronos::weekend(),
        "<duration> -> <number> <duration>" => {
            let n = num(&subn[0]) as usize;
            kronos::merge(n, seq(&subn[1]))
        },
        ////////////////////////////////////////////////////////////////////////////
        "<seq> -> <ordinal> <seq> of the <seq>" => {
            let n = num(&subn[0]) as usize;
            kronos::nthof(n, seq(&subn[1]), seq(&subn[4]))
        },
        "<seq> -> <ordinal> <seq> of <seq>" => {
            let n = num(&subn[0]) as usize;
            kronos::nthof(n, seq(&subn[1]), seq(&subn[3]))
        },
        "<seq> -> last <seq> of the <seq>" =>
            kronos::lastof(1, seq(&subn[1]), seq(&subn[4])),
        "<seq> -> last <seq> of <seq>" =>
            kronos::lastof(1, seq(&subn[1]), seq(&subn[3])),
        //////////////////////////////////////////////////////////////////////////////
        //"<seq> -> <seq> of <seq>" =>
            //kronos::intersect(seq(&subn[0]), seq(&subn[2])),
        "<seq> -> <seq> <seq>" =>
            kronos::intersect(seq(&subn[0]), seq(&subn[1])),
        //"<seq> -> <seq> to <seq>" =>
            //kronos::interval(seq(&subn[0]), seq(&subn[2])),
        ////////////////////////////////////////////////////////////////////////////
        _ => panic!("Unknown [seq] spec={:?}", spec)
    }
}
}

fn duration_to_grain(n: &Subtree) -> (g, i32) {
    let (spec, subn) = xtract!(Subtree::Node, n);
    match spec.as_ref() {
        "<duration> -> <dur-day>" => (g::Day, 1),
        "<duration> -> <dur-week>" => (g::Week, 1),
        "<duration> -> <dur-month>" => (g::Month, 1),
        "<duration> -> <dur-quarter>" => (g::Quarter, 1),
        "<duration> -> <dur-year>" => (g::Year, 1),
        "<duration> -> <duration>" => duration_to_grain(&subn[0]),
        "<duration> -> a <duration>" => duration_to_grain(&subn[1]),
        "<duration> -> <number> <duration>" => {
            let (g, n2) = duration_to_grain(&subn[1]);
            (g, num(&subn[0]) * n2)
        },
        _ => panic!("Unknown duration rule={:?}", spec)
    }
}

pub fn eval_range(reftime: DateTime, n: &Subtree) -> kronos::Range {
    let (spec, subn) = xtract!(Subtree::Node, n);
    match spec.as_ref() {
        "<range> -> today" => kronos::this(kronos::day(), reftime),
        "<range> -> tomorrow" => kronos::next(kronos::day(), 1, reftime),

        "<range> -> next <seq>" => kronos::next(seq(&subn[1]), 1, reftime),
        "<range> -> this <seq>" => kronos::this(seq(&subn[1]), reftime),
        //"<range> -> the <seq>" => kronos::this(seq(&subn[1]), reftime),
        "<range> -> the <day-of-month>" => kronos::this(seq(&subn[1]), reftime),
        "<range> -> <seq>" => kronos::this(seq(&subn[0]), reftime),
        "<range> -> <seq> after next" => kronos::next(seq(&subn[0]), 2, reftime),
        "<range> -> <year>" => kronos::a_year(num(&subn[0])),

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
            for t in &trees {
                t.print();
            }
            assert_eq!(trees.len(), 1); // don't allow ambiguity
            let (spec, subn) = xtract!(Subtree::Node, &trees[0]);
            match spec.as_ref() {
                "<S> -> <range>" => Some(eval_range(reftime, &subn[0])),
                _ => panic!("Unknown [eval] spec={:?}", spec)
            }
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
        assert_eq!(parse_time("the 3rd monday of june",
                              d(2016, 9, 5)), Some(ex));
        //let ex = kronos::Range{
            //start: d(2017, 5, 9), end: d(2017, 5, 10), grain: g::Day};
        //assert_eq!(parse_time("the 3rd day of the 2nd week of may",
                              //d(2016, 9, 5)), Some(ex));
    }
    #[test]
    fn t_seqrelrange() {
        let ex = kronos::Range{
            start: d(2016, 10, 2), end: d(2016, 10, 9), grain: g::Week};
        assert_eq!(parse_time("2nd week next month", d(2016, 9, 5)), Some(ex));
        let ex = kronos::Range{
            start: d(2017, 1, 4), end: d(2017, 1, 5), grain: g::Day};
        assert_eq!(parse_time("4th day next year",
                              d(2016, 9, 5)), Some(ex));
        let ex = kronos::Range{
            start: d(2017, 7, 1), end: d(2017, 8, 1), grain: g::Month};
        assert_eq!(parse_time("july next year", d(2016, 9, 5)), Some(ex));
    }
    #[test]
    fn t_seqrange() {
        let ex = kronos::Range{
            start: d(2014, 9, 11), end: d(2014, 9, 12), grain: g::Day};
        assert_eq!(parse_time("2nd thu of sep 2014", d(2016, 9, 5)), Some(ex));
        // REQUIRES intersection
        //let ex = kronos::Range{
            //start: d(2017, 2, 28), end: d(2017, 3, 1), grain: g::Day};
        //assert_eq!(parse_time("mon feb 28th", d(2016, 9, 5)), Some(ex));
        //let ex = kronos::Range{
            //start: d(2017, 3, 3), end: d(2017, 3, 4), grain: g::Day};
        //assert_eq!(parse_time("3 days after mon feb 28th", d(2016, 9, 5)), Some(ex));
    }
}

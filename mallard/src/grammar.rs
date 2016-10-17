use earlgrey;

use kronos::constants as k;
use std::str::FromStr;

pub fn build_grammar() -> earlgrey::Grammar {
    // more terminals
    static STOP_WORDS: &'static [&'static str] = &[
        "this", "next", "of", "the", "after", "weekend", "last",
    ];
    let mut gb = earlgrey::GrammarBuilder::new();
    for sw in STOP_WORDS { gb = gb.symbol((*sw, move |n: &str| n == *sw)); }

    gb.symbol("<S>")
      // terminals
      .symbol(("<number>", |n: &str| i32::from_str(n).is_ok()))
      .symbol(("<ordinal>", |n: &str| k::ordinal(n).or(k::short_ordinal(n)).is_some()))
      .symbol(("<day-of-week>", |d: &str| k::weekday(d).is_some()))
      .symbol(("<day-of-month>", |n: &str| k::ordinal(n).or(k::short_ordinal(n)).is_some()))
      .symbol(("<named-month>", |m: &str| k::month(m).is_some()))
      .symbol(("<year>", |n: &str| i32::from_str(n).is_ok()))
      .symbol(("day", |d: &str| d == "day" || d == "days"))
      .symbol(("week", |d: &str| d == "week" || d == "weeks"))
      .symbol(("month", |d: &str| d == "month" || d == "months"))
      .symbol(("quarter", |d: &str| d == "quarter" || d == "quarters"))
      .symbol(("year", |d: &str| d == "year" || d == "years"))

      // durations
      .symbol("<duration>")
      .rule("<duration>", &["day"])
      .rule("<duration>", &["week"])
      .rule("<duration>", &["month"])
      .rule("<duration>", &["quarter"])
      .rule("<duration>", &["year"])
      .rule("<duration>", &["<number>", "<duration>"])

      // sequences
      .symbol("<seq>")
      .rule("<seq>", &["<ordinal>", "<seq>", "of", "the", "<seq>"])
      .rule("<seq>", &["<ordinal>", "<seq>", "of", "<seq>"])
      .rule("<seq>", &["last", "<seq>", "of", "the", "<seq>"])
      .rule("<seq>", &["last", "<seq>", "of", "<seq>"])
      .rule("<seq>", &["<ordinal>", "<seq>"]) // 3rd week
      // this introduces ambiguity
      // .rule("<seq>", &["<seq>", "<seq>"]) // feb 28th

      // base sequences
      .rule("<seq>", &["<day-of-week>"])
      .rule("<seq>", &["<day-of-month>"])
      .rule("<seq>", &["<named-month>"])
      .rule("<seq>", &["<duration>"])

      // ranges
      .symbol("<range>")
      .rule("<S>", &["<range>"])
      .rule("<S>", &["<seq>", "<range>"])
      .rule("<S>", &["<duration>", "after", "<range>"])

      .rule("<range>", &["this", "<seq>"])
      .rule("<range>", &["next", "<seq>"])
      .rule("<range>", &["the", "<seq>"])
      .rule("<range>", &["<seq>"])
      .rule("<range>", &["<seq>", "after", "next"])
      .rule("<range>", &["<year>"])


      //.rule("<number>", &["<duration>", "until", "<range>"]) // seconds until feb 24th

      .into_grammar("<S>")
}

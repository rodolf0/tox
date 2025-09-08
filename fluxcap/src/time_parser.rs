// https://github.com/wit-ai/duckling_old/blob/master/resources/languages/en/corpus/time.clj
// https://github.com/wit-ai/duckling_old/blob/master/resources/languages/en/rules/time.clj

// TODO: types should be date-time or duration (not count and TimeEl)

pub fn time_grammar() -> &'static str {
    r#"
    time_spec := anchored_spec
               | relative_spec
               | scoped_spec
               ;

    anchored_spec := weekday
                   | weekday monthname ordinal
                   | weekday ordinal 'of' monthname
                   | weekday ordinal 'of' monthname yearnumber
                   | weekday hourspec
                   | monthname
                   | monthname yearnumber
                   | monthname ordinal
                   | monthname ordinal yearnumber
                   | ordinal 'of' monthname
                   | ordinal 'of' monthname yearnumber
                   | yearnumber
                   | hourspec
                   | 'now'
                   | 'today'
                   | 'yesterday'
                   | 'tomorrow'
                   ;

    relative_spec := 'this' recurring_token
                   | 'next' recurring_token
                   | 'last' recurring_token
                   | [('the' | 'a' | small_int)] recurring_token relative_anchor
                   | 'in' ('a' | 'an' | small_int) recurring_token
                   ;

    recurring_token := 'second'
                     | 'minute'
                     | 'hour'
                     | 'day'
                     | 'month'
                     | 'year'
                     | 'week'
                     | 'weekend'
                     | weekday
                     | monthname
                     | ordinal 'of' monthname
                     | monthname ordinal
                     | weekday monthname ordinal
                     | weekday ordinal 'of' monthname
                     | hourspec
                     ;

    # TODO: group relative_spec, anchored_spec, scoped_spec into time_spec

    relative_anchor := 'ago'
                     | 'hence'
                     | 'before' 'last'
                     | 'before' relative_spec
                     | 'before' anchored_spec
                     | 'before' scoped_spec
                     | ('from' | 'after') 'next'
                     | ('from' | 'after') relative_spec
                     | ('from' | 'after') anchored_spec
                     | ('from' | 'after') scoped_spec
                     ;

    scoped_spec := ['the'] ordinal_qualifier recurring_token 'of' ['the'] scoped_anchor ;

    scoped_anchor := recurring_token
                   | scoped_spec
                   | anchored_spec
                   | relative_spec
                   ;

    ordinal_qualifier := 'next' | 'last' | ordinal | 'last' ordinal ;


    # TODO: comp-grain (3 days and 4 hours ago)
    # TODO: sequence 'until' time
    # TODO: sequence 'since' time
    # TODO: sequence 'between' time 'and' time
    "#
}

fn _grammar() -> Result<earlgrey::Grammar, String> {
    use crate::constants::*;
    use std::str::FromStr;
    earlgrey::EbnfGrammarParser::new(time_grammar(), "time_spec")
        .plug_terminal("weekday", |d| weekday(d).is_some())
        .plug_terminal("monthname", |d| month(d).is_some())
        .plug_terminal("ordinal", |d| {
            ordinal(d).or_else(|| short_ordinal(d)).is_some()
        })
        .plug_terminal("yearnumber", |y| {
            i32::from_str(y).map(|y| 999 < y && y < 3000).is_ok()
        })
        .plug_terminal("hourspec", |h| {
            h.len() > 2 && h.split_at_checked(h.len() - 2).map(|(h, ampm)| {
                (ampm == "am" || ampm == "pm")
                    && usize::from_str(h).map(|h| 1 <= h && h <= 12).is_ok()
            }) == Some(true)
        })
        .plug_terminal("small_int", |s| {
            usize::from_str(s).map(|s| s <= 999).is_ok()
        })
        // literlas that we want to check variations of
        .plug_terminal("second", |s| s == "second" || s == "seconds")
        .plug_terminal("minute", |s| s == "minute" || s == "minutes")
        .plug_terminal("hour", |s| s == "hour" || s == "hours")
        .plug_terminal("day", |s| s == "day" || s == "days")
        .plug_terminal("week", |s| s == "week" || s == "weeks")
        .plug_terminal("month", |s| s == "month" || s == "months")
        .plug_terminal("year", |s| s == "year" || s == "years")
        .into_grammar()
}

pub fn time_parser() -> earlgrey::EarleyParser {
    earlgrey::EarleyParser::new(
        _grammar().unwrap_or_else(|e| panic!("TimeMachine grammar BUG: {:?}", e)),
    )
}

pub fn debug_time_expression(time: &str) -> Result<Vec<earlgrey::Sexpr>, String> {
    let parser = earlgrey::sexpr_parser(
        _grammar().unwrap_or_else(|e| panic!("TimeMachine grammar BUG: {:?}", e)),
    )?;
    parser(time.split(&[' ', ','][..]).filter(|w| !w.is_empty()))
}

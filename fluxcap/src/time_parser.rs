// https://github.com/wit-ai/duckling_old/blob/master/resources/languages/en/corpus/time.clj
// https://github.com/wit-ai/duckling_old/blob/master/resources/languages/en/rules/time.clj

// TODO: types should be date-time or duration (not count and TimeEl)

pub fn time_grammar() -> &'static str {
    r#"
    time_expr := time_span
               | 'on' time_span
               ;

    time_span := explicit_span
               | sequence
               ;

    # These yield a single span
    explicit_span := 'now'
               | 'today'
               | 'yesterday'
               | 'tomorrow'
               | 'this' sequence
               | 'next' sequence
               | 'last' sequence
               | ['the' | 'a' | small_int] sequence relative_anchor
               | 'in' ('a' | 'an' | small_int) sequence
               | sequence yearnumber
               | yearnumber
               | ['the'] ordinal_qualifier sequence 'of' ['the'] explicit_span
               | 'since' time_span
               | 'until' time_span
               | 'between' time_span 'and' time_span
               | duration 'ago'
               | 'in' duration
               ;

    # A sequence yields a series of time spans of which some are selected
    sequence := time_quantity  # eg: 'day', 'week', 'month', etc.
              | 'weekend'
              | monthname
              | monthname ordinal
              | ordinal 'of' monthname
              | weekday
              | weekday monthname ordinal
              | weekday ordinal 'of' monthname
              | weekday hourspec
              | hourspec
              | ['the'] ordinal_qualifier sequence 'of' ['the'] sequence
              | ['the'] ordinal
              | weekday ['the'] ordinal
              ;

    duration := small_int time_quantity
              | 'a' time_quantity
              | duration 'and' small_int time_quantity
              | duration 'and' 'a' time_quantity
              ;

    relative_anchor := 'ago'
                     | 'hence'
                     | 'before' 'last'
                     | 'before' time_span
                     | ('from' | 'after') 'next'
                     | ('from' | 'after') time_span
                     ;

    ordinal_qualifier := 'next' | 'last' | ordinal | 'last' ordinal ;
    "#
}

fn _grammar() -> Result<earlgrey::Grammar, String> {
    use crate::constants::*;
    use std::str::FromStr;
    earlgrey::EbnfGrammarParser::new(time_grammar(), "time_expr")
        .plug_terminal("weekday", |d| weekday(d).is_some())
        .plug_terminal("monthname", |d| month(d).is_some())
        .plug_terminal("ordinal", |d| {
            ordinal(d).or_else(|| short_ordinal(d)).is_some()
        })
        .plug_terminal("yearnumber", |y| {
            i32::from_str(y).map(|y| 999 < y && y < 3000) == Ok(true)
        })
        .plug_terminal("hourspec", |h| hour_spec(h).is_some())
        .plug_terminal("small_int", |s| {
            u16::from_str(s).map(|s| s <= 999) == Ok(true)
        })
        // literlas that we want to check variations of
        .plug_terminal("time_quantity", |q| {
            kronos_grain(q).is_some() || q == "week" || q == "weeks"
        })
        .plug_terminal("weekend", |w| w == "weekend" || w == "weekends")
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

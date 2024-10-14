#![deny(warnings)]

// https://github.com/wit-ai/duckling_old/blob/master/resources/languages/en/corpus/time.clj
// https://github.com/wit-ai/duckling_old/blob/master/resources/languages/en/rules/time.clj

pub fn time_grammar() -> &'static str {
    r#"
    named_seq := day_ordinal
              | weekday
              | month
              | day_ordinal 'of' month
              | month day_ordinal
              | weekday day_ordinal
              | weekday day_ordinal 'of' month
              | weekday month day_ordinal
              | 'weekend' | 'weekends'
              | year
              ;

    sequence := named_seq | grain;

    comp_seq := ordinal sequence 'of' ['the'] @opt_the comp_seq
              | 'last' sequence 'of' ['the'] @opt_the comp_seq
              | sequence
              ;

    comp_grain := small_int grain
               | 'a' grain
               | comp_grain 'and' small_int grain
               | comp_grain 'and' 'a' grain
               ;

    time := 'today'
          | 'tomorrow'
          | 'yesterday'
          | 'on' weekday
          | named_seq

          | 'the' comp_seq
          | 'this' comp_seq
          | 'next' comp_seq
          | 'last' comp_seq

          | comp_seq 'after' 'next'
          | comp_seq 'before' 'last'

          | 'a' named_seq 'ago'
          |  small_int named_seq 'ago'
          | 'in' small_int named_seq

          | comp_grain 'ago'
          | 'in' comp_grain

          | month year
          | month day_ordinal year

          | comp_grain 'after' time
          | comp_grain 'before' time

          | sequence 'until' time
          | sequence 'since' time
          | sequence 'between' time 'and' time
          ;
    "#
}

fn _grammar() -> Result<earlgrey::Grammar, String> {
    use std::str::FromStr;
    use crate::constants::*;
    earlgrey::EbnfGrammarParser::new(time_grammar(), "time")
        .plug_terminal("ordinal", |d| ordinal(d).or_else(|| short_ordinal(d)).is_some())
        .plug_terminal("day_ordinal", |d| ordinal(d).or_else(|| short_ordinal(d)).is_some())
        .plug_terminal("weekday", |d| weekday(d).is_some())
        .plug_terminal("month", |d| month(d).is_some())
        .plug_terminal("grain", |g| kronos::Grain::from_str(g).is_ok())
        .plug_terminal("year", |y| if let Ok(year) = i32::from_str(y)
                       { year > 999 && year < 2200 } else { false })
        .plug_terminal("small_int", |u| if let Ok(u) = usize::from_str(u)
                       { u < 100 } else { false })
        .into_grammar()
}

pub fn time_parser() -> earlgrey::EarleyParser {
    earlgrey::EarleyParser::new(
        _grammar()
        .unwrap_or_else(|e| panic!("TimeMachine grammar BUG: {:?}", e))
    )
}

pub fn debug_time_expression(time: &str) -> Result<Vec<earlgrey::Sexpr>, String> {
    let parser = earlgrey::sexpr_parser(
        _grammar()
        .unwrap_or_else(|e| panic!("TimeMachine grammar BUG: {:?}", e))
    )?;
    parser(time.split(&[' ', ','][..]).filter(|w| !w.is_empty()))
}

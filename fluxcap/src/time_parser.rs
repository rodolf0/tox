#![deny(warnings)]

pub fn time_grammar() -> &'static str {
    r#"
    time_expr := time_span
               | 'on' time_span
               | sequence 'since' time_span
               | sequence 'until' time_span
               | sequence 'between' time_span 'and' time_span
               | sequence 'in' time_span
               ;

    time_span := explicit_span
               | sequence
               ;

    # A sequence yields a series of time spans of which some are selected
    sequence := time_quantity
              | named_sequence
              ;

    named_sequence := 'weekend'
              | monthname
              | monthname ordinal
              | ordinal 'of' monthname
              | weekday
              | weekday monthname ordinal
              | weekday ordinal 'of' monthname
              | clock_time
              | weekday clock_time
              | ['the'] ordinal_qualifier sequence 'of' ['the'] sequence
              | ['the'] ordinal
              | weekday ['the'] ordinal
              ;

    # These yield a single span
    explicit_span := 'now'
               | 'today'
               | 'yesterday'
               | 'tomorrow'
               | numeric_date
               | 'this' sequence
               | 'next' sequence
               | 'last' sequence
               | ['the' | 'a' | small_int] named_sequence relative_anchor
               | 'in' ('a' | 'an' | small_int) named_sequence
               | sequence yearnumber
               | yearnumber
               | ['the'] ordinal_qualifier sequence 'of' ['the'] explicit_span
               | sequence 'on' explicit_span
               | 'since' time_span
               | 'until' time_span
               | 'between' time_span 'and' time_span
               | duration shift_anchor
               | 'in' duration
               | ['the' | 'a' | small_int] time_quantity 'before' 'last'
               | ['the' | 'a' | small_int] time_quantity ('from' | 'after') 'next'
               ;

    duration := small_int time_quantity
              | ('a' | 'an') time_quantity
              | duration 'and' small_int time_quantity
              | duration 'and' ('a' | 'an') time_quantity
              ;

    relative_anchor := shift_anchor
                     | 'before' 'last'
                     | ('from' | 'after') 'next'
                     ;

    shift_anchor := 'ago'
                  | 'hence'
                  | 'before' time_span
                  | ('from' | 'after') time_span
                  ;

    ordinal_qualifier := 'next' | 'last' | ordinal | 'last' ordinal ;
    "#
}

pub fn debug_time_expression(time: &str) -> Result<Vec<earlgrey::Sexpr>, String> {
    let parser = earlgrey::sexpr_parser(time_grammar(), "time_expr")?;
    parser(time.split(&[' ', ','][..]).filter(|w| !w.is_empty()))
}

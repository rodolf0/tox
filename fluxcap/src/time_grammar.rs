#![deny(warnings)]

pub fn time_grammar() -> &'static str {
    r#"
    time_expr := time_span
               | 'on' time_span
               | sequence 'since' explicit_span
               | sequence 'since' sequence
               | sequence 'until' explicit_span
               | sequence 'until' sequence
               | sequence 'between' time_span 'and' time_span
               | sequence 'in' time_span
               ;

    time_span := explicit_span
               | sequence
               ;

    # A sequence yields a series of time spans of which some are selected
    sequence := time_quantity
              | small_int time_quantity
              | named_sequence
              | sequence 'at' clock_time
              | sequence part_of_day
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
              | iso_week
              | iso_quarter
              | iso_half
              | holiday
              | season
              | part_of_day
              | shorthand
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
               | explicit_span part_of_day
               | explicit_span clock_time
               | explicit_span 'at' clock_time
               | clock_time explicit_span
               | part_of_day explicit_span
               | ['the'] ordinal_qualifier sequence 'of' ['the'] explicit_span
               | sequence 'on' explicit_span
               | 'since' explicit_span
               | 'since' sequence
               | 'until' explicit_span
               | 'until' sequence
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

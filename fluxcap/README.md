# fluxcap

Natural language time expression parser inspired by duckling. 
Parses phrases like "next Tuesday at 3pm" or "3 weeks ago" into structured time spans.

## Quick Start

```rust
use fluxcap::TimeMachine;

let tm = TimeMachine::new();

// Parse a simple date
let results = tm.eval("next Tuesday", None).unwrap();
for r in results {
    println!("{:?}", r);
}

// Parse with a reference time (defaults to local time)
use time::macros::datetime;
let reftime = datetime!(2024-06-01 12:00 UTC);
let results = tm.eval("last month", Some(reftime)).unwrap();
for r in results {
    println!("{:?}", r);
}
```

## TimeResult

`TimeMachine::eval` returns a `Vec<TimeResult>`:

- `TimeResult::Span(TimeSpan)` — a time span with start, end, and grain
- `TimeResult::Count(CountResult)` — a count of how many times a sequence occurs within a span

```rust
let tm = TimeMachine::new();
let results = tm.eval("mondays since last month", None).unwrap();
for r in results {
    match r {
        fluxcap::TimeResult::Count(c) => {
            println!("{} full {} (total: {:.1})", c.full_spans, c.unit, c.total);
        }
        fluxcap::TimeResult::Span(s) => {
            println!("{:?}: {} -> {}", s.grain, s.start, s.end);
        }
    }
}
```

## Supported Expressions

**Anchored dates and times**
- `now`, `today`, `yesterday`, `tomorrow`
- `2024`, `january`, `january 2024`, `january 15th`, `15th of january`
- `monday`, `monday january 15th`, `monday 15th of january 2024`
- `3pm`, `15:30`, `monday 3pm`

**Relative expressions**
- `this week`, `next month`, `last year`
- `a day ago`, `3 weeks ago`, `in a month`, `in 2 years`
- `2 months before next year`, `3 days after january 1st`

**Scoped expressions**
- `the 2nd week of january`, `the last friday of the month`
- `the 3rd monday of this year`, `the 1st weekend of january 2020`

**Durations and shifts**
- `2 days and 3 hours ago`, `a week hence`
- `2 days before next month`, `3 days after 2 weeks ago`

**Holidays and special**
- `christmas`, `halloween`, `thanksgiving`, `new_years_day`
- `spring`, `summer`, `fall`, `winter`
- `morning`, `afternoon`, `evening`, `night`, `lunch`
- `eom` (end of month), `eoy` (end of year), `eod` (end of day)

**ISO**
- `q1`, `q2`, `q3`, `q4` (quarters)
- `h1`, `h2` (halves)
- `week_42` (ISO week)

## The `fluxcap` Binary

The crate ships with a command-line tool:

```
$ cargo run -p toxtools --bin fluxcap -- "next tuesday at 3pm"
(Day) 2026-06-23T00:00:00Z -> 2026-06-24T00:00:00Z
(Day) 2026-06-23T00:00:00Z -> 2026-06-24T00:00:00Z

$ cargo run -p toxtools --bin fluxcap -- -v "next tuesday"
(Sexpr output showing the parse tree)
```

## Debugging

Use `parse_sexpr` to inspect the raw parse tree:

```rust
let tm = TimeMachine::new();
let trees = tm.parse_sexpr("3 days ago").unwrap();
for t in trees {
    println!("{}", t.print());
}
```

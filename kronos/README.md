# kronos

A tool for calculating complex time expressions. Kronos composes `TimeSequence` 
iterators that represent infinite sequences into the past and future, anchored 
to a specific instant.

## Example

```rust
use kronos::TimeSeqSpec;

let t0 = time::macros::datetime!(2019-02-05 0:00).assume_utc();

let mondays = TimeSeqSpec::weekday(1);
for monday in mondays.future(t0).take(3) {
    println!("{:?}", monday);
}
```

## TimeSpan

A `TimeSequence` yields `TimeSpan` items — right-open intervals `[start, end)` 
with a `Grain` specifying the resolution.

```rust
let span = kronos::TimeSpan::year(2024);
println!("Year 2024: {} to {}", span.start, span.end);
```

## Basic Sequences

```rust
use kronos::TimeSeqSpec;

TimeSeqSpec::days();          // every day
TimeSeqSpec::weeks();         // every week
TimeSeqSpec::months(None);    // every month
TimeSeqSpec::years();         // every year
TimeSeqSpec::weekends();      // every weekend
TimeSeqSpec::weekday(1);      // every Monday (0=Sunday)
TimeSeqSpec::monthday(15);    // 15th of every month
TimeSeqSpec::hours(Some(9));  // 9:00 every day
```

## Composing Sequences

### Within — nth occurrence

```rust
// 3rd Monday of each month
let s = TimeSeqSpec::weekday(1)
    .within(TimeSeqSpec::months(None), 3)
    .unwrap();

// Last day of February
let s = TimeSeqSpec::days()
    .within(TimeSeqSpec::months(Some(2)), -1)
    .unwrap();
```

### Union, Intersection, Except

```rust
// Mondays and Wednesdays
let s = TimeSeqSpec::weekday(1)
    .union(TimeSeqSpec::weekday(3))
    .unwrap();

// Mondays of June
let s = TimeSeqSpec::months(Some(6))
    .intersection(TimeSeqSpec::weekday(1));

// Every day except Friday
let s = TimeSeqSpec::days()
    .except(TimeSeqSpec::weekday(5));
```

### Shift

```rust
// 3 days after Monday Feb 28th
let s = TimeSeqSpec::days()
    .within(TimeSeqSpec::months(Some(2)), 28).unwrap()
    .intersection(TimeSeqSpec::weekday(1))
    .shift(kronos::Grain::Day, 3);
```

### Interval

```rust
// Spring in the southern hemisphere (Sep 21 to Dec 21 inclusive)
let spring = TimeSeqSpec::days()
    .within(TimeSeqSpec::months(Some(9)), 21).unwrap()
    .to(
        TimeSeqSpec::days()
            .within(TimeSeqSpec::months(Some(12)), 21).unwrap(),
        true, // inclusive
    );
```

### Merge

```rust
// 2-day periods
let s = TimeSeqSpec::days().merge(2);
```

## Iteration

```rust
use kronos::TimeSeqSpec;

let s = TimeSeqSpec::weekday(1);
let t0 = time::macros::datetime!(2019-02-05 0:00).assume_utc();

// Default: include the span containing t0
s.future(t0)          // forward
s.past(t0)            // backward

// Strict variants
s.strict_future(t0)   // exclude spans that start before or at t0
s.inclusive_past(t0)  // include spans that contain t0
```

## TimeSpan Operations

```rust
use kronos::{TimeSpan, Grain};

let span = TimeSpan::year(2024);
let shifted = span.shift(Grain::Year, 1); // Year 2025

let span2 = TimeSpan::year(2024);
let overlap = span.intersect(&span2);
let contains = span.contains(&some_time);
```

## References

* http://homes.cs.washington.edu/~kentonl/pub/ladz-acl.2014.pdf
* https://github.com/wit-ai/duckling_old

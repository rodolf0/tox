# Documentation

Kronos is a tool for calculating date/times. It is meant to give a concrete date for questions like *"When is the 2nd Monday of June?"*, or *"What was the 3rd to last day-of-the-week of past February?"*.

To answer these questions kronos composes `TimeSequence` iterators. These are infinite sequences into the past and the future which you can pin to a particular instant and get resulting time `Range`s.

## Example

Lets first define a `TimeSequence` that represents any and all *Monday*s. Then use it to get an iterator of all future Mondays from a specific **t0** instant onward.
```rust
// Reference time: Tuesday, 5th Feb 2019
let t0 = chrono::NaiveDate::from_ymd(2019, 2, 5)
  .and_hms(0, 0, 0);

// A sequence for *Mondays*
let mondays = kronos::Weekday(1);

// First Monday after t0 reference time
mondays.future(&t0).next()
```

## Ranges

The previous example would return a `Range` which represents an open-ended time interval `[start, end)`. Ranges also have a `Grain` that specifies the resolution of the start and end instants.

Examples of a Range could be Aug 26th 2018.
- It has a resolution of Grain::Day,
- starts on 2018/08/26 00:00:00,
- ends on 2018/08/27 00:00:00

## Composing `TimeSequence`s

### Basic sequences

Some simple TimeSequences can be built almost out of thin air. For example:
- `Weekday(2)` a sequence for Tuesdays.
- `Month(6)` a sequence for all June months.
- `Grains(Grain::Day)` a sequence to iterate over days.

### Composite sequences

More comples TimeSequences can be created by combining other sequences. For example:
- `NthOf(2, Weekday(1), Month(6))` creates a sequence for "the second Mondays of June".
- `LastOf(1, Weekend, Grains(Grain::Year))` for "last weekend of the year".
- `Intersect(Weekday(1), NthOf(28, Grains(Grain::Day), Grains(Grains::Month)))` for "all Monday 28th".

Other compositions allow unions, intersections, intervals, exceptions, etc. Please check each module's tests for [examples](https://github.com/rodolf0/tox/tree/master/kronos/src) on how to use them.


#### References
* http://homes.cs.washington.edu/~kentonl/pub/ladz-acl.2014.pdf
- https://github.com/wit-ai/duckling_old/blob/6b7e2e1bdbd50299cee4075ff48d7323c05758bc/src/duckling/time/pred.clj#L57-L72
- https://github.com/wit-ai/duckling_old/blob/6b7e2e1bdbd50299cee4075ff48d7323c05758bc/src/duckling/time/pred.clj#L333

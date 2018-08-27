# kronos

Kronos is a crate to build *time* iterators. See examples below.

### Range
A *Range* is an open-ended time interval `[start, end)` that defines some concrete time. Ranges have  an accompanying grain that specifies the resolution of the start and end instants defining the time range.

Example of a Range could be Aug 26th 2018.
- It has a resolution of Grain::Day,
- starts on 2018/08/26 00:00:00,
- ends on 2018/08/27 00:00:00 (open ended means this instant is not included).


### Grain
*Grain*s are used either to build time sequences or to inform of resolution of Range elements.


### TimeSequence
Any type that implements *TimeSequence* can be used to emit *Range* elements into the future or the past taking some `t0` as a reference starting instant.

There are multiple ways to create a time-sequence:
- `Weekday(1)` creates a sequence for Mondays.
- `Month(6)` creates a sequence for June.
- `NthOf(2, Weekday(1), Month(6))` creates a sequence for "the 2nd Mondays of June".
- `LastOf(1, Weekend, Grains(Grain::Year))` for "last weekend of the year".
- `Intersect(Weekday(1), NthOf(28, Grains(Grain::Day), Grains(Grains::Month)))` for "all Monday 28th".

Once you have a *TimeSequence* you can use it to emit actual *Range*s. For Example:

```
let day2 = NthOf(2, Grains(Grain::Day), Grains(Grain::Month));
let third_hour_of_2nd_day = NthOf(3, Grains(Grain::Hour), day2);

let t0 = types::Date::from_ymd(2018, 6, 18).and_hms(0, 0, 0);
let mut future = third_hour_of_2nd_day.future(&t0);

eprintln!("{}", future.next().unwrap());
```


### references
* https://duckling.wit.ai/
* http://homes.cs.washington.edu/~kentonl/pub/ladz-acl.2014.pdf
- https://github.com/wit-ai/duckling_old/blob/6b7e2e1bdbd50299cee4075ff48d7323c05758bc/src/duckling/time/pred.clj#L57-L72
- https://github.com/wit-ai/duckling_old/blob/6b7e2e1bdbd50299cee4075ff48d7323c05758bc/src/duckling/time/pred.clj#L333

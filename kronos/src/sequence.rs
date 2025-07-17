// A TimeSequence is a generator of TimeRanges
//   a |----|----|----|----|----|----|----|----|
//
// A TimeRange is a right-open time interval. [----)
// The grain determines up to what granularity start/end timepoints are valid.
//
// To make a sequence generate TimeRange items the generator
// needs to be anchored at a point in time t0.
//
//     s0   s1   s2   s3   s4 ....
//   a |----|----|----|----|----|----|----|----|
//        ^
//        | t0
//
// The 1st TimeRange item in the sequence will contain t0 unless
// its impossible because of a non-complete sequence.
// - Iterating into the future the the start of the first element will be greater than t0.
// - Iterating into the past the the start of the first element will be greater than t0 ?
// TODO: add utility method on TimeRange to check if a timepoint is contained.
// TODO: add utility method to shift TimeRange by an amount.
//
// When iterating into the past. The same applies t0 will be contained by the
// first emitted TimeRange (unless impossible because of a non-complete sequence).
//
// Given right-open intervals, if t0 aligns with the end of a sequence (hence the
// start of the next) the 1st element will seem like an item into the future.
// The user can restrict the 1st item to be in the past by checking end <= t0.
//
// Iterating into the past and future will overlap on their 1st element. The user
// can choose to ignore this overlap by checking if t0 is contained and skip.
//
//
// # Operations combining sequences
//
// Different overlap types between 2 sequences:
//
//   [------a------)
//            [------b------)
//
//   [----a----)
//                [-----b-----)
//
//   [---------a---------)
//        [-----b-----)
//
// - and - Intersection: eg June and summer, 3pm on Saturdays
// - or - Union: eg Mondays and Tuesdays, 1st and 3rd day of the month
// - diff - Except: Days in June except Mondays, Tuesdays except March
// - Within: 3rd monday of May, Last hour of the day
// - apply - first do this ... graph traversal ?
//
//
// An anchored TimeSequence implements the Iterator trait and you
// can transform items as needed with iterator methods.

use time::{Duration, PrimitiveDateTime as DateTime};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Grain {
    Second,
    Minute,
    Hour,
    Day,
    Week,
    Month,
    Year,
}

#[derive(Debug, PartialEq)]
pub struct TimeRange {
    start: DateTime,
    end: DateTime,
    grain: Grain,
}

fn shift(t0: DateTime, grain: Grain, n: i32) -> Result<DateTime, String> {
    if n == 0 {
        return Ok(t0);
    }
    match grain {
        Grain::Second => Ok(t0 + Duration::seconds(n as i64)),
        Grain::Minute => Ok(t0 + Duration::minutes(n as i64)),
        Grain::Hour => Ok(t0 + Duration::hours(n as i64)),
        Grain::Day => Ok(t0 + Duration::days(n as i64)),
        Grain::Week => Ok(t0 + Duration::weeks(n as i64)),
        Grain::Month => {
            let (year, month, day) = t0.to_calendar_date();
            // Work in month space
            let zero_indexed_month = month as i32 - 1;
            let total_months = year * 12 + zero_indexed_month + n;
            // Convert back to year and month
            let n_year = total_months.div_euclid(12);
            let n_zero_month = total_months.rem_euclid(12);
            let n_month = time::Month::try_from(n_zero_month as u8 + 1)
                .map_err(|e| format!("Invalid month: {}", e))?;
            // Check if the day needs to be clamped because of month change
            let n_day = day.min(n_month.length(n_year));
            Ok(t0.replace_date(
                time::Date::from_calendar_date(n_year, n_month, n_day)
                    .map_err(|e| format!("Invalid date: {}", e))?,
            ))
        }
        Grain::Year => t0
            .replace_year(t0.year() + n)
            .map_err(|e| format!("Invalid year: {}", e)),
    }
}

// Generate sequences of time ranges. Each element will have window_span template.
fn grain_iterator(
    t0: DateTime,
    window_span: (Grain, u32),
    step_by: (Grain, i32),
) -> impl Iterator<Item = TimeRange> {
    (0..).map(move |i| {
        let start = shift(t0, step_by.0, i * step_by.1).unwrap();
        let end = shift(start, window_span.0, window_span.1 as i32).unwrap();
        TimeRange {
            start,
            end,
            grain: window_span.0,
        }
    })
}

pub enum TimeSequence {
    Seconds,
    Days,
    Weekdays(u8), // Sunday=0
    Weekends,
    Months,
    Month(u8),
    Within(isize, Box<TimeSequence>, Box<TimeSequence>),
    Union(Box<TimeSequence>, Box<TimeSequence>),
    Intersection(Box<TimeSequence>, Box<TimeSequence>),
}

fn find_month(mut t0: DateTime, month: u8) -> Result<DateTime, String> {
    while t0.month() as u8 != month {
        t0 = shift(t0, Grain::Month, 1)?;
    }
    Ok(t0)
}

fn find_weekend(mut t0: DateTime) -> Result<DateTime, String> {
    while t0.weekday() != time::Weekday::Saturday && t0.weekday() != time::Weekday::Sunday {
        t0 = shift(t0, Grain::Day, 1)?;
    }
    Ok(t0)
}

fn find_weekday(mut t0: DateTime, weekday: u8) -> Result<DateTime, String> {
    while t0.weekday().number_days_from_sunday() != weekday {
        t0 = shift(t0, Grain::Day, 1)?;
    }
    Ok(t0)
}

impl TimeSequence {
    pub fn days() -> TimeSequence {
        TimeSequence::Days
    }

    pub fn weekends() -> TimeSequence {
        TimeSequence::Weekends
    }

    pub fn months() -> TimeSequence {
        TimeSequence::Months
    }

    pub fn month(month: u8) -> TimeSequence {
        TimeSequence::Month(month)
    }

    pub fn or(self, other: Self) -> Self {
        Self::Union(Box::new(self), Box::new(other))
    }

    pub fn within(self, frame: Self, n: isize) -> Self {
        Self::Within(n, Box::new(self), Box::new(frame))
    }

    pub fn future(&self, t0: DateTime) -> Result<Box<dyn Iterator<Item = TimeRange> + '_>, String> {
        use TimeSequence::*;
        Ok(match self {
            Seconds => Box::new(grain_iterator(t0, (Grain::Second, 1), (Grain::Second, 1))),
            Days => Box::new(grain_iterator(t0, (Grain::Day, 1), (Grain::Day, 1))),
            Weekdays(n) => {
                let t0 = find_weekday(t0, *n)?;
                Box::new(grain_iterator(t0, (Grain::Day, 1), (Grain::Day, 7))),
            }
            Weekends => {
                let t0 = find_weekend(t0)?;
                Box::new(grain_iterator(t0, (Grain::Day, 2), (Grain::Day, 7)))
            }
            Month(n) => {
                let t0 = find_month(t0, *n)?;
                Box::new(grain_iterator(t0, (Grain::Month, 1), (Grain::Month, 12)))
            }
            Within(n, window_spec, frame_spec) => {
                // TODO: check that window.grain < frame.grain Or will just getting None be it ?
                Box::new(
                    frame_spec
                        .future(t0)?
                        // .inspect(|x| println!("Frame: {:?}", x))
                        .filter_map(|frame| {
                            window_spec
                                .future(frame.start)
                                .unwrap() // TODO: this can fail at run-time (ie calling next)
                                // Should 'future' items be Results instead of TimeRange ?
                                // .inspect(|x| println!("Window: {:?}", x))
                                // Window has to start within frame's boundary
                                .take_while(|w| w.start < frame.end)
                                .nth((*n - 1) as usize) // TODO: lastof for negative
                        }),
                )
            }
            _ => todo!(), // Union(spec1, spec2) => Sequence::Union(Box::new(spec1), Box::new(spec2)),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use time::macros::datetime;

    #[test]
    fn test_weekend() -> Result<(), String> {
        // The 3rd weekend of june
        let seq = TimeSequence::weekends().within(TimeSequence::month(6), 3);
        let mut sequence = seq.future(datetime!(2025-07-01 0:00))?;
        assert_eq!(
            sequence.next().unwrap(),
            TimeRange {
                start: datetime!(2026-06-20 0:00),
                end: datetime!(2026-06-22 0:00),
                grain: Grain::Day,
            }
        );
        Ok(())
    }
}

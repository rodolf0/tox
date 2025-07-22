// A TimeSeq is a generator of TimeRanges
//   a |----|----|----|----|----|----|----|----|
//
// A TimeSpan is a right-open time interval. [----)
// The grain determines up to what granularity start/end timepoints are valid.
//
// To make a sequence generate TimeSpan items the generator
// needs to be anchored at a point in time t0.
//
//     s0   s1   s2   s3   s4 ....
//   a |----|----|----|----|----|----|----|----|
//        ^
//        | t0
//
// The 1st TimeSpan item in the sequence will contain t0 unless
// its impossible because of a non-complete sequence.
// - Iterating into the future the the start of the first element will be greater than t0.
// - Iterating into the past the the start of the first element will be greater than t0 ?
// TODO: add utility method on TimeSpan to check if a timepoint is contained.
// TODO: add utility method to shift TimeSpan by an amount.
//
// When iterating into the past. The same applies t0 will be contained by the
// first emitted TimeSpan (unless impossible because of a non-complete sequence).
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
// An anchored TimeSeq implements the Iterator trait and you
// can transform items as needed with iterator methods.

use std::collections::VecDeque;

use time::{Duration, PrimitiveDateTime as DateTime};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub(crate) enum Grain {
    Second,
    Minute,
    Hour,
    Day,
    Month,
    Year,
}

#[derive(Debug, PartialEq)]
pub struct TimeSpan {
    pub(crate) start: DateTime,
    pub(crate) end: DateTime,
    pub(crate) grain: Grain,
}

// Shift a DateTime by a given number of grain-counts.
// Day of the month may be clamped in case of month change or leap year.
fn shift(t0: DateTime, grain: Grain, n: i32) -> DateTime {
    if n == 0 {
        return t0;
    }
    match grain {
        Grain::Second => t0 + Duration::seconds(n as i64),
        Grain::Minute => t0 + Duration::minutes(n as i64),
        Grain::Hour => t0 + Duration::hours(n as i64),
        Grain::Day => t0 + Duration::days(n as i64),
        Grain::Month => {
            let (year, month, day) = t0.to_calendar_date();
            // Work in month space
            let zero_indexed_month = month as i32 - 1;
            let total_months = year * 12 + zero_indexed_month + n;
            // Convert back to year and month
            let n_year = total_months.div_euclid(12);
            let n_zero_month = total_months.rem_euclid(12);
            let n_month = time::Month::try_from(n_zero_month as u8 + 1).expect("BUG: bad month");
            // Check if the day needs to be clamped because of month change
            let n_day = day.min(n_month.length(n_year));
            t0.replace_date(
                time::Date::from_calendar_date(n_year, n_month, n_day).expect("BUG: bad date"),
            )
        }
        Grain::Year => {
            // Eg: for Feb 29th max day needs adjustment.
            let max_month_days = t0.month().length(t0.year() + n);
            t0.replace_day(t0.day().min(max_month_days))
                .and_then(|t| t.replace_year(t0.year() + n))
                .expect(&format!(
                    "BUG: bad year shift t0={}, n={}, g={:?}",
                    t0, n, grain
                ))
        }
    }
}

fn truncate(t0: DateTime, grain: Grain) -> DateTime {
    use Grain::*;
    match grain {
        Second => Ok(t0),
        Minute => t0.replace_second(0),
        Hour => t0.replace_minute(0).and_then(|t| t.replace_second(0)),
        Day => Ok(t0.replace_time(time::Time::MIDNIGHT)),
        Month => t0.replace_time(time::Time::MIDNIGHT).replace_day(1),
        Year => t0
            .replace_time(time::Time::MIDNIGHT)
            .replace_day(1)
            .and_then(|t| t.replace_month(time::Month::January)),
    }
    // The rounding in truncate should never be invalid.
    .expect("BUG: truncation failed")
}

// Generate sequences of time ranges. Each element will have window_span template.
fn grain_iterator(
    t0: DateTime,
    window_span: (Grain, u32),
    step_by: (Grain, i32),
) -> impl Iterator<Item = TimeSpan> {
    // Mask t0's resolution beyond the grain of the span
    let t0 = truncate(t0, window_span.0);
    (0..).map(move |i| {
        let start = shift(t0, step_by.0, i * step_by.1);
        let end = shift(start, window_span.0, window_span.1 as i32);
        TimeSpan {
            start,
            end,
            grain: window_span.0,
        }
    })
}

pub enum TimeSeq {
    // Grain {
    //     window_span: (Grain, u32),
    //     step_by: (Grain, i32),
    // },
    Seconds,
    // Minutes,
    // Hours,
    Days,
    Weekdays(u8), // Sunday=0
    Weekends,
    Weeks,
    Months,
    Month(u8),
    Years,
    Within {
        nth: isize,
        window: Box<TimeSeq>,
        frame: Box<TimeSeq>,
    },
    Union(Box<TimeSeq>, Box<TimeSeq>),
    Intersection(Box<TimeSeq>, Box<TimeSeq>),
}

fn find_month(mut t0: DateTime, month: u8) -> DateTime {
    while t0.month() as u8 != month {
        t0 = shift(t0, Grain::Month, 1);
    }
    t0
}

fn find_weekend(mut t0: DateTime) -> DateTime {
    while t0.weekday() != time::Weekday::Saturday && t0.weekday() != time::Weekday::Sunday {
        t0 = shift(t0, Grain::Day, 1);
    }
    t0
}

fn find_weekday(mut t0: DateTime, weekday: u8) -> DateTime {
    while t0.weekday().number_days_from_sunday() != weekday {
        t0 = shift(t0, Grain::Day, 1);
    }
    t0
}

// Guard against impossible sequences, eg: 32nd day of the month
const INFINITE_FUSE: usize = 1000;

enum TimeDir {
    Future,
    Past,
}

impl TimeSeq {
    pub fn seconds() -> TimeSeq {
        TimeSeq::Seconds
    }

    pub fn days() -> TimeSeq {
        TimeSeq::Days
    }

    pub fn weekends() -> TimeSeq {
        TimeSeq::Weekends
    }

    pub fn months() -> TimeSeq {
        TimeSeq::Months
    }

    pub fn month(month: u8) -> TimeSeq {
        TimeSeq::Month(month)
    }

    pub fn years() -> TimeSeq {
        TimeSeq::Years
    }

    pub fn or(self, other: Self) -> Self {
        Self::Union(Box::new(self), Box::new(other))
    }

    pub fn within(self, frame: Self, nth: isize) -> Self {
        Self::Within {
            nth,
            window: Box::new(self),
            frame: Box::new(frame),
        }
    }

    pub fn grain(&self) -> Grain {
        match self {
            TimeSeq::Seconds => Grain::Second,
            TimeSeq::Days => Grain::Day,
            TimeSeq::Weekdays(_) => Grain::Day,
            TimeSeq::Weekends => Grain::Day,
            TimeSeq::Month(_) => Grain::Month,
            TimeSeq::Months => Grain::Month,
            // TimeSeq::Union(left, right) => {
            //     let left_grain = left.grain();
            //     let right_grain = right.grain();
            //     if left_grain == right_grain {
            //         left_grain
            //     } else {
            //         Grain::Second
            //     }
            // }
            TimeSeq::Within {
                nth: _,
                window,
                frame: _,
            } => window.grain(),
            _ => todo!(),
        }
    }

    fn seq(&self, t0: DateTime, direction: TimeDir) -> Box<dyn Iterator<Item = TimeSpan> + '_> {
        use TimeSeq::*;
        match self {
            Seconds => {
                let step_by = match direction {
                    TimeDir::Future => (Grain::Second, 1),
                    TimeDir::Past => (Grain::Second, -1),
                };
                Box::new(grain_iterator(t0, (Grain::Second, 1), step_by))
            }
            Days => {
                let step_by = match direction {
                    TimeDir::Future => (Grain::Day, 1),
                    TimeDir::Past => (Grain::Day, -1),
                };
                Box::new(grain_iterator(t0, (Grain::Day, 1), step_by))
            }
            Weeks => {
                let step_by = match direction {
                    TimeDir::Future => (Grain::Day, 7),
                    TimeDir::Past => (Grain::Day, -7),
                };
                Box::new(grain_iterator(t0, (Grain::Day, 7), step_by))
            }
            Weekdays(n) => {
                let t0 = find_weekday(t0, *n);
                let step_by = match direction {
                    TimeDir::Future => (Grain::Day, 7),
                    TimeDir::Past => (Grain::Day, -7),
                };
                Box::new(grain_iterator(t0, (Grain::Day, 1), step_by))
            }
            Weekends => {
                let t0 = find_weekend(t0);
                let step_by = match direction {
                    TimeDir::Future => (Grain::Day, 7),
                    TimeDir::Past => (Grain::Day, -7),
                };
                Box::new(grain_iterator(t0, (Grain::Day, 2), step_by))
            }
            Month(n) => {
                let t0 = find_month(t0, *n);
                let step_by = match direction {
                    TimeDir::Future => (Grain::Month, 12),
                    TimeDir::Past => (Grain::Month, -12),
                };
                Box::new(grain_iterator(t0, (Grain::Month, 1), step_by))
            }
            Months => {
                let step_by = match direction {
                    TimeDir::Future => (Grain::Month, 1),
                    TimeDir::Past => (Grain::Month, -1),
                };
                Box::new(grain_iterator(t0, (Grain::Month, 1), step_by))
            }
            Years => {
                let step_by = match direction {
                    TimeDir::Future => (Grain::Year, 1),
                    TimeDir::Past => (Grain::Year, -1),
                };
                Box::new(grain_iterator(t0, (Grain::Year, 1), step_by))
            }
            Within { nth, window, frame } if *nth > 0 => {
                // TODO: check that window.grain < frame.grain Or will just getting None be it ?
                Box::new(
                    frame
                        .seq(t0, direction)
                        // .inspect(|f| println!("Frame: {:?}", f))
                        .take(INFINITE_FUSE)
                        .filter_map(|f| {
                            window
                                .seq(f.start, TimeDir::Future)
                                // .inspect(|w| println!("Win: {:?}", w))
                                // Each window has to start within frame's boundary
                                .take_while(|w| w.start < f.end)
                                .nth((*nth - 1) as usize)
                        }),
                )
            }
            Within { nth, window, frame } if *nth < 0 => {
                let nth = -(*nth) as usize;
                // TODO: check that window.grain < frame.grain Or will just getting None be it ?
                Box::new(
                    frame
                        .seq(t0, direction)
                        // .inspect(|f| println!("Frame: {:?}", f))
                        .take(INFINITE_FUSE)
                        .filter_map(move |f| {
                            let mut deque = VecDeque::with_capacity(nth);
                            // Consume the whole iterator and keep the tail.
                            for w in window
                                .seq(f.start, TimeDir::Future)
                                // Each window has to start within frame's boundary
                                .take_while(|w| w.start < f.end)
                            {
                                deque.truncate(nth - 1);
                                deque.push_front(w);
                            }
                            // Not poping the back because it may be shorter than nth.
                            deque.remove(nth - 1)
                        }),
                )
            }
            _ => todo!(), // Union(spec1, spec2) => Sequence::Union(Box::new(spec1), Box::new(spec2)),
        }
    }

    pub fn future(&self, t0: DateTime) -> Box<dyn Iterator<Item = TimeSpan> + '_> {
        Box::new(
            self.seq(t0, TimeDir::Future)
                .skip_while(move |t| t.end <= t0),
        )
    }

    pub fn past(&self, t0: DateTime) -> Box<dyn Iterator<Item = TimeSpan> + '_> {
        Box::new(self.seq(t0, TimeDir::Past).skip_while(move |t| t.end > t0))
    }
}

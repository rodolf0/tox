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
pub enum Grain {
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

fn truncate_week(t0: DateTime) -> DateTime {
    let t0 = t0.replace_time(time::Time::MIDNIGHT);
    let week_days_offset = t0.weekday().number_days_from_sunday();
    t0 - Duration::days(week_days_offset as i64)
}

fn truncate_weekend(t0: DateTime) -> DateTime {
    // Make sure find_weekend was called before truncate_weekend
    assert!(
        t0.weekday() != time::Weekday::Saturday || t0.weekday() != time::Weekday::Sunday,
        "truncate_weekend needs to be called on a weekend "
    );
    let t0 = t0.replace_time(time::Time::MIDNIGHT);
    match t0.weekday() {
        time::Weekday::Saturday => t0,
        time::Weekday::Sunday => t0 - Duration::days(1),
        _ => unreachable!(),
    }
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
    Grain {
        window_span: (Grain, u32),
        step_by: (Grain, i32),
    },
    SpecificGrain {
        grain: Grain,
        n: u16,
    },
    Monthdays(u8),
    Weekdays(u8), // Sunday=0
    Weekends,
    Weeks,
    Within {
        nth: isize,
        window: Box<TimeSeq>,
        frame: Box<TimeSeq>,
    },
    Merge(Box<TimeSeq>, u8), // merge multiple spans into one
    Union(Box<TimeSeq>, Box<TimeSeq>),
    Intersection(Box<TimeSeq>, Box<TimeSeq>),
}

fn grains_in_parent(grain: Grain) -> u16 {
    match grain {
        Grain::Second => 60,
        Grain::Minute => 60,
        Grain::Hour => 24,
        Grain::Day => panic!("Day is not a valid grain for grains_in_parent"),
        Grain::Month => 12,
        Grain::Year => panic!("Year is not a valid grain for grains_in_parent"),
    }
}

fn find_grain(mut t0: DateTime, grain: Grain, n: u16) -> DateTime {
    while match grain {
        Grain::Second => t0.second() as u16,
        Grain::Minute => t0.minute() as u16,
        Grain::Hour => t0.hour() as u16,
        Grain::Day => t0.day() as u16,
        Grain::Month => t0.month() as u16,
        Grain::Year => panic!("Year is not a valid grain for find_grain"),
    } != n
    {
        t0 = shift(t0, grain, 1);
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

#[derive(Clone, Copy)]
enum TimeDir {
    Future,
    Past,
}

impl TimeSeq {
    pub fn seconds(n: Option<u16>) -> TimeSeq {
        match n {
            Some(n) => TimeSeq::SpecificGrain {
                grain: Grain::Second,
                n,
            },
            None => TimeSeq::Grain {
                window_span: (Grain::Second, 1),
                step_by: (Grain::Second, 1),
            },
        }
    }

    pub fn minutes(n: Option<u16>) -> TimeSeq {
        match n {
            Some(n) => TimeSeq::SpecificGrain {
                grain: Grain::Minute,
                n,
            },
            None => TimeSeq::Grain {
                window_span: (Grain::Minute, 1),
                step_by: (Grain::Minute, 1),
            },
        }
    }

    pub fn hours(n: Option<u16>) -> TimeSeq {
        match n {
            Some(n) => TimeSeq::SpecificGrain {
                grain: Grain::Hour,
                n,
            },
            None => TimeSeq::Grain {
                window_span: (Grain::Hour, 1),
                step_by: (Grain::Hour, 1),
            },
        }
    }

    pub fn days() -> TimeSeq {
        TimeSeq::Grain {
            window_span: (Grain::Day, 1),
            step_by: (Grain::Day, 1),
        }
    }

    pub fn months(n: Option<u16>) -> TimeSeq {
        match n {
            Some(n) => TimeSeq::SpecificGrain {
                grain: Grain::Month,
                n,
            },
            None => TimeSeq::Grain {
                window_span: (Grain::Month, 1),
                step_by: (Grain::Month, 1),
            },
        }
    }

    pub fn weeks() -> TimeSeq {
        TimeSeq::Weeks
    }

    pub fn years() -> TimeSeq {
        TimeSeq::Grain {
            window_span: (Grain::Year, 1),
            step_by: (Grain::Year, 1),
        }
    }

    pub fn weekends() -> TimeSeq {
        TimeSeq::Weekends
    }

    pub fn weekday(day: u8) -> TimeSeq {
        TimeSeq::Weekdays(day)
    }

    pub fn monthday(day: u8) -> TimeSeq {
        TimeSeq::Monthdays(day)
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

    pub fn merge(self, n: u8) -> Self {
        Self::Merge(Box::new(self), n)
    }

    fn grain(&self) -> Grain {
        match self {
            TimeSeq::Grain {
                window_span: (grain, _),
                step_by: _,
            } => *grain,
            TimeSeq::SpecificGrain { grain, n: _ } => *grain,
            TimeSeq::Weekdays(_) => Grain::Day,
            TimeSeq::Monthdays(_) => Grain::Day,
            TimeSeq::Weekends => Grain::Day,
            TimeSeq::Within {
                nth: _,
                window,
                frame: _,
            } => window.grain(),
            // TimeSeq::Union(left, right) => {
            //     let left_grain = left.grain();
            //     let right_grain = right.grain();
            //     if left_grain == right_grain {
            //         left_grain
            //     } else {
            //         Grain::Second
            //     }
            // }
            _ => todo!(),
        }
    }

    fn seq(&self, t0: DateTime, direction: TimeDir) -> Box<dyn Iterator<Item = TimeSpan> + '_> {
        match self {
            TimeSeq::Grain {
                window_span,
                step_by: (sb_grain, sb_n),
            } => {
                let step = match direction {
                    TimeDir::Future => *sb_n,
                    TimeDir::Past => -*sb_n,
                };
                Box::new(grain_iterator(t0, *window_span, (*sb_grain, step)))
            }
            TimeSeq::Weeks => {
                let t0 = truncate_week(t0);
                let step_by = match direction {
                    TimeDir::Future => (Grain::Day, 7),
                    TimeDir::Past => (Grain::Day, -7),
                };
                Box::new(grain_iterator(t0, (Grain::Day, 7), step_by))
            }
            TimeSeq::Weekdays(n) => {
                let t0 = find_weekday(t0, *n);
                let step_by = match direction {
                    TimeDir::Future => (Grain::Day, 7),
                    TimeDir::Past => (Grain::Day, -7),
                };
                Box::new(grain_iterator(t0, (Grain::Day, 1), step_by))
            }
            TimeSeq::Monthdays(n) => {
                let mut t0_end = t0;
                Box::new(std::iter::from_fn(move || {
                    while t0_end.day() != *n {
                        t0_end += match direction {
                            TimeDir::Future => Duration::days(1),
                            TimeDir::Past => Duration::days(-1),
                        }
                    }
                    t0_end += match direction {
                        TimeDir::Future => Duration::days(1),
                        TimeDir::Past => Duration::days(-1),
                    };
                    Some(TimeSpan {
                        start: t0_end + Duration::days(-1),
                        end: t0_end,
                        grain: Grain::Day,
                    })
                }))
            }
            TimeSeq::Weekends => {
                let t0 = find_weekend(t0);
                let t0 = truncate_weekend(t0);
                let step_by = match direction {
                    TimeDir::Future => (Grain::Day, 7),
                    TimeDir::Past => (Grain::Day, -7),
                };
                Box::new(grain_iterator(t0, (Grain::Day, 2), step_by))
            }
            TimeSeq::SpecificGrain { grain, n } => {
                let t0 = find_grain(t0, *grain, *n);
                let step_by = match direction {
                    TimeDir::Future => (*grain, grains_in_parent(*grain) as i32),
                    TimeDir::Past => (*grain, -(grains_in_parent(*grain) as i32)),
                };
                Box::new(grain_iterator(t0, (*grain, 1), step_by))
            }
            TimeSeq::Within { nth, window, frame } => {
                // TODO: check that window.grain < frame.grain Or will just getting None be it ?
                Box::new(
                    frame
                        .seq(t0, direction)
                        .take(INFINITE_FUSE)
                        .filter_map(|f| {
                            if *nth > 0 {
                                window
                                    .seq(f.start, TimeDir::Future)
                                    // Each window has to start within frame's boundary
                                    .take_while(|w| w.start < f.end)
                                    .nth((*nth - 1) as usize)
                            } else {
                                let nth = -(*nth) as usize;
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
                            }
                        }),
                )
            }
            TimeSeq::Merge(s, n) => {
                let mut _s = match direction {
                    TimeDir::Future => s.seq(t0, direction).skip(0),
                    TimeDir::Past => s.seq(t0, direction).skip(1),
                };
                Box::new(std::iter::from_fn(move || {
                    let _s2 = _s.by_ref();
                    let spans: Vec<_> = _s2.take(*n as usize).collect();
                    match direction {
                        TimeDir::Future => Some(TimeSpan {
                            start: spans.first().unwrap().start,
                            end: spans.last().unwrap().end,
                            grain: spans.first().unwrap().grain,
                        }),
                        TimeDir::Past => Some(TimeSpan {
                            start: spans.last().unwrap().start,
                            end: spans.first().unwrap().end,
                            grain: spans.first().unwrap().grain,
                        }),
                    }
                }))
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

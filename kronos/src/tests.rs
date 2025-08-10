mod test_grains {
    use crate::sequence::*;
    use time::macros::datetime;

    #[test]
    fn weekend() {
        let s = TimeSeq::weekends();
        let mut f = s.future(datetime!(2025-07-01 0:00));
        assert_eq!(
            f.next().unwrap(),
            TimeSpan {
                start: datetime!(2025-07-05 0:00),
                end: datetime!(2025-07-07 0:00),
                grain: Grain::Day,
            }
        );
        // Check weekend englobes date
        let mut f = s.future(datetime!(2025-07-20 0:00));
        assert_eq!(
            f.next().unwrap(),
            TimeSpan {
                start: datetime!(2025-07-19 0:00),
                end: datetime!(2025-07-21 0:00),
                grain: Grain::Day,
            }
        );
        let mut p = s.past(datetime!(2015-03-14 0:00));
        assert_eq!(
            p.next().unwrap(),
            TimeSpan {
                start: datetime!(2015-03-07 0:00),
                end: datetime!(2015-03-09 0:00),
                grain: Grain::Day
            }
        );
    }

    #[test]
    fn weeks() {
        // check first element englobes date
        let s = TimeSeq::weeks();
        let mut f = s.future(datetime!(2016-01-01 0:00));
        assert_eq!(
            f.next().unwrap(),
            TimeSpan {
                start: datetime!(2015-12-27 0:00),
                end: datetime!(2016-01-03 0:00),
                grain: Grain::Day
            }
        );
        let mut f = s.past(datetime!(2016-01-01 0:00));
        assert_eq!(
            f.next().unwrap(),
            TimeSpan {
                start: datetime!(2015-12-20 0:00),
                end: datetime!(2015-12-27 0:00),
                grain: Grain::Day
            }
        );
    }

    #[test]
    fn monthday() {
        let s = TimeSeq::monthday(31);
        let mut f = s.future(datetime!(2025-07-01 0:00));
        assert_eq!(
            f.next().unwrap(),
            TimeSpan {
                start: datetime!(2025-07-31 0:00),
                end: datetime!(2025-08-01 0:00),
                grain: Grain::Day,
            }
        );
        assert_eq!(
            f.next().unwrap(),
            TimeSpan {
                start: datetime!(2025-08-31 0:00),
                end: datetime!(2025-09-01 0:00),
                grain: Grain::Day,
            }
        );
        assert_eq!(
            f.next().unwrap(),
            TimeSpan {
                start: datetime!(2025-10-31 0:00),
                end: datetime!(2025-11-01 0:00),
                grain: Grain::Day,
            }
        );
    }

    #[test]
    fn days() {
        let s = TimeSeq::days();
        let mut f = s.future(datetime!(2015-02-27 0:00));
        assert_eq!(
            f.next().unwrap(),
            TimeSpan {
                start: datetime!(2015-02-27 0:00),
                end: datetime!(2015-02-28 0:00),
                grain: Grain::Day
            }
        );
        assert_eq!(
            f.next().unwrap(),
            TimeSpan {
                start: datetime!(2015-02-28 0:00),
                end: datetime!(2015-03-01 0:00),
                grain: Grain::Day
            }
        );
    }

    #[test]
    fn months() {
        let s = TimeSeq::months(None);
        let mut f = s.future(datetime!(2015-02-27 0:00));
        assert_eq!(
            f.next().unwrap(),
            TimeSpan {
                start: datetime!(2015-02-01 0:00),
                end: datetime!(2015-03-01 0:00),
                grain: Grain::Month
            }
        );
        assert_eq!(
            f.next().unwrap(),
            TimeSpan {
                start: datetime!(2015-03-01 0:00),
                end: datetime!(2015-04-01 0:00),
                grain: Grain::Month
            }
        );
        // Specific month May
        let s = TimeSeq::months(Some(5));
        let mut f = s.future(datetime!(2015-02-27 0:00));
        assert_eq!(
            f.next().unwrap(),
            TimeSpan {
                start: datetime!(2015-05-01 0:00),
                end: datetime!(2015-06-01 0:00),
                grain: Grain::Month
            }
        );
        assert_eq!(
            f.next().unwrap(),
            TimeSpan {
                start: datetime!(2016-05-01 0:00),
                end: datetime!(2016-06-01 0:00),
                grain: Grain::Month
            }
        );
    }

    #[test]
    fn years() {
        // backward iteration
        let s = TimeSeq::years();
        let mut p = s.past(datetime!(2015-02-27 0:00));
        assert_eq!(
            p.next().unwrap(),
            TimeSpan {
                start: datetime!(2014-01-01 0:00),
                end: datetime!(2015-01-01 0:00),
                grain: Grain::Year
            }
        );
        assert_eq!(
            p.next().unwrap(),
            TimeSpan {
                start: datetime!(2013-01-01 0:00),
                end: datetime!(2014-01-01 0:00),
                grain: Grain::Year
            }
        );
    }

    #[test]
    fn minutes() {
        let s = TimeSeq::minutes(None);
        let mut f = s.future(datetime!(2015-02-27 0:00));
        assert_eq!(
            f.next().unwrap(),
            TimeSpan {
                start: datetime!(2015-02-27 0:00:00),
                end: datetime!(2015-02-27 0:01:00),
                grain: Grain::Minute
            }
        );
        let s = TimeSeq::minutes(None);
        let mut p = s.past(datetime!(2015-02-27 23:20:25));
        assert_eq!(
            p.next().unwrap(),
            TimeSpan {
                start: datetime!(2015-02-27 23:19:00),
                end: datetime!(2015-02-27 23:20:00),
                grain: Grain::Minute
            }
        );
        // Spinning minutes carry over to days
        let s = TimeSeq::minutes(None);
        let mut p = s.past(datetime!(2015-02-27 0:00));
        assert_eq!(
            p.next().unwrap(),
            TimeSpan {
                start: datetime!(2015-02-26 23:59:00),
                end: datetime!(2015-02-27 0:00:00),
                grain: Grain::Minute
            }
        );
        // Specific minute
        let s = TimeSeq::minutes(Some(23));
        let mut p = s.past(datetime!(2015-02-27 0:00));
        assert_eq!(
            p.next().unwrap(),
            TimeSpan {
                start: datetime!(2015-02-26 23:23),
                end: datetime!(2015-02-26 23:24),
                grain: Grain::Minute
            }
        );
        assert_eq!(
            p.next().unwrap(),
            TimeSpan {
                start: datetime!(2015-02-26 22:23),
                end: datetime!(2015-02-26 22:24),
                grain: Grain::Minute
            }
        );
    }

    #[test]
    fn merge() {
        let s = TimeSeq::merge(TimeSeq::days(), 2);
        let mut f = s.future(datetime!(2015-02-28 0:00));
        assert_eq!(
            f.next().unwrap(),
            TimeSpan {
                start: datetime!(2015-02-28 0:00),
                end: datetime!(2015-03-02 0:00),
                grain: Grain::Day,
            }
        );
        assert_eq!(
            f.next().unwrap(),
            TimeSpan {
                start: datetime!(2015-03-02 0:00),
                end: datetime!(2015-03-04 0:00),
                grain: Grain::Day,
            }
        );
        let mut p = s.past(datetime!(2015-02-28 0:00));
        assert_eq!(
            p.next().unwrap(),
            TimeSpan {
                start: datetime!(2015-02-26 0:00),
                end: datetime!(2015-02-28 0:00),
                grain: Grain::Day,
            }
        );
    }
}

mod test_within {
    use crate::sequence::*;
    use time::macros::datetime;

    #[test]
    fn nthof() {
        //The 3rd weekend of june
        let s = TimeSeq::weekends()
            .within(TimeSeq::months(Some(6)), 3)
            .unwrap();
        let mut f = s.future(datetime!(2025-07-01 0:00));
        assert_eq!(
            f.next().unwrap(),
            TimeSpan {
                start: datetime!(2026-06-20 0:00),
                end: datetime!(2026-06-22 0:00),
                grain: Grain::Day,
            }
        );
        assert_eq!(
            f.next().unwrap(),
            TimeSpan {
                start: datetime!(2027-06-19 0:00),
                end: datetime!(2027-06-21 0:00),
                grain: Grain::Day,
            }
        );
        // 10th day of each month (past)
        let s = TimeSeq::days().within(TimeSeq::months(None), 10).unwrap();
        let mut p = s.past(datetime!(2015-03-11 0:00));
        assert_eq!(
            p.next().unwrap(),
            TimeSpan {
                start: datetime!(2015-03-10 0:00),
                end: datetime!(2015-03-11 0:00),
                grain: Grain::Day
            }
        );
        assert_eq!(
            p.next().unwrap(),
            TimeSpan {
                start: datetime!(2015-02-10 0:00),
                end: datetime!(2015-02-11 0:00),
                grain: Grain::Day
            }
        );
        // 3rd monday of each month
        let s = TimeSeq::weekday(1)
            .within(TimeSeq::months(None), 3)
            .unwrap();
        let mut f = s.future(datetime!(2015-03-11 0:00));
        assert_eq!(
            f.next().unwrap(),
            TimeSpan {
                start: datetime!(2015-03-16 0:00),
                end: datetime!(2015-03-17 0:00),
                grain: Grain::Day
            }
        );
        assert_eq!(
            f.next().unwrap(),
            TimeSpan {
                start: datetime!(2015-04-20 0:00),
                end: datetime!(2015-04-21 0:00),
                grain: Grain::Day
            }
        );
        // Iterate into the past
        let mut p = s.past(datetime!(2015-03-11 0:00));
        assert_eq!(
            p.next().unwrap(),
            TimeSpan {
                start: datetime!(2015-02-16 0:00),
                end: datetime!(2015-02-17 0:00),
                grain: Grain::Day
            }
        );
        assert_eq!(
            p.next().unwrap(),
            TimeSpan {
                start: datetime!(2015-01-19 0:00),
                end: datetime!(2015-01-20 0:00),
                grain: Grain::Day
            }
        );
    }

    #[test]
    fn nthof_leap() {
        // 29th day of february (leap years only)
        let s = TimeSeq::days()
            .within(TimeSeq::months(Some(2)), 29)
            .unwrap();
        let mut f = s.future(datetime!(2015-01-01 0:00));
        assert_eq!(
            f.next().unwrap(),
            TimeSpan {
                start: datetime!(2016-02-29 0:00),
                end: datetime!(2016-03-01 0:00),
                grain: Grain::Day
            }
        );
        assert_eq!(
            f.next().unwrap(),
            TimeSpan {
                start: datetime!(2020-02-29 0:00),
                end: datetime!(2020-03-01 0:00),
                grain: Grain::Day
            }
        );
    }

    #[test]
    fn nthof_nonaligned() {
        // 1st weekend of Jan
        let s = TimeSeq::weekends()
            .within(TimeSeq::months(Some(1)), 1)
            .unwrap();
        let mut f = s.future(datetime!(2016-09-04 0:00));
        assert_eq!(
            f.next().unwrap(),
            TimeSpan {
                start: datetime!(2016-12-31 0:00),
                end: datetime!(2017-01-02 0:00),
                grain: Grain::Day
            }
        );
        assert_eq!(
            f.next().unwrap(),
            TimeSpan {
                start: datetime!(2018-01-06 0:00),
                end: datetime!(2018-01-08 0:00),
                grain: Grain::Day
            }
        );
    }

    #[test]
    fn nthof_composed() {
        // 10th day of the 5th month of each year
        let s = TimeSeq::days()
            .within(TimeSeq::months(None), 10)
            .unwrap()
            .within(TimeSeq::years(), 5)
            .unwrap();
        let mut f = s.future(datetime!(2015-03-11 0:00));
        assert_eq!(
            f.next().unwrap(),
            TimeSpan {
                start: datetime!(2015-05-10 0:00),
                end: datetime!(2015-05-11 0:00),
                grain: Grain::Day
            }
        );
        assert_eq!(
            f.next().unwrap(),
            TimeSpan {
                start: datetime!(2016-05-10 0:00),
                end: datetime!(2016-05-11 0:00),
                grain: Grain::Day
            }
        );
        let mut p = s.past(datetime!(2015-03-11 0:00));
        assert_eq!(
            p.next().unwrap(),
            TimeSpan {
                start: datetime!(2014-05-10 0:00),
                end: datetime!(2014-05-11 0:00),
                grain: Grain::Day
            }
        );
        assert_eq!(
            p.next().unwrap(),
            TimeSpan {
                start: datetime!(2013-05-10 0:00),
                end: datetime!(2013-05-11 0:00),
                grain: Grain::Day
            }
        );

        // the 3rd hour of 2nd day of the month
        let s = TimeSeq::hours(None)
            .within(TimeSeq::days().within(TimeSeq::months(None), 2).unwrap(), 3)
            .unwrap();
        let mut f = s.future(datetime!(2015-03-11 0:00));
        assert_eq!(
            f.next().unwrap(),
            TimeSpan {
                start: datetime!(2015-04-02 2:00),
                end: datetime!(2015-04-02 3:00),
                grain: Grain::Hour,
            }
        );
    }

    #[test]
    fn lastof() {
        // 2nd to last day of feb
        let s = TimeSeq::days().within(TimeSeq::months(Some(2)), -2).unwrap();
        let mut f = s.future(datetime!(2025-03-11 0:00));
        assert_eq!(
            f.next().unwrap(),
            TimeSpan {
                start: datetime!(2026-02-27 0:00),
                end: datetime!(2026-02-28 0:00),
                grain: Grain::Day
            }
        );
        assert_eq!(
            f.next().unwrap(),
            TimeSpan {
                start: datetime!(2027-02-27 0:00),
                end: datetime!(2027-02-28 0:00),
                grain: Grain::Day
            }
        );
        // last day of feb
        let s = TimeSeq::days().within(TimeSeq::months(Some(2)), -1).unwrap();
        let mut f = s.future(datetime!(2025-03-11 0:00));
        assert_eq!(
            f.next().unwrap(),
            TimeSpan {
                start: datetime!(2026-02-28 0:00),
                end: datetime!(2026-03-01 0:00),
                grain: Grain::Day
            }
        );
        // last 29th day of feb
        let s = TimeSeq::days().within(TimeSeq::months(Some(2)), -29).unwrap();
        let mut f = s.future(datetime!(2025-03-11 0:00));
        assert_eq!(
            f.next().unwrap(),
            TimeSpan {
                start: datetime!(2028-02-01 0:00),
                end: datetime!(2028-02-02 0:00),
                grain: Grain::Day
            }
        );
        // Last monday of each month
        let s = TimeSeq::weekday(1).within(TimeSeq::months(None), -1).unwrap();
        let mut f = s.future(datetime!(2015-03-11 0:00));
        assert_eq!(
            f.next().unwrap(),
            TimeSpan {
                start: datetime!(2015-03-30 0:00),
                end: datetime!(2015-03-31 0:00),
                grain: Grain::Day
            }
        );
        assert_eq!(
            f.next().unwrap(),
            TimeSpan {
                start: datetime!(2015-04-27 0:00),
                end: datetime!(2015-04-28 0:00),
                grain: Grain::Day
            }
        );

        // backward: 2nd-to-last day of february (in the past)
        let s = TimeSeq::days().within(TimeSeq::months(Some(2)), -2).unwrap();
        let mut p = s.past(datetime!(2014-02-25 0:00));
        assert_eq!(
            p.next().unwrap(),
            TimeSpan {
                start: datetime!(2013-02-27 0:00),
                end: datetime!(2013-02-28 0:00),
                grain: Grain::Day
            }
        );
        assert_eq!(
            p.next().unwrap(),
            TimeSpan {
                start: datetime!(2012-02-28 0:00),
                end: datetime!(2012-02-29 0:00),
                grain: Grain::Day
            }
        );
    }

    #[test]
    fn impossible_lastof() {
        let s = TimeSeq::days().within(TimeSeq::months(None), 32).unwrap();
        assert_eq!(s.future(datetime!(2015-02-25 0:00)).next(), None);
    }
}

mod test_union {
    use crate::sequence::*;
    use time::macros::datetime;

    #[test]
    fn test_union() {
        // Mondays and Wednesdays
        let s = TimeSeq::weekday(1).union(TimeSeq::weekday(3)).unwrap();
        let mut f = s.future(datetime!(2015-02-27 0:00));
        assert_eq!(
            f.next().unwrap(),
            TimeSpan {
                start: datetime!(2015-03-02 0:00),
                end: datetime!(2015-03-03 0:00),
                grain: Grain::Day
            }
        );
        assert_eq!(
            f.next().unwrap(),
            TimeSpan {
                start: datetime!(2015-03-04 0:00),
                end: datetime!(2015-03-05 0:00),
                grain: Grain::Day
            }
        );
        assert_eq!(
            f.next().unwrap(),
            TimeSpan {
                start: datetime!(2015-03-09 0:00),
                end: datetime!(2015-03-10 0:00),
                grain: Grain::Day
            }
        );
    }

    #[test]
    fn test_union_past() {
        // Mondays and Wednesdays and Fridays (into the past)
        let s = TimeSeq::weekday(1)
            .union(TimeSeq::weekday(3))
            .unwrap()
            .union(TimeSeq::weekday(5))
            .unwrap();
        let mut p = s.past(datetime!(2015-02-27 0:00));
        assert_eq!(
            p.next().unwrap(),
            TimeSpan {
                start: datetime!(2015-02-25 0:00),
                end: datetime!(2015-02-26 0:00),
                grain: Grain::Day
            }
        );
        assert_eq!(
            p.next().unwrap(),
            TimeSpan {
                start: datetime!(2015-02-23 0:00),
                end: datetime!(2015-02-24 0:00),
                grain: Grain::Day
            }
        );
    }
}

mod test_intersect {
    use crate::sequence::*;
    use time::macros::datetime;

    #[test]
    fn intersect_basic() {
        // Mondays of June
        let s = TimeSeq::months(Some(6)).intersection(TimeSeq::weekday(1));
        let mut f = s.future(datetime!(2015-03-11 0:00));
        assert_eq!(
            f.next().unwrap(),
            TimeSpan {
                start: datetime!(2015-06-01 0:00),
                end: datetime!(2015-06-02 0:00),
                grain: Grain::Day
            }
        );
        assert_eq!(
            f.next().unwrap(),
            TimeSpan {
                start: datetime!(2015-06-08 0:00),
                end: datetime!(2015-06-09 0:00),
                grain: Grain::Day
            }
        );
        assert_eq!(
            f.next().unwrap(),
            TimeSpan {
                start: datetime!(2015-06-15 0:00),
                end: datetime!(2015-06-16 0:00),
                grain: Grain::Day
            }
        );

        let mut p = s.past(datetime!(2015-06-05 0:00));
        assert_eq!(
            p.next().unwrap(),
            TimeSpan {
                start: datetime!(2015-06-01 0:00),
                end: datetime!(2015-06-02 0:00),
                grain: Grain::Day
            }
        );
        assert_eq!(
            p.next().unwrap(),
            TimeSpan {
                start: datetime!(2014-06-30 0:00),
                end: datetime!(2014-07-01 0:00),
                grain: Grain::Day
            }
        );
    }

    #[test]
    fn intersect2() {
        // 3PM on mondays
        let s = TimeSeq::hours(Some(15)).intersection(TimeSeq::weekday(1));
        let mut f = s.future(datetime!(2015-03-11 0:00));
        assert_eq!(
            f.next().unwrap(),
            TimeSpan {
                start: datetime!(2015-03-16 15:00),
                end: datetime!(2015-03-16 16:00),
                grain: Grain::Hour
            }
        );
        assert_eq!(
            f.next().unwrap(),
            TimeSpan {
                start: datetime!(2015-03-23 15:00),
                end: datetime!(2015-03-23 16:00),
                grain: Grain::Hour
            }
        );
        let mut p = s.past(datetime!(2015-03-16 16:00));
        assert_eq!(
            p.next().unwrap(),
            TimeSpan {
                start: datetime!(2015-03-16 15:00),
                end: datetime!(2015-03-16 16:00),
                grain: Grain::Hour
            }
        );
        assert_eq!(
            p.next().unwrap(),
            TimeSpan {
                start: datetime!(2015-03-09 15:00),
                end: datetime!(2015-03-09 16:00),
                grain: Grain::Hour
            }
        );
    }

    #[test]
    fn intersect_union() {
        // 3PM on (mondays or tuesdays)
        let hour_15 = TimeSeq::hours(Some(15))
            .intersection(TimeSeq::weekday(1).union(TimeSeq::weekday(2)).unwrap());
        let mut f = hour_15.future(datetime!(2015-03-11 0:00));
        assert_eq!(
            f.next().unwrap(),
            TimeSpan {
                start: datetime!(2015-03-16 15:00),
                end: datetime!(2015-03-16 16:00),
                grain: Grain::Hour
            }
        );
        assert_eq!(
            f.next().unwrap(),
            TimeSpan {
                start: datetime!(2015-03-17 15:00),
                end: datetime!(2015-03-17 16:00),
                grain: Grain::Hour
            }
        );
        assert_eq!(
            f.next().unwrap(),
            TimeSpan {
                start: datetime!(2015-03-23 15:00),
                end: datetime!(2015-03-23 16:00),
                grain: Grain::Hour
            }
        );
        let mut f = f.skip(6);
        assert_eq!(
            f.next().unwrap(),
            TimeSpan {
                start: datetime!(2015-04-14 15:00),
                end: datetime!(2015-04-14 16:00),
                grain: Grain::Hour
            }
        );
    }
}

mod test_except {
    use crate::sequence::*;
    use time::macros::datetime;

    #[test]
    fn except_basic() {
        // days except Friday and thursdays
        let s = TimeSeq::days().except(TimeSeq::weekday(4)).except(TimeSeq::weekday(5));
        let mut f = s.future(datetime!(2018-08-22 0:00));
        assert_eq!(
            f.next().unwrap(),
            TimeSpan {
                start: datetime!(2018-08-22 0:00),
                end: datetime!(2018-08-23 0:00),
                grain: Grain::Day
            }
        );
        assert_eq!(
            f.next().unwrap(),
            TimeSpan {
                start: datetime!(2018-08-25 0:00),
                end: datetime!(2018-08-26 0:00),
                grain: Grain::Day
            }
        );
        assert_eq!(
            f.next().unwrap(),
            TimeSpan {
                start: datetime!(2018-08-26 0:00),
                end: datetime!(2018-08-27 0:00),
                grain: Grain::Day
            }
        );

        let mut p = s.past(datetime!(2018-08-19 0:00));
        assert_eq!(
            p.next().unwrap(),
            TimeSpan {
                start: datetime!(2018-08-18 0:00),
                end: datetime!(2018-08-19 0:00),
                grain: Grain::Day
            }
        );
        assert_eq!(
            p.next().unwrap(),
            TimeSpan {
                start: datetime!(2018-08-15 0:00),
                end: datetime!(2018-08-16 0:00),
                grain: Grain::Day
            }
        );

        let mut p = s.past(datetime!(2018-08-17 0:00));
        assert_eq!(
            p.next().unwrap(),
            TimeSpan {
                start: datetime!(2018-08-15 0:00),
                end: datetime!(2018-08-16 0:00),
                grain: Grain::Day
            }
        );
    }

    #[test]
    fn except_diff_grains() {
        // mondays except september
        let s = TimeSeq::weekday(1).except(TimeSeq::months(Some(9)));
        let mut f = s.future(datetime!(2018-08-22 0:00));
        assert_eq!(
            f.next().unwrap(),
            TimeSpan {
                start: datetime!(2018-08-27 0:00),
                end: datetime!(2018-08-28 0:00),
                grain: Grain::Day
            }
        );
        assert_eq!(
            f.next().unwrap(),
            TimeSpan {
                start: datetime!(2018-10-01 0:00),
                end: datetime!(2018-10-02 0:00),
                grain: Grain::Day
            }
        );

        // mondays except August - past
        let s = TimeSeq::weekday(1).except(TimeSeq::months(Some(8)));
        let mut p = s.past(datetime!(2018-08-22 0:00));
        assert_eq!(
            p.next().unwrap(),
            TimeSpan {
                start: datetime!(2018-07-30 0:00),
                end: datetime!(2018-07-31 0:00),
                grain: Grain::Day
            }
        );
    }
}

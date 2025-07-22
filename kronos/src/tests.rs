mod test_grains {
    use crate::sequence::*;
    use time::macros::datetime;

    #[test]
    fn test_weekend() {
        //The 3rd weekend of june
        let seq = TimeSeq::weekends().within(TimeSeq::months(Some(6)), 3);
        let mut sequence = seq.future(datetime!(2025-07-01 0:00));
        assert_eq!(
            sequence.next().unwrap(),
            TimeSpan {
                start: datetime!(2026-06-20 0:00),
                end: datetime!(2026-06-22 0:00),
                grain: Grain::Day,
            }
        );
    }

    #[test]
    fn test_days() {
        //The 3rd weekend of june
        let seq = TimeSeq::monthday(31);
        let mut s = seq.future(datetime!(2025-07-01 0:00));
        assert_eq!(
            s.next().unwrap(),
            TimeSpan {
                start: datetime!(2025-07-31 0:00),
                end: datetime!(2025-08-01 0:00),
                grain: Grain::Day,
            }
        );
        assert_eq!(
            s.next().unwrap(),
            TimeSpan {
                start: datetime!(2025-08-31 0:00),
                end: datetime!(2025-09-01 0:00),
                grain: Grain::Day,
            }
        );
        assert_eq!(
            s.next().unwrap(),
            TimeSpan {
                start: datetime!(2025-10-31 0:00),
                end: datetime!(2025-11-01 0:00),
                grain: Grain::Day,
            }
        );
    }
}

mod test_within {
    use crate::sequence::*;
    use time::macros::datetime;

    #[test]
    fn test_within1() {
        let d10thmo = TimeSeq::days().within(TimeSeq::months(None), 10);

        let mut past = d10thmo.past(datetime!(2015-03-11 0:00));
        assert_eq!(
            past.next().unwrap(),
            TimeSpan {
                start: datetime!(2015-03-10 0:00),
                end: datetime!(2015-03-11 0:00),
                grain: Grain::Day
            }
        );
    }

    #[test]
    fn test_within2() {
        // The 10th day of each month
        let y5th10thday = TimeSeq::days()
            .within(TimeSeq::months(None), 10)
            .within(TimeSeq::years(), 5);

        // let mut future = y5th10thday.future(&dt(2015, 3, 11));
        // assert_eq!(future.next().unwrap(),
        //     Range{start: dt(2015, 5, 10), end: dt(2015, 5, 11), grain: Grain::Day});
        // assert_eq!(future.next().unwrap(),
        //     Range{start: dt(2016, 5, 10), end: dt(2016, 5, 11), grain: Grain::Day});

        let mut past = y5th10thday.past(datetime!(2015-03-11 0:00));
        assert_eq!(
            past.next().unwrap(),
            TimeSpan {
                start: datetime!(2014-05-10 0:00),
                end: datetime!(2014-05-11 0:00),
                grain: Grain::Day
            }
        );
        // assert_eq!(
        //     past.next().unwrap(),
        //     Range {
        //         start: dt(2013, 5, 10),
        //         end: dt(2013, 5, 11),
        //         grain: Grain::Day
        //     }
        // );
    }

    #[test]
    fn test_lastn() {
        // 2nd to last day of feb
        let last2ndfeb = TimeSeq::days().within(TimeSeq::months(Some(2)), -2);
        let mut f = last2ndfeb.future(datetime!(2025-03-11 0:00));
        assert_eq!(
            f.next().unwrap(),
            TimeSpan {
                start: datetime!(2026-02-27 0:00),
                end: datetime!(2026-02-28 0:00),
                grain: Grain::Day
            }
        );
        // 3nd to last day of feb
        let last2ndfeb = TimeSeq::days().within(TimeSeq::months(Some(2)), -3);
        let mut f = last2ndfeb.future(datetime!(2025-03-11 0:00));
        assert_eq!(
            f.next().unwrap(),
            TimeSpan {
                start: datetime!(2026-02-26 0:00),
                end: datetime!(2026-02-27 0:00),
                grain: Grain::Day
            }
        );
        // last day of feb
        let last2ndfeb = TimeSeq::days().within(TimeSeq::months(Some(2)), -1);
        let mut f = last2ndfeb.future(datetime!(2025-03-11 0:00));
        assert_eq!(
            f.next().unwrap(),
            TimeSpan {
                start: datetime!(2026-02-28 0:00),
                end: datetime!(2026-03-01 0:00),
                grain: Grain::Day
            }
        );
        // last 27th day of feb
        let last2ndfeb = TimeSeq::days().within(TimeSeq::months(Some(2)), -28);
        let mut f = last2ndfeb.future(datetime!(2025-03-11 0:00));
        assert_eq!(
            f.next().unwrap(),
            TimeSpan {
                start: datetime!(2026-02-01 0:00),
                end: datetime!(2026-02-02 0:00),
                grain: Grain::Day
            }
        );
        // last 29th day of feb
        let last2ndfeb = TimeSeq::days().within(TimeSeq::months(Some(2)), -29);
        let mut f = last2ndfeb.future(datetime!(2025-03-11 0:00));
        assert_eq!(
            f.next().unwrap(),
            TimeSpan {
                start: datetime!(2028-02-01 0:00),
                end: datetime!(2028-02-02 0:00),
                grain: Grain::Day
            }
        );
    }
}

#![deny(warnings)]

use types::{DateTime, Range, Grain, TimeSequence};

// Guard against impossible sequences, eg: 32nd day of the month
const INFINITE_FUSE: usize = 1000;

#[derive(Clone)]
pub struct NthOf<Frame, Win>(usize, Win, Frame)
    where for<'a> Frame: TimeSequence<'a>,
          for<'a> Win: TimeSequence<'a> + Clone;

impl<'a, Frame, Win> TimeSequence<'a> for NthOf<Frame, Win>
    where for<'b> Frame: TimeSequence<'b>,
          for<'b> Win: TimeSequence<'b> + Clone + 'a
{
    // grain is taken from <win> which is actual instance within frame
    fn grain(&self) -> Grain { self.1.grain() }

    fn future(&self, t0: &DateTime) -> Box<Iterator<Item=Range> + 'a> {
        let win = self.1.clone();
        let nth = self.0;
        Box::new(self.2.future(t0)
            .map(move |outer| win.future(&outer.start)
                // only consider elements of <win> started within <frame>
                .take_while(|inner| inner.start < outer.end)
                .nth(nth - 1))
            .enumerate()
            .filter_map(|(i, elem)| { assert!(i <= INFINITE_FUSE); elem })
        )
    }

    fn past_inclusive(&self, t0: &DateTime) -> Box<Iterator<Item=Range> + 'a> {
        let win = self.1.clone();
        let nth = self.0;
        Box::new(self.2.past_inclusive(t0)
            .map(move |outer| win.future(&outer.start)
                // only consider elements of <win> started within <frame>
                .take_while(|inner| inner.start < outer.end)
                .nth(nth - 1))
            .enumerate()
            .filter_map(|(i, elem)| { assert!(i <= INFINITE_FUSE); elem })
        )
    }
}


#[cfg(test)]
fn dt(year: i32, month: u32, day: u32) -> DateTime {
    use types::Date;
    Date::from_ymd(year, month, day).and_hms(0, 0, 0)
}

#[cfg(test)]
fn dttm(year: i32, month: u32, day: u32, h: u32, m: u32, s: u32) -> DateTime {
    use types::Date;
    Date::from_ymd(year, month, day).and_hms(h, m, s)
}

#[test]
#[should_panic]
fn test_nthof_fuse() {
    use seq_grain::Grains;
    let thirtysecond = NthOf(32, Grains(Grain::Day), Grains(Grain::Month));
    thirtysecond.future(&dt(2016, 8, 31)).next();
}

#[test]
fn test_nthof_basic() {
    use seq_grain::Grains;
    use seq_named::{Weekday, Month};

    // 3rd day of the month
    let day3 = NthOf(3, Grains(Grain::Day), Grains(Grain::Month));
    let mut day3 = day3.future(&dt(2016, 2, 2));
    assert_eq!(day3.next().unwrap(),
        Range{start: dt(2016, 2, 3), end: dt(2016, 2, 4), grain: Grain::Day});
    assert_eq!(day3.next().unwrap(),
        Range{start: dt(2016, 3, 3), end: dt(2016, 3, 4), grain: Grain::Day});

    // 3rd tuesday of the month
    let tue3mo = NthOf(3, Weekday(2), Grains(Grain::Month));
    let mut tue3mo = tue3mo.future(&dt(2016, 2, 10));
    assert_eq!(tue3mo.next().unwrap(),
        Range{start: dt(2016, 2, 16), end: dt(2016, 2, 17), grain: Grain::Day});
    assert_eq!(tue3mo.next().unwrap(),
        Range{start: dt(2016, 3, 15), end: dt(2016, 3, 16), grain: Grain::Day});

    // 2nd monday of april
    let secmonapr = NthOf(2, Weekday(1), Month(4));
    let mut secmonapr = secmonapr.future(&dt(2016, 2, 25));
    assert_eq!(secmonapr.next().unwrap(),
        Range{start: dt(2016, 4, 11), end: dt(2016, 4, 12), grain: Grain::Day});
    assert_eq!(secmonapr.next().unwrap(),
        Range{start: dt(2017, 4, 10), end: dt(2017, 4, 11), grain: Grain::Day});

    // 3rd week of june
    let thirdwkjune = NthOf(3, Grains(Grain::Week), Month(6));
    let mut thirdwkjune = thirdwkjune.future(&dt(2016, 9, 4));
    assert_eq!(thirdwkjune.next().unwrap(),
        Range{start: dt(2017, 6, 11), end: dt(2017, 6, 18), grain: Grain::Week});
    assert_eq!(thirdwkjune.next().unwrap(),
        Range{start: dt(2018, 6, 10), end: dt(2018, 6, 17), grain: Grain::Week});
}

#[test]
fn test_nthof_past() {
    use seq_grain::Grains;
    use seq_named::{Weekday, Month};

    // backward: 3rd hour of Saturday, looking into the past
    let thirdhour = NthOf(3, Grains(Grain::Hour), Weekday(6));
    let mut thirdhour = thirdhour.past(&dttm(2016, 3, 19, 8, 0, 0));
    assert_eq!(thirdhour.next().unwrap(),
        Range{start: dttm(2016, 3, 19, 2, 0, 0),
              end: dttm(2016, 3, 19, 3, 0, 0), grain: Grain::Hour});
    assert_eq!(thirdhour.next().unwrap(),
        Range{start: dttm(2016, 3, 12, 2, 0, 0),
              end: dttm(2016, 3, 12, 3, 0, 0), grain: Grain::Hour});

    // past inclusive
    let thirdhour = NthOf(3, Grains(Grain::Hour), Weekday(6));
    let mut thirdhour = thirdhour.past_inclusive(&dttm(2016, 3, 19, 2, 25, 0));
    assert_eq!(thirdhour.next().unwrap(),
        Range{start: dttm(2016, 3, 19, 2, 0, 0),
              end: dttm(2016, 3, 19, 3, 0, 0), grain: Grain::Hour});
    // past non-inclusive
    let thirdhour = NthOf(3, Grains(Grain::Hour), Weekday(6));
    let mut thirdhour = thirdhour.past(&dttm(2016, 3, 19, 2, 25, 0));
    assert_eq!(thirdhour.next().unwrap(),
        Range{start: dttm(2016, 3, 12, 2, 0, 0),
              end: dttm(2016, 3, 12, 3, 0, 0), grain: Grain::Hour});

    // backward: 3rd week of june
    let thirdwkjune = NthOf(3, Grains(Grain::Week), Month(6));
    let mut thirdwkjune = thirdwkjune.past(&dt(2016, 9, 4));
    assert_eq!(thirdwkjune.next().unwrap(),
        Range{start: dt(2016, 6, 12), end: dt(2016, 6, 19), grain: Grain::Week});
    assert_eq!(thirdwkjune.next().unwrap(),
        Range{start: dt(2015, 6, 14), end: dt(2015, 6, 21), grain: Grain::Week});

    // backward: feb 28th
    let t0_28th = dttm(2022, 2, 28, 1, 0, 0);
    let twenty8th = NthOf(28, Grains(Grain::Day), Grains(Grain::Month));
    let mut atwenty8th = twenty8th.past(&t0_28th);
    assert_eq!(atwenty8th.next().unwrap(),
        Range{start: dt(2022, 1, 28), end: dt(2022, 1, 29), grain: Grain::Day});
    // past-inclusive
    let mut atwenty8th = twenty8th.past_inclusive(&t0_28th);
    assert_eq!(atwenty8th.next().unwrap(),
        Range{start: dt(2022, 2, 28), end: dt(2022, 3, 1), grain: Grain::Day});
}

#[test]
fn test_nth_discontinuous() {
    use seq_grain::Grains;
    use seq_named::Month;

    let feb29th = NthOf(29, Grains(Grain::Day), Month(2));
    let mut feb29th = feb29th.future(&dt(2015, 2, 25));
    assert_eq!(feb29th.next().unwrap(),
        Range{start: dt(2016, 2, 29), end: dt(2016, 3, 1), grain: Grain::Day});
    assert_eq!(feb29th.next().unwrap(),
        Range{start: dt(2020, 2, 29), end: dt(2020, 3, 1), grain: Grain::Day});

    let thirtyfirst = NthOf(31, Grains(Grain::Day), Grains(Grain::Month));
    let mut thirtyfirst = thirtyfirst.future(&dt(2016, 8, 31));
    assert_eq!(thirtyfirst.next().unwrap(),
        Range{start: dt(2016, 8, 31), end: dt(2016, 9, 1), grain: Grain::Day});
    assert_eq!(thirtyfirst.next().unwrap(),
        Range{start: dt(2016, 10, 31), end: dt(2016, 11, 1), grain: Grain::Day});

    // backward: 29th of february
    let feb29th = NthOf(29, Grains(Grain::Day), Month(2));
    let mut feb29th = feb29th.past(&dt(2015, 2, 25));
    assert_eq!(feb29th.next().unwrap(),
        Range{start: dt(2012, 2, 29), end: dt(2012, 3, 1), grain: Grain::Day});
    assert_eq!(feb29th.next().unwrap(),
        Range{start: dt(2008, 2, 29), end: dt(2008, 3, 1), grain: Grain::Day});

    // backward: 29th of february past-inclusive
    let feb29th = NthOf(29, Grains(Grain::Day), Month(2));
    let mut feb29th = feb29th.past_inclusive(&dt(2016, 2, 25));
    assert_eq!(feb29th.next().unwrap(),
        Range{start: dt(2016, 2, 29), end: dt(2016, 3, 1), grain: Grain::Day});
}


//#[test]
//fn test_nth_non_aligned() {
    //let firstwkendjan = Seq::nthof(1, Seq::weekend(), Seq::month(1));
    //let mut firstwkendjan = firstwkendjan(dt(2016, 9, 4), TimeDir::Future);
    //assert_eq!(firstwkendjan.next().unwrap(),
               //Range{start: dt(2016, 12, 31), end: dt(2017, 1, 2),
                     //grain: Grain::Day});
    //assert_eq!(firstwkendjan.next().unwrap(),
               //Range{start: dt(2018, 1, 6), end: dt(2018, 1, 8),
                     //grain: Grain::Day});
//}

#[test]
fn test_nth_composed() {
    use seq_grain::Grains;

    // the 5th instance of 10th-day-of-the-month (each year) aka May 10th
    let mo10th = NthOf(10, Grains(Grain::Day), Grains(Grain::Month));
    let y5th10thday = NthOf(5, mo10th, Grains(Grain::Year));
    let mut future = y5th10thday.future(&dt(2015, 3, 11));
    assert_eq!(future.next().unwrap(),
        Range{start: dt(2015, 5, 10), end: dt(2015, 5, 11), grain: Grain::Day});
    assert_eq!(future.next().unwrap(),
        Range{start: dt(2016, 5, 10), end: dt(2016, 5, 11), grain: Grain::Day});

    let mut past = y5th10thday.past(&dt(2015, 3, 11));
    assert_eq!(past.next().unwrap(),
        Range{start: dt(2014, 5, 10), end: dt(2014, 5, 11), grain: Grain::Day});
    assert_eq!(past.next().unwrap(),
        Range{start: dt(2013, 5, 10), end: dt(2013, 5, 11), grain: Grain::Day});

    let mut past_inclusive = y5th10thday.past_inclusive(&dt(2015, 3, 11));
    assert_eq!(past_inclusive.next().unwrap(),
        Range{start: dt(2015, 5, 10), end: dt(2015, 5, 11), grain: Grain::Day});

    // the 3rd hour of 2nd day of the month
    let day2 = NthOf(2, Grains(Grain::Day), Grains(Grain::Month));
    let hour3day2 = NthOf(3, Grains(Grain::Hour), day2);
    let mut future = hour3day2.future2(&dt(2015, 3, 11));
    assert_eq!(future.next().unwrap(),
        Range{start: dttm(2015, 4, 2, 2, 0, 0),
              end: dttm(2015, 4, 2, 3, 0, 0), grain: Grain::Hour});
}

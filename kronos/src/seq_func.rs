#![deny(warnings)]

use crate::types::{DateTime, Range, Grain, TimeSequence};

#[derive(Clone)]
pub struct Map<Seq, RangeMapper>(pub Seq, pub RangeMapper)
    where Seq: TimeSequence,
          RangeMapper: FnMut(Range)->Option<Range> + Clone;

impl<Seq, RangeMapper> TimeSequence for Map<Seq, RangeMapper>
    where Seq: TimeSequence,
  RangeMapper: FnMut(Range)->Option<Range> + Clone,

{
    fn _future_raw(&self, t0: &DateTime) -> Box<dyn Iterator<Item=Range> + '_> {
        let mut f = self.1.clone();
        Box::new(self.0._future_raw(t0).filter_map(move |x| f(x)))
    }

    fn _past_raw(&self, t0: &DateTime) -> Box<dyn Iterator<Item=Range> + '_> {
        let mut f = self.1.clone();
        Box::new(self.0._past_raw(t0).filter_map(move |x| f(x)))
    }
}


pub fn shift<Seq>(seq: Seq, grain: Grain, n: i32) -> impl TimeSequence
    where Seq: TimeSequence
{
    use crate::utils;
    Map(seq, move |x| Some(Range{
        start: utils::shift_datetime(x.start, grain, n),
        end: utils::shift_datetime(x.end, grain, n),
        grain: x.grain}))
}


pub fn step_by<Seq>(seq: Seq, n: usize) -> impl TimeSequence
    where Seq: TimeSequence
{
    let mut counter = 0;
    Map(seq, move |x| {
        counter += 1;
        if (counter - 1) % n == 0 {
            Some(x)
        } else {
            None
        }
    })
}

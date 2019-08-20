#![deny(warnings)]

use std::rc::Rc;
use crate::types::{DateTime, Range, Grain, TimeSequence};

#[derive(Clone)]
pub struct Map<Seq>(pub Seq, pub Rc<dyn Fn(Range)->Option<Range>>)
    where Seq: TimeSequence;

impl<Seq> TimeSequence for Map<Seq>
    where Seq: TimeSequence + Clone
{
    fn _future_raw(&self, t0: &DateTime) -> Box<dyn Iterator<Item=Range> + '_> {
        let f = self.1.clone();
        Box::new(self.0._future_raw(t0).filter_map(move |x| f(x)))
    }

    fn _past_raw(&self, t0: &DateTime) -> Box<dyn Iterator<Item=Range> + '_> {
        let f = self.1.clone();
        Box::new(self.0._past_raw(t0).filter_map(move |x| f(x)))
    }
}


pub fn shift<Seq>(seq: Seq, grain: Grain, n: i32) -> impl TimeSequence
    where Seq: TimeSequence + Clone
{
    use crate::utils;
    Map(seq, Rc::new(move |x| Some(Range{
        start: utils::shift_datetime(x.start, grain, n),
        end: utils::shift_datetime(x.end, grain, n),
        grain: x.grain})))
}

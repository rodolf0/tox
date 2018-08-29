#![deny(warnings)]

use std::rc::Rc;
use types::{DateTime, Range, Grain, TimeSequence};

#[derive(Clone)]
pub struct Map<Seq>(pub Seq, pub Rc<Fn(Range)->Option<Range>>)
    where for<'a> Seq: TimeSequence<'a>;

impl<'a, Seq> TimeSequence<'a> for Map<Seq>
    where for<'b> Seq: TimeSequence<'b> + Clone + 'a
{
    fn _future_raw(&self, t0: &DateTime) -> Box<Iterator<Item=Range> + 'a> {
        let f = self.1.clone();
        Box::new(self.0._future_raw(t0).filter_map(move |x| f(x)))
    }

    fn _past_raw(&self, t0: &DateTime) -> Box<Iterator<Item=Range> + 'a> {
        let f = self.1.clone();
        Box::new(self.0._past_raw(t0).filter_map(move |x| f(x)))
    }
}


pub fn shift<'a, Seq>(seq: Seq, grain: Grain, n: i32) -> impl TimeSequence<'a>
    where for<'b> Seq: TimeSequence<'b> + Clone + 'a
{
    use utils;
    Map(seq, Rc::new(move |x| Some(Range{
        start: utils::shift_datetime(x.start, grain, n),
        end: utils::shift_datetime(x.end, grain, n),
        grain: x.grain})))
}

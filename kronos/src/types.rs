//#![deny(warnings)]



// duckling links
// - https://github.com/wit-ai/duckling_old/blob/6b7e2e1bdbd50299cee4075ff48d7323c05758bc/src/duckling/time/pred.clj#L57-L72
// - https://duckling.wit.ai/#limitations
// - https://github.com/wit-ai/duckling_old/blob/6b7e2e1bdbd50299cee4075ff48d7323c05758bc/src/duckling/time/pred.clj#L333

// base:
// - named bases (tuesday)
// - base by granularity (month)
// - range base (weekend, mon 2.30pm to tues 3am)
//   - monday to friday
//   - afternoon (13hs - 19hs)
//
// - disjoint (mon, wed and fri)
//   - mon 2.30pm to tue 1am and fri 4 to 5pm
//   - each iteration picks one of the options
// - multiple-base eg: 2 days -> mon+tue, wed+thu, fri+sat ...
//
// filters:
// - ever other month
// - of june (is this a filter or a base?)
// - shift-by-2 (eg 2 days after monday)
//
//
// * composite durations: 3hs and 20 minutes
//
// * SET operations - union / intersect / etc
//
// granularity: inferred from base
//
// Bases implement a trait that anchors them (eg: Monday how to turn into datetime?)
//
// base.eval_at(instant, future) -> iterator<Range>



// * is moving to the past different in all types?
//
// * should future/past first element contain reftime?
//   - maybe should assume past contains it too?
//   - provide Range fn contains(t0)
//   - if t0 is standing in first range going past / future, should it return it?
//     - eg: Monday if t0 is Monday
//
// * in which case do we not-want this?
//   - seems past should also contain t0 if part of it is to be accounted yet
//   - rephrase trait past method:
//     start-time of ranges must be less-or-eq than reference
//
// * call past(strict=true) instead of latent? ... ie: range ends before-eq t0
//   - strict means end-time must be less-or-eq than t0
//
//   - strict for future means that start-time must be greater-or-eq than t0
//
// * strict needed in:
//   - 3rd hour of the weekend when t0 is within weekend and going to the past
//     if you're past 3rd hour ... you want this weekend too
//
//   - seems 'strict' can be defined directly by lastof / nthof
//   - don't expose 'strict' in interface?
//
// * strict shouldn't be part of the TimeSequence interface, it should be
//   an adaptor
//
// * eval method (future/past) exposed to user shouldn't have 'strict' option
//  - there should be an internal version that does have it for composition
//




// TimeSequence is a floating description of a set of time Ranges.
// They can be evaluated in the context of an instant to produce time Ranges.

extern crate chrono;

pub type DateTime = chrono::NaiveDateTime;
pub type Date = chrono::NaiveDate;
pub type Duration = chrono::Duration;

#[derive(Debug,PartialEq,Eq,PartialOrd,Ord,Clone,Copy)]
pub enum Grain {
    Second,
    Minute,
    Hour,
    Day,
    Week,
    Month,
    Quarter,
    Half,
    Year,
    Lustrum,
    Decade,
    Century,
    Millenium,
}

// TODO: Fortnight is not aligned to any known frame its just 14 nights



// Ranges are right-open intervals of time, ie: [start, end)
#[derive(Clone,Debug,PartialEq)]
pub struct Range {
    pub start: DateTime, // included
    pub end: DateTime,   // excluded
    pub grain: Grain,
}

pub trait TimeSequence<'a> {
    // Resolution of Ranges produced by this sequence
    fn grain(&self) -> Grain;

    // Yield instances of this sequence into the future.
    // End-time of Ranges must be greater than reference t0 DateTime.
    // NOTE: First Range may start after t0 if for example discontinuous.
    fn _future_raw(&self, t0: &DateTime) -> Box<dyn Iterator<Item=Range> + 'a>;

    // Yield instances of this sequence into the past
    // Start-time of emited Ranges must be less-or-equal than reference t0.
    fn _past_raw(&self, t0: &DateTime) -> Box<Iterator<Item=Range> + 'a>;

    // NOTE: past_raw and future_raw are mainly used internaly.
    // Their first elements may overlap and are needed for composing NthOf.
    // End-user wants future + past which have no overlap in emitted Ranges

    fn future(&self, t0: &DateTime) -> Box<dyn Iterator<Item=Range> + 'a> {
        let t0 = t0.clone();
        Box::new(self._future_raw(&t0)
            .skip_while(move |range| range.end <= t0))
    }

    // End-time of emited Ranges must be less-or-equal than reference DateTime.
    // Complement of "future" where end-time must be greater than t0.
    fn past(&self, t0: &DateTime) -> Box<Iterator<Item=Range> + 'a> {
        let t0 = t0.clone();
        Box::new(self._past_raw(&t0)
            .skip_while(move |range| range.end > t0))
    }
}

#![deny(warnings)]

pub type DateTime = chrono::NaiveDateTime;
pub type Date = chrono::NaiveDate;
pub type Duration = chrono::Duration;

use std::str::FromStr;


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

impl FromStr for Grain {
    type Err = String;
    fn from_str(s: &str) -> Result<Grain, String> {
        match s.to_lowercase().as_ref() {
            "second" | "seconds" => Ok(Grain::Second),
            "minute" | "minutes" => Ok(Grain::Minute),
            "hour" | "hours" => Ok(Grain::Hour),
            "day" | "days" => Ok(Grain::Day),
            "week" | "weeks" => Ok(Grain::Week),
            "month" | "months" => Ok(Grain::Month),
            "quarter" | "quarters" => Ok(Grain::Quarter),
            "half" | "halfs" => Ok(Grain::Half),
            "year" | "years" => Ok(Grain::Year),
            "lustrum" | "lustrums" => Ok(Grain::Lustrum),
            "decade" | "decades" => Ok(Grain::Decade),
            "century" | "centuries" => Ok(Grain::Century),
            "millenium" | "millenia" | "milleniums" => Ok(Grain::Millenium),
            _ => Err(format!("Can't build Grain from {}", s))
        }
    }
}

#[derive(Debug,PartialEq,Eq,Clone,Copy)]
pub enum Season {
    Spring,
    Summer,
    Autumn,
    Winter,
}

impl FromStr for Season {
    type Err = String;
    fn from_str(s: &str) -> Result<Season, String> {
        match s.to_lowercase().as_ref() {
            "spring" | "springs" => Ok(Season::Spring),
            "summer" | "summers" => Ok(Season::Summer),
            "autumn" | "autumns" => Ok(Season::Autumn),
            "winter" | "winters" => Ok(Season::Winter),
            _ => Err(format!("Can't build Season from {}", s))
        }
    }
}

// Ranges are right-open intervals of time, ie: [start, end)
#[derive(Clone,Debug,PartialEq)]
pub struct Range {
    pub start: DateTime, // included
    pub end: DateTime,   // excluded
    pub grain: Grain,    // resolution of start/end
}

impl Range {
    pub fn intersect(&self, other: &Range) -> Option<Range> {
        use std::cmp;
        if self.start < other.end && self.end > other.start {
            return Some(Range{
                start: cmp::max(self.start, other.start),
                end: cmp::min(self.end, other.end),
                grain: cmp::min(self.grain, other.grain)
            });
        }
        None
    }

    pub fn len(&self) -> Duration {
        self.end.signed_duration_since(self.start)
    }
}


// TimeSequence is a floating description of a set of time Ranges.
// They can be evaluated in the context of an instant to produce time Ranges.

pub trait TimeSequence {
    // Yield instances of this sequence into the future.
    // End-time of Ranges must be greater than reference t0 DateTime.
    // NOTE: First Range may start after t0 if for example discontinuous.
    fn _future_raw(&self, t0: &DateTime) -> Box<dyn Iterator<Item=Range> + '_>;

    // Yield instances of this sequence into the past
    // Start-time of emited Ranges must be less-or-equal than reference t0.
    fn _past_raw(&self, t0: &DateTime) -> Box<dyn Iterator<Item=Range> + '_>;

    // NOTE: past_raw and future_raw are mainly used internaly.
    // Their first elements may overlap and are needed for composing NthOf.
    // End-user wants future + past which have no overlap in emitted Ranges

    fn future(&self, t0: &DateTime) -> Box<dyn Iterator<Item=Range> + '_> {
        let t0 = *t0;
        Box::new(self._future_raw(&t0)
            .skip_while(move |range| range.end <= t0))
    }

    // End-time of emited Ranges must be less-or-equal than reference DateTime.
    // Complement of "future" where end-time must be greater than t0.
    fn past(&self, t0: &DateTime) -> Box<dyn Iterator<Item=Range> + '_> {
        let t0 = *t0;
        Box::new(self._past_raw(&t0)
            .skip_while(move |range| range.end > t0))
    }
}

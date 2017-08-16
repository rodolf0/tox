#![deny(warnings)]

extern crate chrono;
type DateTime = chrono::NaiveDateTime;
type Date = chrono::NaiveDate;

use kronos;
use std::collections::HashMap;
use time_machine::TimeMachine;


pub struct TrainingSet<'a> {
    reftime: DateTime,
    examples: HashMap<&'a str, kronos::Range>,
    tm: TimeMachine<'a>,
}

fn r(t0: &str, t1: &str, g: kronos::Grain) -> kronos::Range {
    let fmt = "%Y-%m-%d %H:%M:%S";
    let t0 = DateTime::parse_from_str(t0, fmt).or(
                DateTime::parse_from_str(&format!("{} 00:00:00", t0), fmt))
                .expect("Bad t0 parse");
    let t1 = DateTime::parse_from_str(t1, fmt).or(
                DateTime::parse_from_str(&format!("{} 00:00:00", t1), fmt))
                .expect("Bad t0 parse");
    kronos::Range{start: t0, end: t1, grain: g}
}

pub fn load_trainingset<'a>() -> TrainingSet<'a> {
    use kronos::Grain::*;
    let reftime = Date::from_ymd(2017, 08, 12).and_hms(0, 0, 0);
    let examples: HashMap<&str, kronos::Range> = [

        ("mon", r("2017-08-14", "2017-08-15", Day)),
        ("monday", r("2017-08-14", "2017-08-15", Day)),
        ("next monday", r("2017-08-14", "2017-08-15", Day)),
        ("this monday", r("2017-08-14", "2017-08-15", Day)),
        ("next march", r("2018-03-01", "2018-04-01", Month)),

    ].iter().cloned().collect();

    TrainingSet{reftime: reftime, examples: examples, tm: TimeMachine::new()}
}

use std::collections::HashMap;

use std::iter::FromIterator;

fn weekday(d: &str) -> Option<usize> {
    match d {
        "sunday"    |  "sun" => Some(0),
        "monday"    |  "mon" => Some(1),
        "tuesday"   |  "tue" => Some(2),
        "wednesday" |  "wed" => Some(3),
        "thursday"  |  "thu" => Some(4),
        "friday"    |  "fri" => Some(5),
        "saturday"  |  "sat" => Some(6),
        _           => None
    }
}

fn month(m: &str) -> Option<usize> {
    match m {
        "january"   |  "jan" => Some(1),
        "february"  |  "feb" => Some(2),
        "march"     |  "mar" => Some(3),
        "april"     |  "apr" => Some(4),
        "may"       |  "may" => Some(5),
        "june"      |  "jun" => Some(6),
        "july"      |  "jul" => Some(7),
        "august"    |  "aug" => Some(8),
        "september" |  "sep" => Some(9),
        "october"   |  "oct" => Some(10),
        "november"  |  "nov" => Some(11),
        "december"  |  "dec" => Some(12),
        _           => None
    }
}

fn ordinal(n: &str) -> Option<usize> {
    static ORD: [&'static str;31] = [
        "first", "second", "third", "fourth", "fifth", "sixth", "seventh",
        "eigth", "ninth", "thenth", "eleventh", "twelveth", "thirteenth",
        "fourteenth", "fifteenth", "sixteenth", "seventeenth", "eighteenth",
        "nineteenth", "twentieth", "twenty-first", "twenty-second",
        "twenty-third", "twenty-fourth", "twenty-fith", "twenty-sixth",
        "twenty-seventh", "twenty-eigth", "twenty-ninth", "thirtieth",
        "thirty-first",
    ];
    let ord = ORD.iter()
       .enumerate()
       .filter_map(|(i, name)| match *name == n { true => Some(i), _=> None })
       .next();
}

fn short_ordinal(n: &str) -> Option<usize> {
    use std::str::FromStr;
    let num = n.chars().take_while(|d| d.is_numeric()).collect::<String>();
    match &n[d.len()..] {
        "st"|"nd"|"rd"|"th" => usize::from_str(num),
        _ => None
    }
}

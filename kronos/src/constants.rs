#![deny(warnings)]

pub fn weekday(d: &str) -> Option<u32> {
    match d {
        "sunday"    | "sundays"    | "sun" => Some(0),
        "monday"    | "mondays"    | "mon" => Some(1),
        "tuesday"   | "tuesdays"   | "tue" => Some(2),
        "wednesday" | "wednesdays" | "wed" => Some(3),
        "thursday"  | "thursdays"  | "thu" => Some(4),
        "friday"    | "fridays"    | "fri" => Some(5),
        "saturday"  | "saturdays"  | "sat" => Some(6),
        _           => None
    }
}

pub fn month(m: &str) -> Option<u32> {
    match m {
        "january"   |  "jan"    => Some(1),
        "february"  |  "feb"    => Some(2),
        "march"     |  "mar"    => Some(3),
        "april"     |  "apr"    => Some(4),
        "may"       => Some(5),
        "june"      |  "jun"    => Some(6),
        "july"      |  "jul"    => Some(7),
        "august"    |  "aug"    => Some(8),
        "september" |  "sep"    => Some(9),
        "october"   |  "oct"    => Some(10),
        "november"  |  "nov"    => Some(11),
        "december"  |  "dec"    => Some(12),
        _           => None
    }
}

pub fn ordinal(n: &str) -> Option<u32> {
    static ORD: [&str;31] = [
        "first", "second", "third", "fourth", "fifth", "sixth", "seventh",
        "eigth", "ninth", "thenth", "eleventh", "twelveth", "thirteenth",
        "fourteenth", "fifteenth", "sixteenth", "seventeenth", "eighteenth",
        "nineteenth", "twentieth", "twenty-first", "twenty-second",
        "twenty-third", "twenty-fourth", "twenty-fith", "twenty-sixth",
        "twenty-seventh", "twenty-eigth", "twenty-ninth", "thirtieth",
        "thirty-first",
    ];
    ORD.iter()
       .enumerate()
       .filter_map(|(i, name)| match *name == n {
           true => Some((i+1) as u32), _=> None })
       .next()
}

pub fn short_ordinal(n: &str) -> Option<u32> {
    use std::str::FromStr;
    let num = n.chars().take_while(|d| d.is_numeric()).collect::<String>();
    match &n[num.len()..] {
        "st"|"nd"|"rd"|"th" => u32::from_str(&num).ok(),
        _ => None
    }
}

#[cfg(test)]
mod tests {
    use super::{ordinal, short_ordinal};
    #[test]
    fn test_short_ordinal() {
        assert_eq!(short_ordinal("22nd"), Some(22));
        assert_eq!(short_ordinal("43rd"), Some(43));
        assert_eq!(short_ordinal("5ht"), None);
    }
    #[test]
    fn test_ordinal() {
        assert_eq!(ordinal("twenty-fourth"), Some(24));
        assert_eq!(ordinal("twelveth"), Some(12));
    }
}

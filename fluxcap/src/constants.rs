#![deny(warnings)]

pub fn weekday(d: &str) -> Option<u8> {
    match d {
        "sunday" | "sundays" | "sun" => Some(0),
        "monday" | "mondays" | "mon" => Some(1),
        "tuesday" | "tuesdays" | "tue" => Some(2),
        "wednesday" | "wednesdays" | "wed" => Some(3),
        "thursday" | "thursdays" | "thu" => Some(4),
        "friday" | "fridays" | "fri" => Some(5),
        "saturday" | "saturdays" | "sat" => Some(6),
        _ => None,
    }
}

pub fn month(m: &str) -> Option<u8> {
    match m {
        "january" | "januaries" | "jan" => Some(1),
        "february" | "februaries" | "feb" => Some(2),
        "march" | "marches" | "mar" => Some(3),
        "april" | "aprils" | "apr" => Some(4),
        "may" | "mays" => Some(5),
        "june" | "junes" | "jun" => Some(6),
        "july" | "julys" | "jul" => Some(7),
        "august" | "augusts" | "aug" => Some(8),
        "september" | "septembers" | "sep" => Some(9),
        "october" | "octobers" | "oct" => Some(10),
        "november" | "novembers" | "nov" => Some(11),
        "december" | "decembers" | "dec" => Some(12),
        _ => None,
    }
}

pub fn ordinal(n: &str) -> Option<u8> {
    static ORD: [&str; 31] = [
        "first",
        "second",
        "third",
        "fourth",
        "fifth",
        "sixth",
        "seventh",
        "eigth",
        "ninth",
        "thenth",
        "eleventh",
        "twelveth",
        "thirteenth",
        "fourteenth",
        "fifteenth",
        "sixteenth",
        "seventeenth",
        "eighteenth",
        "nineteenth",
        "twentieth",
        "twenty-first",
        "twenty-second",
        "twenty-third",
        "twenty-fourth",
        "twenty-fith",
        "twenty-sixth",
        "twenty-seventh",
        "twenty-eigth",
        "twenty-ninth",
        "thirtieth",
        "thirty-first",
    ];
    ORD.iter()
        .enumerate()
        .filter_map(|(i, txt)| if *txt == n { Some((i + 1) as u8) } else { None })
        .next()
}

pub fn short_ordinal(n: &str) -> Option<u8> {
    use std::str::FromStr;
    let num = n.chars().take_while(|d| d.is_numeric()).collect::<String>();
    match &n[num.len()..] {
        "st" | "nd" | "rd" | "th" | "sts" | "nds" | "rds" | "ths" => u8::from_str(&num).ok(),
        _ => None,
    }
}

pub fn kronos_grain(q: &str) -> Option<kronos::Grain> {
    match q {
        "second" | "seconds" => Some(kronos::Grain::Second),
        "minute" | "minutes" => Some(kronos::Grain::Minute),
        "hour" | "hours" => Some(kronos::Grain::Hour),
        "day" | "days" => Some(kronos::Grain::Day),
        "month" | "months" => Some(kronos::Grain::Month),
        "year" | "years" => Some(kronos::Grain::Year),
        _ => None,
    }
}

pub fn hour_spec(h: &str) -> Option<u8> {
    use std::str::FromStr;
    if h.len() < 3 {
        return None;
    }
    let Some((hour, ampm)) = h.split_at_checked(h.len() - 2) else {
        return None;
    };
    let hour = u8::from_str(hour).ok()?;
    match ampm {
        "am" => Some(if hour == 12 { 0 } else { hour }),
        "pm" => Some(if hour == 12 { 12 } else { hour + 12 }),
        _ => None,
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

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
        "eighth",
        "ninth",
        "tenth",
        "eleventh",
        "twelfth",
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
        "twenty-fifth",
        "twenty-sixth",
        "twenty-seventh",
        "twenty-eighth",
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

pub fn parse_clock_time(s: &str) -> Option<(u8, u8, u8, kronos::Grain)> {
    use std::str::FromStr;
    let s = s.trim();
    let mut ampm = None;
    let mut time_part = s;
    // Parse am/pm suffix if present
    if let Some(stripped) = s.strip_suffix("am") {
        ampm = Some("am");
        time_part = stripped;
    } else if let Some(stripped) = s.strip_suffix("pm") {
        ampm = Some("pm");
        time_part = stripped;
    }
    // Split time part into components
    let parts: Vec<&str> = time_part.split(':').collect();
    if parts.is_empty() || parts.len() > 3 {
        return None;
    }
    // Parse hour, minute, second
    let mut hour = u8::from_str(parts[0]).ok()?;
    let minute = if parts.len() >= 2 {
        u8::from_str(parts[1]).ok()?
    } else {
        0
    };
    let second = if parts.len() == 3 {
        u8::from_str(parts[2]).ok()?
    } else {
        0
    };
    let grain = match parts.len() {
        3 => kronos::Grain::Second,
        2 => kronos::Grain::Minute,
        _ => kronos::Grain::Hour,
    };
    // Adjust hour based on am/pm
    if let Some(ap) = ampm {
        if hour > 12 {
            return None;
        }
        if ap == "am" && hour == 12 {
            hour = 0;
        } else if ap == "pm" && hour < 12 {
            hour += 12;
        }
    }
    if hour > 23 || minute > 59 || second > 59 {
        return None;
    }
    Some((hour, minute, second, grain))
}

pub fn parse_date(s: &str) -> Option<(i32, u8, u8)> {
    use std::str::FromStr;
    if s.contains('/') {
        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() < 2 || parts.len() > 3 {
            return None;
        }
        let month = u8::from_str(parts[0]).ok()?;
        let day = u8::from_str(parts[1]).ok()?;
        let year = if parts.len() == 3 {
            let mut y = i32::from_str(parts[2]).ok()?;
            if y < 100 {
                y += 2000;
            }
            y
        } else {
            time::OffsetDateTime::now_utc().year()
        };
        let month_enum = month.try_into().ok()?;
        if time::Date::from_calendar_date(year, month_enum, day).is_err() {
            return None;
        }
        Some((year, month, day))
    } else if s.contains('-') {
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() != 3 {
            return None;
        }
        let year = i32::from_str(parts[0]).ok()?;
        let month = u8::from_str(parts[1]).ok()?;
        let day = u8::from_str(parts[2]).ok()?;
        let month_enum = month.try_into().ok()?;
        if time::Date::from_calendar_date(year, month_enum, day).is_err() {
            return None;
        }
        Some((year, month, day))
    } else {
        None
    }
}

pub fn tokenize(time: &str) -> impl Iterator<Item = String> {
    let words: Vec<&str> = time
        .split(&[' ', ','][..])
        .filter(|w| !w.is_empty())
        .collect();
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < words.len() {
        let w1 = words[i].to_lowercase();
        // Multi-word phrase merging
        if i + 2 < words.len() {
            let w2 = words[i + 1].to_lowercase();
            let w3 = words[i + 2].to_lowercase();
            if w1 == "new" && (w2 == "year's" || w2 == "years") && w3 == "eve" {
                tokens.push("new_years_eve".to_string());
                i += 3;
                continue;
            }
            if w1 == "new" && (w2 == "year's" || w2 == "years") && w3 == "day" {
                tokens.push("new_years_day".to_string());
                i += 3;
                continue;
            }
            if w1 == "end" && w2 == "of" {
                if w3 == "day" {
                    tokens.push("eod".to_string());
                    i += 3;
                    continue;
                }
                if w3 == "month" {
                    tokens.push("eom".to_string());
                    i += 3;
                    continue;
                }
                if w3 == "year" {
                    tokens.push("eoy".to_string());
                    i += 3;
                    continue;
                }
            }
        }
        if i + 1 < words.len() {
            let w2 = words[i + 1].to_lowercase();
            if w1 == "new" && w2 == "year" {
                tokens.push("new_years_day".to_string());
                i += 2;
                continue;
            }
            if (w1 == "valentine's" || w1 == "valentines") && w2 == "day" {
                tokens.push("valentines_day".to_string());
                i += 2;
                continue;
            }
            if (w1 == "memorial"
                || w1 == "labor"
                || w1 == "father's"
                || w1 == "fathers"
                || w1 == "mother's"
                || w1 == "mothers")
                && w2 == "day"
            {
                tokens.push(format!("{}_{}", w1.replace("'", ""), w2));
                i += 2;
                continue;
            }
            if w1 == "christmas" && w2 == "eve" {
                tokens.push("christmas_eve".to_string());
                i += 2;
                continue;
            }
            if w1 == "christmas" && w2 == "day" {
                tokens.push("christmas".to_string());
                i += 2;
                continue;
            }
            if w1 == "week" && w2.chars().all(|c| c.is_numeric()) {
                tokens.push(format!("week_{}", w2));
                i += 2;
                continue;
            }
        }
        tokens.push(w1);
        i += 1;
    }
    tokens.into_iter()
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
        assert_eq!(ordinal("twelfth"), Some(12));
    }
}

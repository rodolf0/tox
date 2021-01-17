use crate::scanner::Scanner;

#[test]
fn scan_number() {
    let tests = vec![
        "987",
        "-543",
        "435i",
        "41.98",
        "-83.5",
        "-54.3i",
        "28e3",
        "54E+2",
        "54e-33",
        "43e0i",
        "3E8i",
        "-38e3",
        "-53e+5",
        "-65E-4",
        "-32E-4i",
        "-33e+2i",
        "85.365e3",
        "54.234E+2",
        "54.849e-33",
        "1.4e+2i",
        "3.14e-5i",
        "-38.657e3",
        "53.845e+5",
        "65.987E-4",
        "-4.4e+2i",
        "-6.14e-5i",
    ];
    for t in tests.iter() {
        let result = Scanner::new(t.chars()).scan_number();
        assert_eq!(Some(t.to_string()), result);
    }
}

#[test]
fn scan_math_ops() {
    let tests = vec![
        "<", "<=", "==", ">=", ">", "(", ")", ",", "*", "**", "^", "!", "+", "-", "/", "%",
    ];
    for t in tests.iter() {
        let result = Scanner::new(t.chars()).scan_math_op();
        assert_eq!(Some(t.to_string()), result);
    }
}

#[test]
fn scan_identifiers() {
    let tests = vec!["id1", "func", "anyword", "_00", "bla23"];
    for t in tests.iter() {
        let result = Scanner::new(t.chars()).scan_identifier();
        assert_eq!(Some(t.to_string()), result);
    }
}

#[test]
fn scan_string() {
    let tests = vec![
        r"'this is a test'",
        r"'another test \' with an escaped quote'",
    ];
    for t in tests.iter() {
        let result = Scanner::new(t.chars()).scan_quoted_string('\'');
        assert_eq!(Some(t.to_string()), result);
    }
}

#[test]
fn scan_units() {
    static PFX: &[&str] = &[
        "y", "z", "a", "f", "p", "n", "µ", "m", "c", "d",
        "", // no multiplier prefix, raw unit
        "da", "h", "k", "M", "G", "T", "P", "E", "Z", "Y"
    ];
    static UNITS: &[&str] = &[
        "s", "m", "g", "A", "K", "mol", "cd",
        "rad", "sr", "Hz", "N", "Pa", "J", "W", "C", "V", "F", "Ω", "S",
        "Wb", "T", "H", "°C", "lm", "lx", "Bq", "Gy", "Sv", "kat",
    ];
    for prefix in PFX {
        for unit_base in UNITS {
            let unit = format!("{}{}", prefix, unit_base);
            let result = Scanner::new(unit.chars()).scan_unit();
            assert_eq!(result, Some((prefix.to_string(), unit_base.to_string())));
        }
    }
}

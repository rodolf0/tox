use crate::scanner::Scanner;

#[test]
fn scan_math_ops() {
    let tests = vec![
        "<", "<=", "=", "==", ">=", ">", "(", ")", ",", "*",
        "**", "^", "!", "+", "-", "/", "%", ":=",
    ];
    for t in tests.iter() {
        let result = Scanner::new(t.chars()).scan_math_op();
        assert_eq!(Some(t.to_string()), result);
    }
    // Negative tests
    let result = Scanner::new(":".chars()).scan_math_op();
    assert_eq!(result, None);
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

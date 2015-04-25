#![cfg(test)]

use mathscanner::MathScanner;

#[test]
fn test_numbers() {
    let expect = [
        "0", "10", "-0", "-7", "-10", "987654321", "0.34", "-2.14", "2e3", "-3e1", "5.3E2",
        "2.65i", "9i", "8.2e-3i",
        "1.234e2", "-3.4523e+1", "256E-2", "354e-4", "-3487.23e-1", "0.001e+5", "-9e-2",
    ];
    let nums = expect.connect(" ");

    let mut m = MathScanner::from_str(&nums);
    for exnum in expect.iter() {
        m.ignore_ws();
        let num = m.scan_number().unwrap();
        assert_eq!(num, *exnum);
    }
    m.ignore_ws();
    assert_eq!(m.curr(), None);
}

#[test]
fn test_exnums() {
    let expect = ["0x0", "0x10", "0x20", "0xff", "0xabcdEf", "0b0101"];
    let nums = expect.connect(" ");
    let mut m = MathScanner::from_str(&nums);
    for exnum in expect.iter() {
        m.ignore_ws();
        let num = m.scan_exotic_int().unwrap();
        assert_eq!(num, *exnum);
    }
    m.ignore_ws();
    assert_eq!(m.curr(), None);
}

#[test]
fn test_mixed() {
    let expect = [("0", "number"),
                  ("0b10", "exint"),
                  ("_id", "id"),
                  ("-0", "number"),
                  ("-7", "number"),
                  ("word", "id"),
                  ("-10", "number"),
                  ("987654321", "number"),
                  ("0.34", "number"),
                  ("test", "id"),
                  ("-2.14", "number"),
                  ("2e3", "number"),
                  ("-3e1", "number"),
                  ("0x34", "exint"),
                  ("5.3E2", "number")];
    let mixed = expect.iter().map(|&(num, _)| num)
                      .collect::<Vec<&str>>().connect(" ");

    let mut m = MathScanner::from_str(&mixed);
    for &(tok, typ) in expect.iter() {
        m.ignore_ws();
        match typ {
            "number" => assert_eq!(m.scan_number().unwrap(), tok),
            "exint" => assert_eq!(m.scan_exotic_int().unwrap(), tok),
            "id" => assert_eq!(m.scan_id().unwrap(), tok),
            _ => unreachable!()
        }
    }
    assert_eq!(m.curr(), None);
}

#[test]
fn test_misc() {
    let expect = [("0", "number"),
                  (",", "?"),
                  ("0b10", "exint"),
                  ("|", "?"),
                  ("_id", "id"),
                  ("-0", "number"),
                  ("-7", "number"),
                  ("+", "?"),
                  ("word", "id"),
                  ("-10", "number"),
                  ("*", "?"),
                  ("987654321", "number")];
    let mixed = "0,0b10|_id -0 -7+word -10*987654321,";

    let mut m = MathScanner::from_str(&mixed);
    for &(tok, typ) in expect.iter() {
        m.ignore_ws();
        match typ {
            "number" => assert_eq!(m.scan_number().unwrap(), tok),
            "exint" => assert_eq!(m.scan_exotic_int().unwrap(), tok),
            "id" => assert_eq!(m.scan_id().unwrap(), tok),
            "?" => assert_eq!(m.next().unwrap(), tok.chars().next().unwrap()),
            _ => unreachable!()
        }
    }
    assert_eq!(m.curr(), None);
}

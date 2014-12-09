extern crate tox;

#[cfg(not(test))]
fn main() {
    use std::io;

    let nums = concat!(
        "0 10 -0 -7 -10 987654321 0.34 -2.14 2e3 -3e1 5.3E2 ",
        "2.65i 9i 8.2e-3i ",
        "1.234e2 -3.4523e+1 256E-2 354e-4 -3487.23e-1 0.001e+5 -9e-2 ");
    let expect = [
        "0", "10", "-0", "-7", "-10", "987654321", "0.34", "-2.14", "2e3", "-3e1", "5.3E2",
        "2.65i", "9i", "8.2e-3i",
        "1.234e2", "-3.4523e+1", "256E-2", "354e-4", "-3487.23e-1", "0.001e+5", "-9e-2",
    ];

    let b = io::MemReader::new(nums.into_string().into_bytes());
    let mut m = tox::matchers::Matcher::new(b);

    for exnum in expect.iter() {
        m.ignore_ws();
        match m.match_number() {
            None => panic!("Error parsing {}", *exnum),
            Some(n) => println!("Parsed [{}] for [{}]", n, *exnum)
        }
    }
}

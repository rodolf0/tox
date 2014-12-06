extern crate tox;

fn main() {
    use std::io;
    let b = io::MemReader::new(b"a buffer with some numbers 0234 234 0912".to_vec());
    let mut m = tox::matchers::Matcher::new(b);

    loop {
        //assert!(m.skip_ws().is_ok());
        match m.match_id() {
            Err(e) => {
                println!("{}", e);
                break;
            },
            Ok(true) => println!("{}", m.extract()),
            Ok(false) => println!("ignoring {}", m.next()),
        }
    }
}

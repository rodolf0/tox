extern crate chrono;
extern crate mallard;

fn main() {
    if std::env::args().len() < 1 {
        println!("usage: ktime <time-expr>");
        return;
    }
    let input = std::env::args().skip(1).collect::<Vec<String>>().join(" ");
    let reftime = chrono::Local::now().naive_local();
    let tm = mallard::TimeMachine::new();
    match tm.parse_time(reftime, &input) {
        Some(time) => println!("{:?}", time),
        None => println!("Parse error")
    }
}

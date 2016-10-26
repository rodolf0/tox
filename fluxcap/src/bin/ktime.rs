extern crate chrono;
extern crate fluxcap;
extern crate kronos;

fn main() {
    if std::env::args().len() < 1 {
        println!("usage: ktime <time-expr>");
        return;
    }
    let input = std::env::args().skip(1)
        .filter(|arg| arg != "-v")
        .collect::<Vec<String>>().join(" ");
    let reftime = chrono::Local::now().naive_local();
    let tm = fluxcap::TimeMachine::new();
    let verbose = std::env::args()
        .filter(|arg| arg == "-v").count() > 0;
    if verbose {
        tm.print_trees(&input);
    }

    match tm.parse_time(reftime, &input) {
        Some(time) => {
            let t0 = time.start.format("%a, %b %e %Y");
            let t1 = time.end.format("%a, %b %e %Y");
            if time.grain == kronos::Granularity::Day {
                println!("{}", t0.to_string())
            } else {
                println!("{:?}: {} - {}", time.grain,
                         t0.to_string(), t1.to_string())
            }
        }
        None => match tm.time_diff(reftime, &input) {
            Some(time) => println!("{}", time),
            None => println!("Parse error")
        }
    }
}

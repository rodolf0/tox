extern crate chrono;
extern crate fluxcap;
extern crate kronos;

use std::io;

fn main() {
    let input = if std::env::args().len() <= 1 {
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).ok();
        buffer.pop();
        buffer
    } else {
        std::env::args()
            .skip(1)
            .filter(|arg| arg != "-v")
            .collect::<Vec<String>>()
            .join(" ")
    };

    let reftime = chrono::Local::now().naive_local();
    let tm = fluxcap::TimeMachine::new(reftime);

    fn fmt(grain: kronos::Grain) -> &'static str {
        use kronos::Grain::*;
        match grain {
            Second => "%A, %e %B %Y %H:%M:%S",
            Minute => "%A, %e %B %Y %H:%M",
            Hour => "%A, %e %B %Y %Hhs",
            Day | Week => "%A, %e %B %Y",
            Month | Quarter | Half => "%B %Y",
            Year | Lustrum | Decade | Century | Millenium => "%Y",
        }
    }

    for r in tm.eval(&input) {
        match &r {
            &fluxcap::TimeEl::Time(ref r) if r.grain <= kronos::Grain::Day =>
                println!("({:?}) {}", r.grain, r.start.format(fmt(r.grain))),
            &fluxcap::TimeEl::Time(ref r) =>
                println!("({:?}) {} - {}", r.grain,
                         r.start.format(fmt(r.grain)),
                         r.end.format(fmt(r.grain))),
            _ => println!("{:?}", r),
        }
    }

    let verbose = std::env::args().any(|arg| arg == "-v");
    if verbose {
        match fluxcap::debug_time_expression(&input) {
            Err(e) => eprintln!("TimeMachine {:?} for '{}'", e, input),
            Ok(trees) => for t in trees {
                println!("{}", t.print());
            }
        }
    }
}

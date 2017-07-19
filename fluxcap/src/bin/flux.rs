extern crate chrono;
extern crate fluxcap;
extern crate kronos;

fn main() {
    if std::env::args().len() < 1 {
        println!("usage: flux <time-expr>");
        return;
    }
    let input = std::env::args().skip(1).filter(|arg| arg != "-v")
        .collect::<Vec<String>>().join(" ");
    let reftime = chrono::Local::now().naive_local();
    let tm = fluxcap::TimeMachine::new();

    fn fmt(grain: kronos::Grain) -> &'static str {
        use kronos::Grain::*;
        match grain {
            Second => "%A, %e %B %Y %H:%M:%S",
            Minute => "%A, %e %B %Y %H:%M",
            Hour => "%A, %e %B %Y %Hhs",
            Day => "%A, %e %B %Y",
            Week => "%A, %e %B %Y",
            Month => "%B %Y",
            Quarter => "%B %Y",
            Year => "%Y",
        }
    }

    for r in tm.eval(reftime, &input) {
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
}
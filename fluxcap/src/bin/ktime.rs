extern crate chrono;
extern crate fluxcap;
extern crate kronos;

use std::path::Path;

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

    let traindata = fluxcap::load_training(&Path::new("time.train")).unwrap();
    let w = fluxcap::learn(fluxcap::build_grammar(), &traindata);

    tm.rankedparse(reftime, &input, &w);

    //match tm.oneparse(reftime, &input, &w) {
        //Some(time) => {
            //let t0 = time.start.format("%a, %b %e %Y");
            //let t1 = time.end - chrono::Duration::nanoseconds(1);
            //let t1 = t1.format("%a, %b %e %Y");
            //if time.grain == kronos::Granularity::Day {
                //println!("{}", t0.to_string())
            //} else {
                //println!("{:?}: {} - {}", time.grain,
                         //t0.to_string(), t1.to_string())
            //}
        //},
        //None => match tm.time_diff(reftime, &input) {
            //Some(time) => println!("{}", time),
            //None => println!("Parse error")
        //}
    //}
}

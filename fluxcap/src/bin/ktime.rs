extern crate chrono;
extern crate fluxcap;
extern crate kronos;

fn main() {
    if std::env::args().len() < 1 {
        println!("usage: ktime <time-expr>");
        return;
    }
    let verbose = std::env::args().filter(|arg| arg == "-v").count() > 0;
    let input = std::env::args().skip(1).filter(|arg| arg != "-v")
        .collect::<Vec<String>>().join(" ");
    let reftime = chrono::Local::now().naive_local();
    let tm = fluxcap::TimeMachine::new();
    if verbose {
        for t in tm.parse(&input) {
            t.print();
        }
    }

    //use std::path::Path;
    //let traindata = fluxcap::load_training(&Path::new("time.train")).unwrap();
    //let w = fluxcap::learn(fluxcap::build_grammar(), &traindata);

    println!("{:?}", tm.eval(reftime, &input));
}

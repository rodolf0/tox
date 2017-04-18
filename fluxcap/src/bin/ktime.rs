extern crate chrono;
extern crate fluxcap;
extern crate kronos;

fn main() {
    if std::env::args().len() < 1 {
        println!("usage: ktime <time-expr>");
        return;
    }
    let input = std::env::args().skip(1).filter(|arg| arg != "-v")
        .collect::<Vec<String>>().join(" ");
    let reftime = chrono::Local::now().naive_local();
    let tm = fluxcap::TimeMachine::new();

    for r in tm.eval(reftime, &input) {
        println!("{:?}", r);
    }

    //use std::path::Path;
    //let traindata = fluxcap::load_training(&Path::new("time.train")).unwrap();
    //let w = fluxcap::learn(fluxcap::build_grammar(), &traindata);

    //println!("{:?}", tm.eval(reftime, &input));
}

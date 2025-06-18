extern crate chrono;
extern crate fluxcap;
extern crate kronos;

use std::io;

fn main() -> Result<(), String> {
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

    for r in tm.eval(&input)? {
        println!("{}", r);
    }

    let verbose = std::env::args().any(|arg| arg == "-v");
    if verbose {
        match fluxcap::debug_time_expression(&input) {
            Err(error) => eprintln!("{}", error),
            Ok(trees) => for t in trees {
                println!("{}", t.print());
            }
        }
    }
    Ok(())
}

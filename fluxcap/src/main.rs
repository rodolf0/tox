// TODO: remove this and put it in tools
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

    let tm = fluxcap::TimeMachine::new();
    let verbose = std::env::args().any(|arg| arg == "-v");

    for r in tm.eval(&input, None)? {
        if verbose {
            println!("{:?}", r);
        } else {
            match r {
                fluxcap::TimeResult::Count(c) => {
                    let total = if c.total.fract() == 0.0 {
                        format!("{:.0}", c.total)
                    } else {
                        format!("{:.2}", c.total)
                    };
                    println!("{} {}", total, c.unit);
                }
                fluxcap::TimeResult::Span(s) => {
                    println!("({:?}) {} -> {}", s.grain, s.start, s.end);
                }
            }
        }
    }

    if verbose {
        match tm.parse_sexpr(&input) {
            Err(error) => eprintln!("{}", error),
            Ok(trees) => for t in trees {
                println!("{}", t.print());
            }
        }
    }
    Ok(())
}

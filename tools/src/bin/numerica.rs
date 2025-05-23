extern crate numerica;

fn main() -> Result<(), String> {
    let parser = numerica::parser()?;

    if std::env::args().len() > 1 {
        let input = std::env::args().skip(1).collect::<Vec<String>>().join(" ");
        match parser(input.as_str()) {
            Err(e) => println!("Parse err: {:?}", e),
            Ok(expr) => {
                if numerica::is_stochastic(&expr) {
                    let mut ctx = numerica::Context::new();
                    if let Err(_) = numerica::plot_histogram(&expr, &mut ctx) {
                        println!("{}", expr);
                    }
                } else {
                    println!("{}", expr);
                };
            }
        }
        return Ok(());
    }

    use rustyline::error::ReadlineError;
    let mut rl = rustyline::DefaultEditor::new().map_err(|e| e.to_string())?;
    let mut ctx = numerica::Context::new();
    loop {
        match rl.readline("~> ") {
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => return Ok(()),
            Err(e) => return Err(format!("Readline err: {:?}", e)),
            Ok(line) => match parser(line.as_str()) {
                Err(e) => println!("Parse err: {:?}", e),
                Ok(expr) => {
                    // Debug
                    // let _ = numerica::expr_tree(line.as_str());
                    let _ = rl.add_history_entry(&line);
                    match numerica::evaluate(expr, &mut ctx) {
                        Err(e) => println!("Eval err: {:?}", e),
                        Ok(expr) => {
                            if numerica::is_stochastic(&expr) {
                                let mut ctx = numerica::Context::new();
                                if let Err(_) = numerica::plot_histogram(&expr, &mut ctx) {
                                    println!("{}", expr);
                                }
                            } else {
                                println!("{}", expr);
                            };
                        }
                    }
                }
            },
        }
    }
}

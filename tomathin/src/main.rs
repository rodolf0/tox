extern crate tomathin;

fn main() -> Result<(), String> {
    let parser = tomathin::parser()?;

    if std::env::args().len() > 1 {
        let input = std::env::args().skip(1).collect::<Vec<String>>().join(" ");
        match parser(input.as_str()) {
            Err(e) => println!("Parse err: {:?}", e),
            Ok(expr) => println!("{:?}", expr),
        }
        return Ok(());
    }

    use rustyline::error::ReadlineError;
    let mut rl = rustyline::DefaultEditor::new().map_err(|e| e.to_string())?;
    loop {
        match rl.readline("~> ") {
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => return Ok(()),
            Err(e) => return Err(format!("Readline err: {:?}", e)),
            Ok(line) => match parser(line.as_str()) {
                Err(e) => println!("Parse err: {:?}", e),
                Ok(expr) => {
                    let _ = rl.add_history_entry(&line);
                    println!("{:?}", expr);
                }
            },
        }
    }
}

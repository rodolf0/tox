use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;

#[test]
fn integration_tests() {
    let bin = std::env::var("CARGO_BIN_EXE_lox").expect("CARGO_BIN_EXE_lox not set");
    let tests_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests");

    let mut failures = Vec::new();

    for entry in fs::read_dir(&tests_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("lox") {
            continue;
        }

        let stem = path.file_stem().unwrap().to_str().unwrap();
        let out_path = path.with_file_name(format!("{}.out", stem));
        let err_path = path.with_file_name(format!("{}.err", stem));

        let has_out = out_path.exists();
        let has_err = err_path.exists();

        if !has_out && !has_err {
            continue;
        }

        let (tx, rx) = std::sync::mpsc::channel();
        let bin = bin.clone();
        let path = path.clone();
        std::thread::spawn(move || {
            let output = Command::new(&bin)
                .arg(&path)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output();
            let _ = tx.send(output);
        });

        let output = match rx.recv_timeout(Duration::from_secs(30)) {
            Ok(Ok(output)) => output,
            Ok(Err(e)) => {
                failures.push(format!("{}: failed to spawn process: {}", stem, e));
                continue;
            }
            Err(_) => {
                failures.push(format!("{}: timed out after 30 seconds", stem));
                continue;
            }
        };

        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

        if has_out {
            let expected = fs::read_to_string(&out_path).unwrap().trim().to_string();
            if stdout != expected {
                failures.push(format!(
                    "{}: stdout mismatch\nexpected:\n{}\nactual:\n{}",
                    stem, expected, stdout
                ));
            }
        }

        if has_err {
            let expected = fs::read_to_string(&err_path).unwrap().trim().to_string();
            if stderr != expected {
                failures.push(format!(
                    "{}: stderr mismatch\nexpected:\n{}\nactual:\n{}",
                    stem, expected, stderr
                ));
            }
        }
    }

    if !failures.is_empty() {
        panic!("Integration test failures:\n\n{}", failures.join("\n\n"));
    }
}

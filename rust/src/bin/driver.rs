//! The rust edition's driver: one json request per line on stdin, one json response per line on stdout.

use serde_json::{Value, json};
use std::io::{self, BufRead, Write};
use zootella_ergo::driver;

fn main() {
    driver::quiet_panics();
    let stdin = io::stdin();
    let mut stdout = io::stdout().lock();
    for line in stdin.lock().lines() {
        let line = line.unwrap_or_else(|error| stop(format!("stdin isn't readable: {error}")));
        let request: Value = serde_json::from_str(&line)
            .unwrap_or_else(|error| stop(format!("not a request: {error}"))); // outside the top gate: a line the driver can't read is a bug in the harness, not a call to refuse, so it stops loudly
        let id = request["id"].clone();
        let call = request["call"].as_str().unwrap_or("");
        let args = request["args"].as_array().cloned().unwrap_or_default();
        let response = match driver::dispatch(call, &args) {
            Ok(result) => json!({"id": id, "result": result}),
            Err(error) => json!({"id": id, "error": error}),
        };
        writeln!(stdout, "{response}").expect("stdout is writable");
        stdout.flush().expect("stdout is writable");
    }
}

/// Stop the driver loudly, saying why on stderr, since quiet_panics has silenced what a panic would say.
fn stop(message: String) -> ! {
    eprintln!("{message}");
    std::process::exit(1)
}

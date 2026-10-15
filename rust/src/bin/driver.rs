//! The rust edition's driver: one json request per line on stdin, one json response per line on stdout.

use serde_json::{Value, json};
use std::io::{self, BufRead, Write};
use zootella_ergo::driver;

fn main() {
    driver::quiet_panics();
    let stdin = io::stdin();
    let mut stdout = io::stdout().lock();
    for line in stdin.lock().lines() {
        let line = line.expect("stdin is readable");
        let request: Value = serde_json::from_str(&line).unwrap_or(Value::Null); // a malformed line dispatches as an unknown function below
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

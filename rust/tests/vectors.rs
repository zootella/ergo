//! The vector reader: run every case in the shared test vectors against this edition.

use serde_json::Value;
use std::fs;

#[test]
fn vectors() {
    let directory = concat!(env!("CARGO_MANIFEST_DIR"), "/../tests"); // the shared vectors beside this edition
    let mut count = 0;
    let mut failures = Vec::new();
    for entry in fs::read_dir(directory).expect("the shared tests directory is beside this crate") {
        let path = entry.expect("readable directory entry").path();
        if path.extension().is_none_or(|e| e != "json") {
            continue;
        }
        let text = fs::read_to_string(&path).expect("readable vector file");
        let cases: Vec<Value> = serde_json::from_str(&text).expect("vector file is a json array");
        for c in cases {
            count += 1;
            let call = c["function"].as_str().expect("case names a function");
            let args = c["input"].as_array().expect("case has an input array");
            let throws = c["throws"].as_bool().unwrap_or(false);
            let got = zootella_ergo::driver::dispatch(call, args);
            let ok = if throws {
                got.as_ref()
                    .is_err_and(|e| !e.starts_with("unknown function")) // an unknown function refuses everything, so it can't pass a throws vector
            } else {
                got.as_ref().is_ok_and(|v| *v == c["output"])
            };
            if !ok {
                let expected = if throws {
                    "a panic".to_string()
                } else {
                    c["output"].to_string()
                };
                failures.push(format!(
                    "{call}({args:?}) {} expected {expected}, got {got:?}",
                    c["note"].as_str().unwrap_or("")
                ));
            }
        }
    }
    assert!(count > 0, "no vectors found in {directory}");
    assert!(
        failures.is_empty(),
        "{} of {count} vectors failed:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

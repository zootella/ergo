//! The fuzz test: random ticks through both text forms, as fast as this edition can, for the number of seconds ERGO_FUZZ names.
//!
//! Skipped otherwise, so cargo test stays deterministic and fast. Run with ERGO_FUZZ=4 cargo test --release --test fuzz -- --nocapture, where --release measures the edition as it ships and --nocapture shows the count it reached.

use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use zootella_ergo::{has_tag, iso_to_tick, tag, text_to_tick, tick_to_iso, tick_to_text, time};

/// The seconds ERGO_FUZZ names, or zero when it doesn't, which each test takes as its cue to return without running.
fn fuzz_seconds() -> f64 {
    std::env::var("ERGO_FUZZ")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0)
}

/// Centers and radii from cold3: a broad sweep of decades, leap day, each unit boundary at a narrow and then a wide aperture, and the two edges of the range.
const CENTERS: [(&str, i64); 14] = [
    ("2005", 30 * time::YEAR),
    ("2000feb29", 3 * time::DAY),
    ("2000", time::HOUR),
    ("2000mar", time::HOUR),
    ("2000jan1.12", 6 * time::HOUR),
    ("2000jan1.1230", 30 * time::MINUTE),
    ("2000jan1.1230.30000", 30 * time::SECOND),
    ("2000", 15 * time::DAY),
    ("2000mar", 15 * time::DAY),
    ("2000jan1.12", 15 * time::DAY),
    ("2000jan1.1230", 12 * time::HOUR),
    ("2000jan1.1230.30000", 30 * time::MINUTE),
    ("1970", time::DAY),
    ("9999dec31", time::DAY),
];

#[test]
fn fuzz_tick() {
    let seconds = fuzz_seconds();
    if seconds <= 0.0 {
        return; // skipped: set ERGO_FUZZ to a number of seconds
    }
    let budget = Duration::from_secs_f64(seconds / CENTERS.len() as f64); // time to spend on each center
    let last = text_to_tick("9999dec31.2359.59999").expect("the ceiling is a tick"); // so a tick drawn near an edge center stays a tick
    let mut x = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("the clock is past 1970")
        .as_nanos() as u64
        | 1; // xorshift state, seeded from the clock so every run differs, and odd so it's never zero
    let mut cycles = 0u64;
    for (center, radius) in CENTERS {
        let c = text_to_tick(center).expect("every center is a tick");
        let stop = Instant::now() + budget;
        while Instant::now() < stop {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            let t = (c + (x % (2 * radius as u64 + 1)) as i64 - radius).clamp(0, last); // a random tick within the radius, clamped to the range of ticks
            assert_eq!(text_to_tick(&tick_to_text(t)), Ok(t));
            assert_eq!(iso_to_tick(&tick_to_iso(t)), Ok(t));
            cycles += 1;
        }
    }
    println!(
        "rust fuzzed {cycles} ticks through both text forms in {seconds} seconds, {} a second",
        (cycles as f64 / seconds).round()
    );
}

/// A hot loop making tags: the smoke test that this edition makes them as fast as it can.
#[test]
fn fuzz_tag() {
    let seconds = fuzz_seconds();
    if seconds <= 0.0 {
        return; // skipped: set ERGO_FUZZ to a number of seconds
    }
    let stop = Instant::now() + Duration::from_secs_f64(seconds);
    let mut count = 0u64;
    while Instant::now() < stop {
        assert!(has_tag(&tag()));
        count += 1;
    }
    println!(
        "rust made {count} tags in {seconds} seconds, {} a second, each checked with has_tag",
        (count as f64 / seconds).round()
    );
}

//! The fuzz test: random ticks through both text forms and say_tick, as fast as this edition can, for the number of seconds ERGO_FUZZ names.
//!
//! Skipped otherwise, so cargo test stays deterministic and fast. Run with ERGO_FUZZ=4 cargo test --release --test fuzz -- --nocapture, where --release measures the edition as it ships and --nocapture shows the count it reached.

use std::hint::black_box;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use zootella_ergo::{
    Blake3, Data, Sha1, Sha256, blake3, has_tag, iso_to_tick, say_tick, sha1, sha256, size, tag,
    text_to_tick, tick_to_iso, tick_to_text, time,
};

/// The seconds ERGO_FUZZ names, or zero when it doesn't, which each test takes as its cue to return without running.
fn fuzz_seconds() -> f64 {
    std::env::var("ERGO_FUZZ")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0)
}

/// Centers and radii from the tick's original fuzz test: a broad sweep of decades, leap day, each unit boundary at a narrow and then a wide aperture, and the two edges of the range.
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

/// A template that says every field of a tick, the weekday among them.
const EVERY_FIELD: &str = "Dddd YYYY-MM-DD hh12:mm:ss.ttt AA";

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
            black_box(say_tick(t, EVERY_FIELD)); // and say it with every field, as the fuzz across the editions does, which is where what it says gets compared
            cycles += 1;
        }
    }
    println!(
        "rust fuzzed {cycles} ticks through both text forms and say_tick in {seconds} seconds, {} a second",
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

/// Each hash as fast as this edition can, on one mebibyte of random bytes again and again, and once a piece at a time to check it agrees.
#[test]
fn fuzz_hash() {
    let seconds = fuzz_seconds();
    if seconds <= 0.0 {
        return; // skipped: set ERGO_FUZZ to a number of seconds
    }
    let data = Data::random(size::MB as usize); // one mebibyte of random bytes from the operating system's random source, hashed again and again
    // each hash by name, all at once, and in two pieces at a split that lines up with nothing
    type Hash = fn(&Data) -> Data;
    let hashes: [(&str, Hash, Hash); 3] = [
        ("sha1", sha1, |d| {
            let mut h = Sha1::new();
            h.update(&d.clip_view(0, 1000));
            h.update(&d.clip_view(1000, d.size()));
            h.finalize()
        }),
        ("sha256", sha256, |d| {
            let mut h = Sha256::new();
            h.update(&d.clip_view(0, 1000));
            h.update(&d.clip_view(1000, d.size()));
            h.finalize()
        }),
        ("blake3", blake3, |d| {
            let mut h = Blake3::new();
            h.update(&d.clip_view(0, 1000));
            h.update(&d.clip_view(1000, d.size()));
            h.finalize()
        }),
    ];
    let share = seconds / hashes.len() as f64; // an even share of the seconds for each hash
    let mut said = Vec::new();
    for (name, all, pieces) in hashes {
        let stop = Instant::now() + Duration::from_secs_f64(share);
        let mut count = 0u64;
        let last = loop {
            let h = black_box(all(black_box(&data))); // black_box on both sides, so the optimizer can neither hash the same input once nor drop a result it sees unused
            count += 1;
            if Instant::now() >= stop {
                break h; // at least once, however few the seconds
            }
        };
        // and once a piece at a time, at a split that lines up with nothing, which must agree
        assert_eq!(
            pieces(&data),
            last,
            "{name} disagreed with itself a piece at a time"
        );
        said.push(format!(
            "{name} {} MiB a second",
            (count as f64 / share).round()
        ));
    }
    println!(
        "rust hashed one mebibyte again and again: {}",
        said.join(", ")
    );
}

//! The fuzz tests: random ticks through both text forms and say_tick, and random tags, hashes, and outlines, as fast as this edition can, for the number of seconds ERGO_FUZZ names.
//!
//! Skipped otherwise, so cargo test stays deterministic and fast. Run with ERGO_FUZZ=4 cargo test --release --test fuzz -- --nocapture, where --release measures the edition as it ships and --nocapture shows the count it reached.

use std::hint::black_box;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use zootella_ergo::{
    Blake3, Data, Outline, Sha1, Sha256, blake3, data_to_outline, data_to_quoted, has_tag,
    iso_to_tick, say_tick, sha1, sha256, size, span_decode, span_encode, tag, text_to_outline,
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

/// Xorshift, seeded from the clock so every run differs, the random source the outline fuzz tests draw from.
struct Random(u64);

impl Random {
    /// A new generator, seeded from the clock, and odd so it's never zero.
    fn new() -> Random {
        Random(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("the clock is past 1970")
                .as_nanos() as u64
                | 1,
        )
    }
    /// A number from 0 up to n.
    fn below(&mut self, n: usize) -> usize {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 % n as u64) as usize
    }
    /// True about as often as the chance, out of 1.
    fn chance(&mut self, p: f64) -> bool {
        (self.below(1 << 20) as f64) < p * (1 << 20) as f64
    }
    /// One of the things in a list.
    fn pick<'a, T>(&mut self, a: &'a [T]) -> &'a T {
        &a[self.below(a.len())]
    }
    /// The indexes 0 through n - 1 in a random order.
    fn shuffled(&mut self, n: usize) -> Vec<usize> {
        let mut a: Vec<usize> = (0..n).collect();
        for i in (1..n).rev() {
            a.swap(i, self.below(i + 1));
        }
        a
    }
}

/// The names random outlines take, a few of them, blank most of all, so siblings share names and ties come up.
const OUTLINE_NAMES: [&str; 9] = ["", "", "", "a", "a", "b", "ab", "a0", "z9"];
/// And the values, as base16: none most of all, and ones that tie, that begin one another, and that are the characters quoted encoding treats with care, a quote, a space, and a number sign.
const OUTLINE_VALUES: [&str; 10] = ["", "", "", "00", "01", "61", "6161", "22", "20", "23"];
/// And the characters of a value that reads as text, with the ones a text outline treats with care.
const OUTLINE_CHARACTERS: [char; 8] = ['a', 'b', ':', '#', ' ', '"', '\t', '\n'];

/// A random value: often one from the list above, sometimes text, sometimes random bytes.
fn random_value(r: &mut Random) -> Data {
    if r.chance(0.6) {
        return Data::from_base16(r.pick(&OUTLINE_VALUES)).expect("base16");
    }
    if r.chance(0.5) {
        let s: String = (0..r.below(12))
            .map(|_| *r.pick(&OUTLINE_CHARACTERS))
            .collect();
        return Data::from_text(&s);
    }
    Data::random(r.below(40))
}

/// A random outline, its contents nested down at most depth levels.
fn random_outline(r: &mut Random, depth: usize) -> Outline {
    let mut o = Outline::new(r.pick(&OUTLINE_NAMES));
    o.set_value(random_value(r));
    if depth > 0 {
        for _ in 0..r.below(5) {
            let deeper = (depth - 1).saturating_sub(r.below(2));
            o.add(random_outline(r, deeper));
        }
    }
    o
}

/// An outline as plain text, in the order it holds its contents, to compare two outlines, or one before and after.
fn outline_shape(o: &Outline) -> String {
    let contents: String = (0..o.length()).map(|i| outline_shape(o.get(i))).collect();
    format!("[{:?} {:?} [{contents}]]", o.name(), o.value().base16())
}

/// A copy of an outline with its contents in a random order at every level.
fn shuffled_outline(r: &mut Random, o: &Outline) -> Outline {
    let mut c = Outline::new(o.name());
    c.set_value(o.value().clone());
    for i in r.shuffled(o.length()) {
        c.add(shuffled_outline(r, o.get(i)));
    }
    c
}

/// A value in quoted encoding a person might write instead: as the writer writes it, as base16 alone, or with quotable bytes quoted one at a time, or not.
fn sloppy_quoted(r: &mut Random, d: &Data) -> String {
    if r.chance(0.4) {
        return data_to_quoted(d);
    }
    if r.chance(0.5) {
        return d.base16();
    }
    let mut s = String::new();
    for &b in d.array() {
        if (0x20..=0x7e).contains(&b) && b != 0x22 && r.chance(0.5) {
            s.push('"');
            s.push(char::from(b));
            s.push('"');
        } else {
            s.push_str(&format!("{b:02x}"));
        }
    }
    s
}

/// An outline as text the way a person might write it by hand: contents in any order, any indent, any quoting, trailing whitespace and comments, lines of comments, blank lines, and either line ending.
fn sloppy_text(r: &mut Random, o: &Outline) -> String {
    let line_separator = char::from_u32(0x2028).expect("a character");
    let ideographic_space = char::from_u32(0x3000).expect("a character");
    let endings = [
        String::new(),
        String::new(),
        " ".to_string(),
        "\t".to_string(),
        " #note".to_string(),
        "#".to_string(),
        " # \"quoted\" #more".to_string(),
        format!("\t#{line_separator}\r"),
    ];
    let blanks = [
        String::new(),
        "  ".to_string(),
        "\t".to_string(),
        ideographic_space.to_string(),
    ];
    let mut s = String::new();
    let mut stack = vec![(o, String::new())]; // a stack of our own, the outlines still to write and the indent of each
    while let Some((o, indent)) = stack.pop() {
        s.push_str(&format!(
            "{indent}{}:{}",
            o.name(),
            sloppy_quoted(r, o.value())
        ));
        s.push_str(r.pick(&endings));
        s.push_str(r.pick(&["\n", "\r\n"]));
        if r.chance(0.1) {
            s.push_str(&format!("{}\n", r.pick(&blanks))); // a blank line, of nothing or of whitespace
        }
        if r.chance(0.1) {
            let at = r.pick(&["", " ", "\t", indent.as_str()]).to_string();
            s.push_str(&format!(
                "{at}{}\n",
                r.pick(&["#", "# a note", "#x:\"commented out\""])
            )); // a line that's only a comment, at any indent
        }
        let unit = *r.pick(&["  ", "    ", "\t", " ", " \t"]); // each parent picks the indent its own contents share
        for i in r.shuffled(o.length()) {
            stack.push((o.get(i), format!("{indent}{unit}")));
        }
    }
    s
}

/// A copy of some bytes with one thing changed: a byte replaced, inserted, or deleted, or the end cut off.
fn mutated_data(r: &mut Random, d: &Data) -> Data {
    let mut a = d.array().to_vec();
    let i = r.below(a.len() + 1);
    let b = if r.chance(0.5) {
        r.below(256) as u8
    } else {
        *r.pick(&[0x00, 0x01, 0x30, 0x61, 0x62, 0x80])
    };
    let x = r.below(10);
    if x < 4 && !a.is_empty() {
        let at = i.min(a.len() - 1);
        a[at] = b;
    } else if x < 6 {
        a.insert(i, b);
    } else if x < 8 && !a.is_empty() {
        a.remove(i.min(a.len() - 1));
    } else {
        a.truncate(i);
    }
    Data::from(a)
}

/// And a copy of some text with one character replaced, inserted, or deleted, from the characters a text outline cares about.
fn mutated_text(r: &mut Random, s: &str) -> String {
    let mut a: Vec<char> = s.chars().collect();
    let i = r.below(a.len() + 1);
    let c = *r.pick(&[
        'a', ':', '"', ' ', '\t', '#', '\n', '0', 'A', '\r', '\u{a0}',
    ]);
    let x = r.below(10);
    if x < 4 && i < a.len() {
        a[i] = c;
    } else if x < 7 {
        a.insert(i, c);
    } else if i < a.len() {
        a.remove(i);
    }
    a.into_iter().collect()
}

/// A random outline round-trips through both forms, and writes the same bytes from any order and any hand-written text.
#[test]
fn fuzz_outline() {
    let seconds = fuzz_seconds();
    if seconds <= 0.0 {
        return; // skipped: set ERGO_FUZZ to a number of seconds
    }
    let mut r = Random::new();
    let stop = Instant::now() + Duration::from_secs_f64(seconds);
    let mut cycles = 0u64;
    while Instant::now() < stop {
        let mut o = random_outline(&mut r, 5);
        let before = outline_shape(&o);
        let d = o.data(); // the canonical bytes
        let t = o.text(); // and the canonical text
        assert_eq!(outline_shape(&o), before, "a writer changed the outline");
        let back = data_to_outline(&d).expect("the bytes read back");
        assert_eq!(back.data(), d, "{t}");
        assert_eq!(back.text(), t);
        assert_eq!(
            text_to_outline(&t).expect("the text reads back").data(),
            d,
            "{t}"
        );
        let mut shuffled = shuffled_outline(&mut r, &o);
        assert_eq!(shuffled.data(), d, "{t}");
        shuffled.sort();
        assert_eq!(outline_shape(&shuffled), outline_shape(&back), "{t}");
        o.sort();
        assert_eq!(outline_shape(&o), outline_shape(&back), "{t}");
        let sloppy = sloppy_text(&mut r, &o);
        assert_eq!(
            text_to_outline(&sloppy).map(|s| s.data()),
            Ok(d),
            "{sloppy:?}"
        );
        cycles += 1;
    }
    println!(
        "rust fuzzed {cycles} random outlines through both forms, in any order, and as hand-written text, in {seconds} seconds, {} a second",
        (cycles as f64 / seconds).round()
    );
}

/// The readers take only what reads back the same: changed bytes, text, and spans refuse, or read to a form that round-trips.
#[test]
fn fuzz_outline_readers() {
    let seconds = fuzz_seconds();
    if seconds <= 0.0 {
        return; // skipped: set ERGO_FUZZ to a number of seconds
    }
    let mut r = Random::new();
    let stop = Instant::now() + Duration::from_secs_f64(seconds);
    let mut cycles = 0u64;
    let mut refused = 0u64;
    while Instant::now() < stop {
        let o = random_outline(&mut r, 3);

        // the binary reader is strict: whatever bytes it takes are exactly the bytes its writer writes
        let d = mutated_data(&mut r, &o.data());
        match data_to_outline(&d) {
            Ok(from_data) => assert_eq!(from_data.data(), d, "{}", d.base16()),
            Err(_) => refused += 1, // refused, which is fine; the property is about what it takes
        }

        // the text reader is lenient, but whatever text it takes, it writes back as text that reads to the same outline
        let s = mutated_text(&mut r, &o.text());
        match text_to_outline(&s) {
            Ok(from_text) => {
                let t = from_text.text();
                let again = text_to_outline(&t).expect("its own text reads back");
                assert_eq!(again.text(), t, "{s:?}");
                assert_eq!(again.data(), from_text.data(), "{s:?}");
            }
            Err(_) => refused += 1,
        }

        // and a span is strict, too: a size writes as one set of bytes, and reads back from them, and no other bytes read to it
        let bits = r.below(29);
        let n = r.below(1 << bits); // as often small as large, up to the most four bytes hold
        let e = span_encode(n);
        assert_eq!(
            span_decode(&e, 0).map(|s| (s.value, s.bytes_read)),
            Ok((n, e.size()))
        );
        let x = Data::random(1 + r.below(5));
        match span_decode(&x, 0) {
            Ok(from_span) => assert_eq!(
                span_encode(from_span.value),
                x.clip_view(0, from_span.bytes_read),
                "{}",
                x.base16()
            ),
            Err(_) => refused += 1,
        }
        cycles += 1;
    }
    println!(
        "rust changed {cycles} random outlines' bytes and text, and read random spans, in {seconds} seconds, {} a second, and the readers refused {refused} of the {}",
        (cycles as f64 / seconds).round(),
        cycles * 3
    );
}

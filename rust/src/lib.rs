//! ergo, short for ergonomic: simple helper functions, the same in JavaScript, Python, Rust, and Go.

#[cfg(feature = "driver")]
#[doc(hidden)]
pub mod driver;

// size

/// Numbers of bytes in the binary units that memory and storage are measured in, 1024 to each rather than 1000, so `64 * size::KB` names what a bare 65536 would hide. Unsigned, like a file's length, because a count of bytes is never negative.
pub mod size {
    /// One byte, the unit.
    pub const B: u64 = 1;
    /// Kibibyte.
    pub const KB: u64 = 1024 * B;
    /// Mebibyte.
    pub const MB: u64 = 1024 * KB;
    /// Gibibyte.
    pub const GB: u64 = 1024 * MB;
    /// Tebibyte.
    pub const TB: u64 = 1024 * GB;
    /// Pebibyte, and the last: 2^50, still a safe integer in every edition.
    pub const PB: u64 = 1024 * TB;
}

/// Each 1024 of the one before, labeled the everyday way. Bytes come before these, and are the one unit that pluralizes.
const SIZE_UNITS: [&str; 8] = [" KB", " MB", " GB", " TB", " PB", " EB", " ZB", " YB"];

/// Say a count of bytes in four digits or less with a unit, like "1 byte", "2 bytes", or "117 MB". The count rounds down.
///
/// ```
/// assert_eq!(zootella_ergo::say_size4(1), "1 byte");
/// assert_eq!(zootella_ergo::say_size4(123_456_789), "117 MB");
/// ```
pub fn say_size4(n: u64) -> String {
    if n <= 9999 {
        let s = if n == 1 { "" } else { "s" }; // singular only for exactly one
        return format!("{n} byte{s}"); // at most four digits of bytes, so say bytes
    }
    let mut w = n; // whole units of the unit we're counting in
    for unit in SIZE_UNITS {
        w /= size::KB; // count in the next unit up
        if w <= 9999 {
            return format!("{w}{unit}"); // fits in four digits, so this is the unit to say it in
        }
    }
    n.to_string() // past yottabytes, which a u64 can't reach; the compiler still wants an answer here
}

// time

/// Spans of time in milliseconds, the unit of a tick, so `now + 2 * time::HOUR` is a deadline two hours out. Signed, like a tick, so a span subtracts as easily as it adds.
pub mod time {
    /// The unit; a tick counts milliseconds since 1970.
    pub const MILLISECOND: i64 = 1;
    /// A thousand of those.
    pub const SECOND: i64 = 1000 * MILLISECOND;
    /// Sixty seconds.
    pub const MINUTE: i64 = 60 * SECOND;
    /// Sixty minutes.
    pub const HOUR: i64 = 60 * MINUTE;
    /// Twenty-four hours.
    pub const DAY: i64 = 24 * HOUR;
    /// Seven days.
    pub const WEEK: i64 = 7 * DAY;
    /// 365 and a quarter days, the average over the leap cycle, which is exact in milliseconds.
    pub const YEAR: i64 = 365 * DAY + DAY / 4;
    /// A twelfth of that year, also exact.
    pub const MONTH: i64 = YEAR / 12;
}

// tick

// A tick is an integer count of milliseconds since the start of 1970, and ergo keeps every tick between zero and the last millisecond of 9999. Two text forms carry a tick exactly. The iso form is like "2012-08-06T05:17:57.300Z": always 24 characters, always milliseconds, always Z. The short form is like "2012aug6.0517.57300": the year, month, and day, then a dot and the clock, then a dot and the milliseconds into the minute, written only as deep as the moment needs, so the start of 1990 is just "1990" and the longest text is 20 characters.
//
// Each form reads back to exactly the tick that wrote it, and reads nothing else. A reader accepts the text its writer produces and refuses everything else, from a missing millisecond digit to "1990feb1" where the writer would say "1990feb". The last check in each reader is the round trip itself: parse the text, write the tick, and require the same text. That's what keeps every text one to one with its tick.
//
// Every function that takes a tick panics on a number that isn't one, because a caller handing it one has a bug. A number from outside, like a system clock or a file's modified time, isn't the caller's to vouch for, so has_tick asks about it first, and asking never panics; once has_tick says yes, nothing here panics on that tick. check_tick is the guard that enforces the same answer, and it's the gate every function here goes through. This matters more in Rust than in the languages that throw: a panic can't be caught the way an exception can, and one that starts inside a panic hook ends the process.
//
// The standard library has no calendar, only SystemTime, so the few lines of arithmetic between a count of days and a year, month, and day are here, and the shared vectors and the fuzz test pin them against the other three editions' platforms.

/// The last millisecond of 9999, which keeps a year at four digits in every edition.
const LAST_TICK: i64 = 253_402_300_799_999;
const MONTHS: [&str; 12] = [
    "jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec",
];

/// Text a reader refused, because it isn't exactly what a writer produces. Carries the text so a log can show it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError(pub String);

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not a text form of a tick: {:?}", self.0)
    }
}

impl std::error::Error for ParseError {}

/// The utc parts of a tick: the year, month 1 through 12, day 1 through 31, hour, minute, and milliseconds into the minute.
struct Parts {
    y: i64,
    m: u32,
    d: u32,
    h: u32,
    i: u32,
    ms: u32,
}

/// True if `t` is a tick, zero through the last millisecond of 9999. Never panics, so a number from outside, like a clock or a file's modified time, can be asked about before it's handed to a function that takes a tick.
///
/// ```
/// assert!(zootella_ergo::has_tick(1_344_230_277_300));
/// assert!(!zootella_ergo::has_tick(-1));
/// ```
pub fn has_tick(t: i64) -> bool {
    (0..=LAST_TICK).contains(&t)
}

/// A guard: pass a tick through, or panic because the caller handed us something that isn't one.
///
/// # Panics
///
/// Panics if `t` isn't a tick, which is a bug in the caller, not an ordinary answer to handle. Ask [`has_tick`] first about a number from outside.
pub fn check_tick(t: i64) -> i64 {
    assert!(has_tick(t), "not a tick: {t}");
    t
}

/// Split a tick into its utc parts.
///
/// # Panics
///
/// Panics if `t` isn't a tick, which is a bug in the caller: a tick is zero through the last millisecond of 9999.
fn split_tick(t: i64) -> Parts {
    let t = check_tick(t);
    let rest = (t % time::DAY) as u32; // milliseconds into the day
    let (y, m, d) = civil(t / time::DAY);
    Parts {
        y,
        m,
        d,
        h: rest / 3_600_000,
        i: rest / 60_000 % 60,
        ms: rest % 60_000,
    }
}

/// The tick at the given utc parts, or None if there's no such moment, like february 30, or it's outside the range of ticks.
fn join_tick(p: &Parts) -> Option<i64> {
    if !(1970..=9999).contains(&p.y)
        || !(1..=12).contains(&p.m)
        || !(1..=31).contains(&p.d)
        || p.h > 23
        || p.i > 59
        || p.ms > 59_999
    {
        return None;
    }
    let t = days(p.y, p.m, p.d) * time::DAY
        + i64::from(p.h) * time::HOUR
        + i64::from(p.i) * time::MINUTE
        + i64::from(p.ms);
    (0..=LAST_TICK).contains(&t).then_some(t)
}

/// The year, month, and day of a count of days since 1970, by Howard Hinnant's arithmetic: count from march 1 so leap days fall at the end of the year, in eras of 400 years, the leap cycle. Days here are never negative, since ticks start at 1970.
fn civil(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468; // days since 0000-03-01
    let era = z / 146_097; // eras of 400 years, 146097 days each
    let doe = (z - era * 146_097) as u32; // day of era, 0 through 146096
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365; // year of era, 0 through 399
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // day of year, counting from march 1
    let mp = (5 * doy + 2) / 153; // month counting from march, 0 through 11
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = i64::from(yoe) + era * 400 + if m <= 2 { 1 } else { 0 }; // january and february belong to the next year
    (y, m, d)
}

/// The count of days since 1970 of a year, month, and day, the same arithmetic in reverse. A day past the month's end rolls into the next month, and the round trip in the reader catches that.
fn days(y: i64, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y / 400;
    let yoe = (y - era * 400) as u32;
    let mp = if m > 2 { m - 3 } else { m + 9 };
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + i64::from(doe) - 719_468
}

/// The number a text of numerals 0 through 9 names, or None if it's blank, has anything else in it, or is too long for a u32.
fn numerals(s: &str) -> Option<u32> {
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    s.parse().ok()
}

/// Write a tick in the iso form, like "2012-08-06T05:17:57.300Z".
///
/// ```
/// assert_eq!(zootella_ergo::tick_to_iso(1_344_230_277_300), "2012-08-06T05:17:57.300Z");
/// ```
///
/// # Panics
///
/// Panics if `t` isn't a tick, which is a bug in the caller: a tick is zero through the last millisecond of 9999. Ask [`has_tick`] first about a number from outside.
pub fn tick_to_iso(t: i64) -> String {
    let p = split_tick(t);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
        p.y,
        p.m,
        p.d,
        p.h,
        p.i,
        p.ms / 1000,
        p.ms % 1000
    )
}

/// Read the iso form back to its tick, or a [`ParseError`] because the text isn't exactly what a tick writes.
///
/// ```
/// assert_eq!(zootella_ergo::iso_to_tick("2012-08-06T05:17:57.300Z"), Ok(1_344_230_277_300));
/// assert!(zootella_ergo::iso_to_tick("2012-08-06T05:17:57Z").is_err());
/// ```
pub fn iso_to_tick(s: &str) -> Result<i64, ParseError> {
    parse_iso(s)
        .filter(|&t| tick_to_iso(t) == s) // the round trip is the final check, catching february 30 after the arithmetic rolled it into march
        .ok_or_else(|| ParseError(s.to_string()))
}

/// Read the iso form leniently enough to compute a tick, or None; iso_to_tick above makes it strict with the round trip.
fn parse_iso(s: &str) -> Option<i64> {
    const SHAPE: &[u8] = b"0000-00-00T00:00:00.000Z"; // a 0 is a numeral, and anything else is itself
    if s.len() != SHAPE.len() || !s.is_ascii() {
        return None; // ascii, so byte positions below are character positions
    }
    for (c, want) in s.bytes().zip(SHAPE) {
        if (*want == b'0' && !c.is_ascii_digit()) || (*want != b'0' && c != *want) {
            return None;
        }
    }
    join_tick(&Parts {
        y: i64::from(numerals(&s[0..4])?),
        m: numerals(&s[5..7])?,
        d: numerals(&s[8..10])?,
        h: numerals(&s[11..13])?,
        i: numerals(&s[14..16])?,
        ms: numerals(&s[17..19])? * 1000 + numerals(&s[20..23])?,
    })
}

/// Write a tick in the short form, like "2012aug6.0517.57300", as deep as the moment needs and no deeper.
///
/// ```
/// assert_eq!(zootella_ergo::tick_to_text(1_344_230_277_300), "2012aug6.0517.57300");
/// assert_eq!(zootella_ergo::tick_to_text(631_152_000_000), "1990");
/// ```
///
/// # Panics
///
/// Panics if `t` isn't a tick, which is a bug in the caller: a tick is zero through the last millisecond of 9999. Ask [`has_tick`] first about a number from outside.
pub fn tick_to_text(t: i64) -> String {
    let p = split_tick(t);
    let mut depth = 0; // how deep the text must go: the year alone, then the month, day, hour, minute, and millisecond
    if p.m > 1 {
        depth = 1;
    }
    if p.d > 1 {
        depth = 2;
    }
    if p.h > 0 {
        depth = 3;
    }
    if p.i > 0 {
        depth = 4;
    }
    if p.ms > 0 {
        depth = 5;
    }
    let mut text = p.y.to_string();
    if depth >= 1 {
        text += MONTHS[p.m as usize - 1];
    }
    if depth >= 2 {
        text += &p.d.to_string();
    }
    if depth >= 3 {
        text += &format!(".{:02}", p.h);
    }
    if depth >= 4 {
        text += &format!("{:02}", p.i);
    }
    if depth >= 5 {
        text += &format!(".{}", p.ms);
    }
    text
}

/// Read the short form back to its tick, or a [`ParseError`] because the text isn't exactly what a tick writes.
///
/// ```
/// assert_eq!(zootella_ergo::text_to_tick("2012aug6.0517.57300"), Ok(1_344_230_277_300));
/// assert!(zootella_ergo::text_to_tick("1990feb1").is_err()); // the writer says "1990feb" for that moment
/// ```
pub fn text_to_tick(s: &str) -> Result<i64, ParseError> {
    parse_text(s)
        .filter(|&t| tick_to_text(t) == s) // the round trip is the final check, catching february 30 and a padded or deeper text alike
        .ok_or_else(|| ParseError(s.to_string()))
}

/// Read the short form leniently enough to compute a tick, or None; text_to_tick above makes it strict with the round trip.
fn parse_text(s: &str) -> Option<i64> {
    if !s.is_ascii() {
        return None; // ascii, so byte positions below are character positions
    }
    let parts: Vec<&str> = s.split('.').collect(); // the year, month, and day; then the clock; then the milliseconds into the minute
    if parts.len() > 3 {
        return None;
    }
    let p1 = parts[0];
    if ![4, 7, 8, 9].contains(&p1.len()) {
        return None; // "1990", "1990feb", "1990feb2", "1990feb28"
    }
    let mut p = Parts {
        y: i64::from(numerals(&p1[0..4])?),
        m: 1,
        d: 1,
        h: 0,
        i: 0,
        ms: 0,
    };
    if p1.len() >= 7 {
        p.m = MONTHS.iter().position(|&name| name == &p1[4..7])? as u32 + 1;
    }
    if p1.len() >= 8 {
        p.d = numerals(&p1[7..])?;
    }
    if parts.len() >= 2 {
        let p2 = parts[1];
        if p2.len() != 2 && p2.len() != 4 {
            return None; // "09" an hour, "0905" an hour and minute
        }
        p.h = numerals(&p2[0..2])?;
        if p2.len() == 4 {
            p.i = numerals(&p2[2..4])?;
        }
    }
    if parts.len() >= 3 {
        p.ms = numerals(parts[2])?;
    }
    join_tick(&p)
}

// tag

/// Long enough to be unique, short enough to be reasonable, and nanoid's default.
pub const TAG_LENGTH: usize = 21;

/// Letters and numerals only, so a double-click selects the whole tag; nanoid's default alphabet also has - and _.
const TAG_ALPHABET: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

/// Generate a new universally unique tag of 21 letters and numerals, like "AgKxru95C7jFp5iPuK9O7".
///
/// ```
/// let t = zootella_ergo::tag();
/// assert!(zootella_ergo::has_tag(&t));
/// ```
pub fn tag() -> String {
    // the algorithm is nanoid's: mask random bytes down to the alphabet's size, and skip the few values that land past its end, so every character is equally likely
    let mask = 63u8; // the alphabet has 62 characters; 63 is the smallest all-ones mask that covers them
    let step =
        (1.6 * f64::from(mask) * TAG_LENGTH as f64 / TAG_ALPHABET.len() as f64).ceil() as usize; // random bytes per draw; 1.6 is nanoid's margin so one draw almost always suffices
    let mut tag = String::with_capacity(TAG_LENGTH);
    let mut bytes = vec![0u8; step];
    loop {
        getrandom::fill(&mut bytes).expect("the operating system's random source is unavailable"); // the standard library has no random source yet, and getrandom is the crate the ecosystem's randomness rests on
        for b in &bytes {
            let i = usize::from(b & mask); // 0 through 63
            if i < TAG_ALPHABET.len() {
                tag.push(TAG_ALPHABET[i] as char); // 62 and 63 fall past the end; skipping them keeps the distribution uniform
            }
            if tag.len() == TAG_LENGTH {
                return tag;
            }
        }
    }
}

/// True if s is exactly 21 letters and numerals.
pub fn has_tag(s: &str) -> bool {
    s.len() == TAG_LENGTH && s.bytes().all(|b| b.is_ascii_alphanumeric()) // 21 bytes that are all ascii are 21 characters
}

/// A guard: pass a tag through, or panic because the caller handed us something that isn't one.
///
/// # Panics
///
/// Panics if `s` is not a tag, which is a bug in the caller, not an ordinary answer to handle.
pub fn check_tag(s: &str) -> &str {
    assert!(has_tag(s), "not a tag: {s:?}");
    s
}

// hash

// Three hashes, each the one the world already knows. SHA-1 is the oldest, the one older manifests name a file by and git names its objects with. It's broken for collisions, two different inputs made to share a hash on purpose, so use it to match a hash that already exists, never to protect anything. SHA-256 is the standard everywhere today. BLAKE3 is the newest, built for speed as a tree, so pieces of one input can hash on many cores at once, and it's the one to name a file by for years to come.
//
// Each hash comes in two forms that give the same answer. The function takes all the bytes at once, like sha256(data), for data already in memory. The hasher takes them a piece at a time, as a file or a stream delivers them: make one with Sha256::new(), call update with each piece in order, and call finalize for the hash. A file of any size hashes this way in the memory of one piece, and a single read of the file can feed all three hashers. Finalize doesn't finish a hasher, as in the blake3 crate: update it again and finalize again for the hash of everything so far. Each hasher is also an io::Write, so io::copy can feed one straight from a reader.
//
// The standard library has no cryptographic hashes, so SHA-1 and SHA-256 come from RustCrypto's sha1 and sha2, and BLAKE3 from the blake3 crate, the official implementation by BLAKE3's authors. Each edition takes each hash from its platform wherever the platform has it, and otherwise from the official implementation or the best known one, and the shared vectors and the fuzz test hold all four to the same answers.

/// The SHA-1 hash of `data`, 20 bytes. For matching a hash that already exists, like an older manifest's, since SHA-1 is broken for collisions.
///
/// ```
/// let h = zootella_ergo::sha1(b"abc");
/// assert_eq!(h[..4], [0xa9, 0x99, 0x3e, 0x36]);
/// ```
pub fn sha1(data: &[u8]) -> [u8; 20] {
    <sha1::Sha1 as sha1::Digest>::digest(data).into()
}

/// The SHA-256 hash of `data`, 32 bytes.
///
/// ```
/// let h = zootella_ergo::sha256(b"abc");
/// assert_eq!(h[..4], [0xba, 0x78, 0x16, 0xbf]);
/// ```
pub fn sha256(data: &[u8]) -> [u8; 32] {
    <sha2::Sha256 as sha2::Digest>::digest(data).into()
}

/// The BLAKE3 hash of `data`, 32 bytes.
///
/// ```
/// let h = zootella_ergo::blake3(b"abc");
/// assert_eq!(h[..4], [0x64, 0x37, 0xb3, 0xac]);
/// ```
pub fn blake3(data: &[u8]) -> [u8; 32] {
    blake3::hash(data).into()
}

/// A SHA-1 hasher, a piece at a time: [`update`](Sha1::update) with each piece in order, then [`finalize`](Sha1::finalize) for the same 20 bytes [`sha1`] gives all at once.
///
/// ```
/// let mut h = zootella_ergo::Sha1::new();
/// h.update(b"a");
/// assert_eq!(h.finalize(), zootella_ergo::sha1(b"a")); // the hash so far, which leaves the hasher going
/// h.update(b"bc");
/// assert_eq!(h.finalize(), zootella_ergo::sha1(b"abc"));
/// ```
#[derive(Clone, Debug, Default)]
pub struct Sha1(sha1::Sha1);

impl Sha1 {
    /// A new hasher, with nothing in it yet.
    pub fn new() -> Self {
        Self::default()
    }
    /// Hash the next piece.
    pub fn update(&mut self, data: &[u8]) {
        sha1::Digest::update(&mut self.0, data);
    }
    /// The hash of every piece so far, from a copy, so the hasher can take more.
    pub fn finalize(&self) -> [u8; 20] {
        sha1::Digest::finalize(self.0.clone()).into()
    }
}

/// A SHA-256 hasher, the same way, for 32 bytes.
#[derive(Clone, Debug, Default)]
pub struct Sha256(sha2::Sha256);

impl Sha256 {
    /// A new hasher, with nothing in it yet.
    pub fn new() -> Self {
        Self::default()
    }
    /// Hash the next piece.
    pub fn update(&mut self, data: &[u8]) {
        sha2::Digest::update(&mut self.0, data);
    }
    /// The hash of every piece so far, from a copy, so the hasher can take more.
    pub fn finalize(&self) -> [u8; 32] {
        sha2::Digest::finalize(self.0.clone()).into()
    }
}

/// A BLAKE3 hasher, the same way, for 32 bytes.
#[derive(Clone, Debug, Default)]
pub struct Blake3(blake3::Hasher);

impl Blake3 {
    /// A new hasher, with nothing in it yet.
    pub fn new() -> Self {
        Self::default()
    }
    /// Hash the next piece.
    pub fn update(&mut self, data: &[u8]) {
        self.0.update(data);
    }
    /// The hash of every piece so far; the blake3 crate's finalize leaves its hasher able to take more on its own.
    pub fn finalize(&self) -> [u8; 32] {
        self.0.finalize().into()
    }
}

// each hasher is an io::Write, so io::copy can feed one from a file or any other reader; hashing never fails, so neither does a write
impl std::io::Write for Sha1 {
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        self.update(data);
        Ok(data.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
impl std::io::Write for Sha256 {
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        self.update(data);
        Ok(data.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
impl std::io::Write for Blake3 {
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        self.update(data);
        Ok(data.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

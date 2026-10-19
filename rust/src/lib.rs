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

// data

// A Data holds bytes. Every language keeps bytes its own way, in arrays and buffers and views with their own rules about copying and size, and every platform call that takes or gives bytes wants its own one of them. A Data makes those choices once: it holds the bytes, and it speaks every form they travel in, text and base16, base32, base62, and base64, through methods with plain names. Make one from any form, read it out as any other, and it's the same bytes. In Rust a Data shares its bytes through an Arc, so cloning one, or clipping a view of part of it, never copies them.
//
// Every function in ergo that trades in bytes trades in Data. A hash takes a Data and answers one, and never decides what text the caller wants next; the caller says, with the next method. Only Data knows the encodings.
//
// Each form is exact, both ways. Every reader ends by writing the bytes back out and requiring the text it read, so it accepts only what its writer produces: base16 in lowercase, base32 without padding, base64 with it, nothing extra and nothing missing. Text becomes UTF-8 in Unicode Normalization Form C, so the same characters typed two ways become the same bytes, and reading bytes as text refuses bytes that aren't UTF-8, or aren't in that form, rather than guess. A reader refuses with a DataError, the way the tick readers refuse with a ParseError.
//
// clip_view looks at part of a Data without copying it. clip_copy copies the part out, into bytes of its own.
//
// The standard library has no base16, base32, or base64, so the few lines of each are here, beside base62, which is ergo's own; the shared vectors pin all four against the other editions, whose platforms have the first three.

/// Bytes, made from exactly one form, like `Data::from_text("hello")` or `Data::from_base16("68656c6c6f")`, and read out as any: array, text, base16, base32, base62, or base64.
///
/// ```
/// use zootella_ergo::Data;
/// let d = Data::from_text("hello");
/// assert_eq!(d.base16(), "68656c6c6f");
/// assert_eq!(Data::from_base62("Q6LhR6z").unwrap().text().unwrap(), "hello");
/// ```
#[derive(Clone)]
pub struct Data {
    bytes: std::sync::Arc<Vec<u8>>,
    start: usize,
    end: usize,
}

impl Data {
    /// A Data of bytes copied from the given ones; `Data::from(vec)` takes a `Vec<u8>` without copying it.
    pub fn from_array(a: &[u8]) -> Data {
        Data::from(a.to_vec())
    }
    /// A Data of text, as UTF-8 in Normalization Form C, so the same characters typed two ways become the same bytes. A `&str` is always UTF-8, so this never fails.
    pub fn from_text(s: &str) -> Data {
        use unicode_normalization::UnicodeNormalization;
        Data::from(s.nfc().collect::<String>().into_bytes())
    }
    /// A Data read from base16, or a [`DataError`] because the text isn't exactly what [`base16`](Data::base16) writes, two lowercase digits a byte.
    pub fn from_base16(s: &str) -> Result<Data, DataError> {
        base16_to_bytes(s).map(Data::from)
    }
    /// A Data read from base32, or a [`DataError`] because the text isn't exactly what [`base32`](Data::base32) writes, uppercase and without padding.
    pub fn from_base32(s: &str) -> Result<Data, DataError> {
        base32_to_bytes(s).map(Data::from)
    }
    /// A Data read from base62, or a [`DataError`] because the text isn't exactly what [`base62`](Data::base62) writes.
    pub fn from_base62(s: &str) -> Result<Data, DataError> {
        base62_to_bytes(s).map(Data::from)
    }
    /// A Data read from base64, or a [`DataError`] because the text isn't exactly what [`base64`](Data::base64) writes, with its padding.
    pub fn from_base64(s: &str) -> Result<Data, DataError> {
        base64_to_bytes(s).map(Data::from)
    }
    /// A Data of `n` bytes from the operating system's random source.
    pub fn random(n: usize) -> Data {
        let mut a = vec![0u8; n];
        getrandom::fill(&mut a).expect("the operating system's random source is unavailable");
        Data::from(a)
    }

    /// The size in bytes.
    pub fn size(&self) -> usize {
        self.end - self.start
    }
    /// The bytes themselves, which this doesn't copy.
    pub fn array(&self) -> &[u8] {
        &self.bytes[self.start..self.end]
    }
    /// The bytes as text, or a [`DataError`] because they aren't UTF-8 in Normalization Form C, since other bytes would become different bytes as text. A leading byte order mark stays the character it is.
    pub fn text(&self) -> Result<String, DataError> {
        let refused = || DataError {
            form: "text",
            text: self.base16(),
        };
        let s = std::str::from_utf8(self.array()).map_err(|_| refused())?;
        if !unicode_normalization::is_nfc(s) {
            return Err(refused());
        }
        Ok(s.to_string())
    }
    /// The bytes in base16, two lowercase digits each.
    pub fn base16(&self) -> String {
        bytes_to_base16(self.array())
    }
    /// The bytes in base32, uppercase and without padding.
    pub fn base32(&self) -> String {
        bytes_to_base32(self.array())
    }
    /// The bytes in base62.
    pub fn base62(&self) -> String {
        bytes_to_base62(self.array())
    }
    /// The bytes in base64, with its padding.
    pub fn base64(&self) -> String {
        bytes_to_base64(self.array())
    }

    /// The byte at index `i`, a number 0x00 0 through 0xff 255.
    ///
    /// # Panics
    ///
    /// Panics if there's no byte at `i`, which is a bug in the caller; ask [`size`](Data::size) first.
    pub fn get(&self, i: usize) -> u8 {
        assert!(i < self.size(), "no byte {i} in {} bytes", self.size());
        self.array()[i]
    }
    /// The bytes from index `start` up to `end`, looked at without copying, since the bytes beneath never change.
    ///
    /// # Panics
    ///
    /// Panics unless `start <= end <= size()`, which is a bug in the caller. A clip of nothing is fine.
    pub fn clip_view(&self, start: usize, end: usize) -> Data {
        check_clip(self.size(), start, end);
        Data {
            bytes: self.bytes.clone(),
            start: self.start + start,
            end: self.start + end,
        }
    }
    /// The bytes from index `start` up to `end`, copied out into bytes of their own 🪏.
    ///
    /// # Panics
    ///
    /// Panics unless `start <= end <= size()`, which is a bug in the caller.
    pub fn clip_copy(&self, start: usize, end: usize) -> Data {
        check_clip(self.size(), start, end);
        Data::from_array(&self.array()[start..end])
    }
}

/// A Data that takes the vector as it is, without copying it.
impl From<Vec<u8>> for Data {
    fn from(a: Vec<u8>) -> Data {
        let end = a.len();
        Data {
            bytes: std::sync::Arc::new(a),
            start: 0,
            end,
        }
    }
}

impl AsRef<[u8]> for Data {
    fn as_ref(&self) -> &[u8] {
        self.array()
    }
}

/// Two Data are equal when they hold the same bytes, however each was made.
impl PartialEq for Data {
    fn eq(&self, other: &Data) -> bool {
        self.array() == other.array()
    }
}
impl Eq for Data {}

impl std::fmt::Debug for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Data({})", self.base16())
    }
}

/// Text a Data reader refused, because it isn't exactly the form that reader's writer produces, like base16 with an uppercase digit, or bytes read as text that aren't UTF-8 in Normalization Form C. Carries the form and the refused text, as base16 for bytes that aren't text, so a log can show both.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataError {
    /// The form the text was meant to be, like "base16" or "text".
    pub form: &'static str,
    /// The text refused.
    pub text: String,
}

impl std::fmt::Display for DataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not {}: {:?}", self.form, self.text)
    }
}

impl std::error::Error for DataError {}

/// Start through end are byte indices from the start, so a clip of nothing is fine, but not one past the end, or backwards.
fn check_clip(size: usize, start: usize, end: usize) {
    assert!(
        start <= end && end <= size,
        "not a clip of {size} bytes: {start} to {end}"
    );
}

/// The last check in every reader: write the bytes back out, and require exactly the text it read.
fn round_trip(
    form: &'static str,
    s: &str,
    a: Vec<u8>,
    write: fn(&[u8]) -> String,
) -> Result<Vec<u8>, DataError> {
    if write(&a) == s {
        Ok(a)
    } else {
        Err(refuse(form, s))
    }
}

/// The error a reader answers with.
fn refuse(form: &'static str, s: &str) -> DataError {
    DataError {
        form,
        text: s.to_string(),
    }
}

/// The digits of base16, which the standard library leaves to us.
const BASE16_ALPHABET: &[u8; 16] = b"0123456789abcdef";

fn bytes_to_base16(a: &[u8]) -> String {
    let mut s = String::with_capacity(a.len() * 2);
    for &b in a {
        s.push(BASE16_ALPHABET[usize::from(b >> 4)] as char);
        s.push(BASE16_ALPHABET[usize::from(b & 15)] as char);
    }
    s
}
fn base16_to_bytes(s: &str) -> Result<Vec<u8>, DataError> {
    let digit = |c: u8| BASE16_ALPHABET.iter().position(|&x| x == c); // lowercase digits only
    let mut a = Vec::with_capacity(s.len() / 2);
    let mut pairs = s.as_bytes().chunks_exact(2);
    for pair in pairs.by_ref() {
        match (digit(pair[0]), digit(pair[1])) {
            (Some(high), Some(low)) => a.push((high * 16 + low) as u8),
            _ => return Err(refuse("base16", s)),
        }
    }
    if !pairs.remainder().is_empty() {
        return Err(refuse("base16", s)); // an odd number of digits
    }
    round_trip("base16", s, a, bytes_to_base16)
}

/// The base64 alphabet, which ends in + and /, with = for padding.
const BASE64_ALPHABET: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn bytes_to_base64(a: &[u8]) -> String {
    let mut s = String::with_capacity(a.len().div_ceil(3) * 4);
    for group in a.chunks(3) {
        let n = (u32::from(group[0]) << 16)
            | (u32::from(*group.get(1).unwrap_or(&0)) << 8)
            | u32::from(*group.get(2).unwrap_or(&0)); // three bytes, or fewer at the end with zeros after them
        for k in 0..4 {
            if k <= group.len() {
                s.push(BASE64_ALPHABET[((n >> (18 - 6 * k)) & 63) as usize] as char);
            } else {
                s.push('='); // padding, one for each byte the last group lacks
            }
        }
    }
    s
}
fn base64_to_bytes(s: &str) -> Result<Vec<u8>, DataError> {
    let body = s.trim_end_matches('='); // the round trip below requires exactly the padding the writer adds
    let mut a = Vec::with_capacity(s.len() / 4 * 3);
    let mut value = 0u32; // holds the accumulated bits
    let mut bits = 0u32; // how many of them
    for c in body.bytes() {
        let code = BASE64_ALPHABET
            .iter()
            .position(|&x| x == c)
            .ok_or_else(|| refuse("base64", s))?;
        value = (value << 6) | code as u32;
        bits += 6;
        if bits >= 8 {
            a.push(((value >> (bits - 8)) & 255) as u8);
            bits -= 8;
        }
    }
    round_trip("base64", s, a, bytes_to_base64)
}

// base32, for hash values in the database, so they can be short, but also always the same length

/// The base32 alphabet, uppercase letters and 2 through 7.
const BASE32_ALPHABET: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

fn bytes_to_base32(a: &[u8]) -> String {
    let mut s = String::with_capacity((a.len() * 8).div_ceil(5));
    let mut bits = 0u32; // bit counter
    let mut value = 0u32; // holds the accumulated bits
    for &b in a {
        value = (value << 8) | u32::from(b); // shift value left by 8 bits and OR with the current byte
        bits += 8;
        while bits >= 5 {
            s.push(BASE32_ALPHABET[((value >> (bits - 5)) & 31) as usize] as char); // extract the top 5 bits as a base32 character
            bits -= 5;
        }
    }
    if bits > 0 {
        s.push(BASE32_ALPHABET[((value << (5 - bits)) & 31) as usize] as char); // handle the remaining bits
    }
    s
}
fn base32_to_bytes(s: &str) -> Result<Vec<u8>, DataError> {
    let mut a = Vec::with_capacity(s.len() * 5 / 8);
    let mut bits = 0u32; // bit counter
    let mut value = 0u32; // holds the accumulated bits
    for c in s.bytes() {
        let code = BASE32_ALPHABET
            .iter()
            .position(|&x| x == c)
            .ok_or_else(|| refuse("base32", s))?;
        value = (value << 5) | code as u32;
        bits += 5;
        if bits >= 8 {
            a.push(((value >> (bits - 8)) & 255) as u8); // extract the top 8 bits as a byte
            bits -= 8;
        }
    }
    round_trip("base32", s, a, bytes_to_base32) // which refuses padding, and a last character with bits that belong to no byte
}

// base62
//
// Why so many different encoding formats? base16 is good because it's simple, standard, and you can see the bytes. base32 is shorter, double-clickable, and always the same length, good for storing hash values in the database. base62 is shorter and double-clickable, good for bytes in the query string of api calls and the location bar the user might see. base64 is about the same length, standard, but not double-clickable, included here for completeness.

/// Narrow characters in popular spots, for data.
const ALPHABET62_STREAM: &[u8] = b"1023456789ABCDEFIGHJKLMNOPQRSTUVjWXYZabcdefghkmnlopqrstuvwxyzi";
//                                 ^               ^               ^               ^            ^
//                                 1               I               j               l            i  moved characters
//                                 0               16              32              48           61  to these indices
//
// Encoding random short data, some characters appear more frequently than others.
// The last character, at index 61, is the most common.
// Also, the characters at 0, 16, 32, and 48 are slightly more common.
// Rendered to pixels for a user, we want base62 text to be as short as possible!
// So, we've adjusted the alphabet to put narrow characters in the popular spots.

/// Turn data into text using base 62, each 4 or 6 bits will become a character 0-9, a-z, and A-Z.
fn bytes_to_base62(a: &[u8]) -> String {
    let mut s = String::new(); // target string to build up and return
    let mut i = 0; // the index in bits, from 0 through all the bits in the given data
    while i < a.len() * 8 {
        // the same index as a distance in bytes followed by a distance in bits
        let byte_index = i / 8;
        let bit_index = i % 8;

        // copy the two bytes at byte_index into pair; on the very last byte, leave the right byte in pair all 0s
        let mut pair = u32::from(a[byte_index]) << 8;
        if byte_index + 1 < a.len() {
            pair |= u32::from(a[byte_index + 1]);
        }

        // read the 6 bits at i as a number, called code, which will be 0 through 63
        let code = (pair >> (10 - bit_index)) & 63;

        // describe the 6 bits with a numeral or letter, 111100 is 60, if 61 move forward 4, not 6
        if code < 61 {
            s.push(ALPHABET62_STREAM[code as usize] as char); // 000000 0 through 111100 60 append character at that index
            i += 6;
        } else {
            s.push(ALPHABET62_STREAM[61] as char); // 111101 61, 111110 62, and 111111 63 append 'i' and move past '1111'
            i += 4;
        }
    }
    s
}
fn base62_to_bytes(s: &str) -> Result<Vec<u8>, DataError> {
    let mut b = Bin::new(s.len()); // more characters become fewer bytes, so this capacity will always be plenty
    let mut hold = 0u32; // a place to hold bits from several characters until we have 8 and can write a byte
    let mut bits = 0; // the number of bits stored in the right side of hold right now
    for c in s.bytes() {
        let code = ALPHABET62_STREAM
            .iter()
            .position(|&x| x == c)
            .ok_or_else(|| refuse("base62", s))? as u32;

        // insert the bits from code into hold
        if code == 61 {
            hold = (hold << 4) | 15; // insert 1111 for common 'i' at last alphabet index 61
            bits += 4;
        } else {
            hold = (hold << 6) | code; // insert 000000 through 111100 for other characters
            bits += 6;
        }

        // if we have enough bits in hold to write a byte
        if bits >= 8 {
            b.add_byte(((hold >> (bits - 8)) & 0xff) as u8); // move the 8 leftmost bits in hold to our Bin object
            bits -= 8; // remove the bits we wrote from hold, any extra bits there will be written next time
        }
    }
    round_trip("base62", s, b.array().to_vec(), bytes_to_base62) // which refuses a last character with bits that belong to no byte
}

// bin

/// A set number of bytes, which you add a byte or a Data at a time until it's full. It's how to assemble bytes whose size you know before you start, and refusing to hold more than that catches a mistake in the prediction.
///
/// ```
/// use zootella_ergo::{Bin, Data};
/// let mut b = Bin::new(6);
/// b.add(&Data::from_text("hello"));
/// b.add_byte(b'!');
/// assert_eq!(b.data().text().unwrap(), "hello!");
/// ```
#[derive(Clone, Debug)]
pub struct Bin {
    array: Vec<u8>,
    capacity: usize,
}

impl Bin {
    /// A new Bin that can hold `capacity` bytes, holding none yet.
    pub fn new(capacity: usize) -> Bin {
        Bin {
            array: Vec::with_capacity(capacity),
            capacity,
        }
    }
    /// How many bytes it can hold.
    pub fn capacity(&self) -> usize {
        self.capacity
    }
    /// How many bytes it does hold.
    pub fn size(&self) -> usize {
        self.array.len()
    }
    /// Add all of a Data.
    ///
    /// # Panics
    ///
    /// Panics if it won't fit, which means the size predicted for this Bin was wrong.
    pub fn add(&mut self, data: &Data) {
        assert!(
            self.size() + data.size() <= self.capacity,
            "a Bin of {} bytes holding {} can't take {} more",
            self.capacity,
            self.size(),
            data.size()
        );
        self.array.extend_from_slice(data.array());
    }
    /// Add one byte.
    ///
    /// # Panics
    ///
    /// Panics if the Bin is already full.
    pub fn add_byte(&mut self, b: u8) {
        assert!(
            self.size() < self.capacity,
            "a Bin of {} bytes is full",
            self.capacity
        );
        self.array.push(b);
    }
    /// What it holds so far.
    pub fn array(&self) -> &[u8] {
        &self.array
    }
    /// What it holds so far, as a Data, which copies it, since the Bin can still take more.
    pub fn data(&self) -> Data {
        Data::from_array(&self.array)
    }
}

// hash

// Three hashes, each the one the world already knows. SHA-1 is the oldest, the one older manifests name a file by and git names its objects with. It's broken for collisions, two different inputs made to share a hash on purpose, so use it to match a hash that already exists, never to protect anything. SHA-256 is the standard everywhere today. BLAKE3 is the newest, built for speed as a tree, so pieces of one input can hash on many cores at once, and it's the one to name a file by for years to come.
//
// Each hash takes a Data and answers one, and comes in two forms that give the same answer. The function takes all the bytes at once, like sha256(&data), for data already in memory. The hasher takes them a piece at a time, as a file or a stream delivers them: make one with Sha256::new(), call update with each piece in order, and call finalize for the hash. A file of any size hashes this way in the memory of one piece, and a single read of the file can feed all three hashers. Finalize doesn't finish a hasher, as in the blake3 crate: update it again and finalize again for the hash of everything so far. Each hasher is also an io::Write, so io::copy can feed one straight from a reader, the raw bytes of each read going in with no Data made of them.
//
// The standard library has no cryptographic hashes, so SHA-1 and SHA-256 come from RustCrypto's sha1 and sha2, and BLAKE3 from the blake3 crate, the official implementation by BLAKE3's authors. Each edition takes each hash from its platform wherever the platform has it, and otherwise from the official implementation or the best known one, and the shared vectors and the fuzz test hold all four to the same answers.

/// The SHA-1 hash of a Data, as a Data of 20 bytes. For matching a hash that already exists, like an older manifest's, since SHA-1 is broken for collisions.
///
/// ```
/// use zootella_ergo::{Data, sha1};
/// assert_eq!(sha1(&Data::from_text("abc")).base16(), "a9993e364706816aba3e25717850c26c9cd0d89d");
/// ```
pub fn sha1(data: &Data) -> Data {
    Data::from(<sha1::Sha1 as sha1::Digest>::digest(data.array()).to_vec())
}

/// The SHA-256 hash of a Data, as a Data of 32 bytes.
///
/// ```
/// use zootella_ergo::{Data, sha256};
/// assert_eq!(sha256(&Data::from_text("abc")).array()[..4], [0xba, 0x78, 0x16, 0xbf]);
/// ```
pub fn sha256(data: &Data) -> Data {
    Data::from(<sha2::Sha256 as sha2::Digest>::digest(data.array()).to_vec())
}

/// The BLAKE3 hash of a Data, as a Data of 32 bytes.
///
/// ```
/// use zootella_ergo::{Data, blake3};
/// assert_eq!(blake3(&Data::from_text("abc")).array()[..4], [0x64, 0x37, 0xb3, 0xac]);
/// ```
pub fn blake3(data: &Data) -> Data {
    Data::from_array(blake3::hash(data.array()).as_bytes())
}

/// The SHA-256 hash of text, as base32, the shortcut for the common case: text to Data, hashed, and out as the form icarus keeps hash values in.
///
/// ```
/// assert_eq!(zootella_ergo::hash_text("hello"), "FTZE3OS7WCRQ4JXIHMVMLOPCTYNRMHS4D6TUEXTTAQZWFE4LTASA");
/// ```
pub fn hash_text(s: &str) -> String {
    sha256(&Data::from_text(s)).base32()
}

/// A SHA-1 hasher, a piece at a time: [`update`](Sha1::update) with each piece in order, each a Data, then [`finalize`](Sha1::finalize) for the same Data of 20 bytes [`sha1`] gives all at once.
///
/// ```
/// use zootella_ergo::{Data, Sha1, sha1};
/// let mut h = Sha1::new();
/// h.update(&Data::from_text("a"));
/// assert_eq!(h.finalize(), sha1(&Data::from_text("a"))); // the hash so far, which leaves the hasher going
/// h.update(&Data::from_text("bc"));
/// assert_eq!(h.finalize(), sha1(&Data::from_text("abc")));
/// ```
#[derive(Clone, Debug, Default)]
pub struct Sha1(sha1::Sha1);

impl Sha1 {
    /// A new hasher, with nothing in it yet.
    pub fn new() -> Self {
        Self::default()
    }
    /// Hash the next piece.
    pub fn update(&mut self, data: &Data) {
        sha1::Digest::update(&mut self.0, data.array());
    }
    /// The hash of every piece so far, from a copy, so the hasher can take more.
    pub fn finalize(&self) -> Data {
        Data::from(sha1::Digest::finalize(self.0.clone()).to_vec())
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
    pub fn update(&mut self, data: &Data) {
        sha2::Digest::update(&mut self.0, data.array());
    }
    /// The hash of every piece so far, from a copy, so the hasher can take more.
    pub fn finalize(&self) -> Data {
        Data::from(sha2::Digest::finalize(self.0.clone()).to_vec())
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
    pub fn update(&mut self, data: &Data) {
        self.0.update(data.array());
    }
    /// The hash of every piece so far; the blake3 crate's finalize leaves its hasher able to take more on its own.
    pub fn finalize(&self) -> Data {
        Data::from_array(self.0.finalize().as_bytes())
    }
}

// each hasher is also an io::Write, which takes the language's own bytes, so io::copy can feed one straight from a file or any other reader without making a Data of each read; hashing never fails, so neither does a write
impl std::io::Write for Sha1 {
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        sha1::Digest::update(&mut self.0, data);
        Ok(data.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
impl std::io::Write for Sha256 {
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        sha2::Digest::update(&mut self.0, data);
        Ok(data.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
impl std::io::Write for Blake3 {
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        self.0.update(data);
        Ok(data.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

// hash measure

// The fuji hashes are two of ergo's own, from icarus, and both are SHA-256 over a summary of a file rather than over the file itself. The piece hash covers every byte: it cuts the file into 4 MiB pieces, hashes each one, and hashes the list of those hashes. The tip hash covers almost none of them: it hashes the file's first 4 KiB, 4 KiB from its middle, and its last two, at most 16 KiB however large the file, so it's done in a few reads and finds likely copies of a file across millions of others for the cost of those few reads each. A file of 16 KiB or less is its own tip. Each summary starts with a title that names the protocol and the file's size, like "Fuji.Tips.SHA256.4KiB.5.", so two files of different sizes never share a hash, even with identical tips.
//
// The pieces are there by design, for JavaScript, whose Web Crypto hashes only all at once: hashing 4 MiB pieces and then the list of them lets a page hash a file of any size at Web Crypto's full speed. Here each piece fills a vector of its own, which moves into its Data without a copy, where JavaScript slides a conveyor belt. The tip hash can come two ways that must agree: from a file, by seeking to just the stripes, or from a stream, by noticing the stripes as they pass on the way to the piece hash.

/// The pieces of a file at a unit, and the stripes the tip hash reads, each a start and end measured from the start of the file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HashMeasurement {
    /// Total number of pieces.
    pub pieces: u64,
    /// Quantity of complete pieces.
    pub complete: u64,
    /// Byte size of the last piece if it is a fragment, or zero.
    pub fragment: u64,
    /// The stripes the tip hash reads, each a start and an end, not a start and a length.
    pub stripes: Vec<(u64, u64)>,
    /// How many bytes all the stripes hold.
    pub stripe_size: u64,
}

/// Given a file size and a unit, measure the pieces of that size, and the stripes the tip hash reads 📏.
///
/// ```
/// let m = zootella_ergo::hash_measure(17, 4);
/// assert_eq!(m.stripes, [(0, 4), (8, 17)]); // the first stripe, then the middle, next to last, and last together
/// ```
///
/// # Panics
///
/// Panics unless both the file size and the unit are 1 or more bytes, a bug in the caller.
pub fn hash_measure(file: u64, unit: u64) -> HashMeasurement {
    assert!(
        file >= 1 && unit >= 1,
        "not a file and unit to measure: {file}, {unit}"
    );
    let pieces = file.div_ceil(unit);
    let complete = file / unit;
    let fragment = file % unit;
    if file <= 4 * unit {
        // file size is 4 units or less, so just hash the whole thing
        return HashMeasurement {
            pieces,
            complete,
            fragment,
            stripes: vec![(0, file)],
            stripe_size: file,
        };
    }
    let first = 0;
    let middle = file / (2 * unit) * unit; // byte index of middle piece
    let penultimate = (pieces - 2) * unit; // second to last piece, must always be complete
    let last_size = if fragment > 0 { fragment } else { unit }; // how big the last stripe is
    let mut stripes = vec![(first, first + unit)]; // first and middle never touch
    if middle + unit == penultimate {
        stripes.push((middle, middle + unit + unit + last_size)); // but middle and penultimate can, and penultimate and last always do
    } else {
        stripes.push((middle, middle + unit)); // middle with space before and after
        stripes.push((penultimate, penultimate + unit + last_size));
    }
    HashMeasurement {
        pieces,
        complete,
        fragment,
        stripes,
        stripe_size: 3 * unit + last_size,
    }
}

/// Given a foreground stripe and a background pattern of stripes that don't touch or overlap, the ranges where they overlap, each a start and end from the start, and an empty list for none.
pub fn hash_stripe_overlap(foreground: (u64, u64), background: &[(u64, u64)]) -> Vec<(u64, u64)> {
    let (a, b) = foreground;
    background
        .iter()
        .filter(|&&(c, d)| a < d && b > c) // there's an overlap!
        .map(|&(c, d)| (a.max(c), b.min(d)))
        .collect()
}

// hash file and stream

/// One of a protocol's two units: the title its summary starts with, and the size of a piece or a stripe.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HashUnit {
    /// The title the summary starts with, before the file's size.
    pub title: String,
    /// The size of a piece, or of a stripe, in bytes.
    pub size: u64,
}

/// The units of the piece hash and the tip hash. The default is the fuji protocol, a new simple protocol that can hash huge files on both the server and the page 🗻.
///
/// 4 MiB hashes in ~10ms similar to the frequency of page animation frames, uploads in under a second over a typical cable modem, and keeps the index of hash values smaller than the piece size for files under 500 GB 🗄️. 4 KiB matches the cluster and sector size on Windows NTFS and the block size on Mac APFS, so hard drive reads can be as fast as possible 💽.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HashProtocol {
    /// The piece hash's title and piece size.
    pub pieces: HashUnit,
    /// The tip hash's title and stripe size.
    pub tips: HashUnit,
}

impl Default for HashProtocol {
    fn default() -> Self {
        HashProtocol {
            pieces: HashUnit {
                title: "Fuji.Pieces.SHA256.4MiB.".to_string(),
                size: 4 * size::MB,
            },
            tips: HashUnit {
                title: "Fuji.Tips.SHA256.4KiB.".to_string(),
                size: 4 * size::KB,
            },
        }
    }
}

/// What hash_file and hash_stream know so far, given to a progress callback as it changes and returned at the end.
#[derive(Clone, Debug)]
pub struct HashStatus {
    /// The size the file was said to be.
    pub total_size: u64,
    /// How much of it has been hashed.
    pub hashed_size: u64,
    /// The tip hash, once it's done.
    pub tip_hash: Option<Data>,
    /// The piece hash, once it's done, from hash_stream only.
    pub piece_hash: Option<Data>,
    /// A tick, milliseconds since 1970.
    pub start_time: i64,
    /// When we last changed anything here.
    pub update_time: i64,
    /// Milliseconds, once it's done.
    pub duration: Option<i64>,
}

/// Given a file to read that can seek, and its size, quickly compute just the tip hash with the fuji protocol, reading only the stripes; [`HashProtocol::hash_file`] takes another protocol.
///
/// # Panics
///
/// Panics if `size` is zero, a bug in the caller: an empty file has no fuji hash.
pub fn hash_file<F: std::io::Read + std::io::Seek>(
    file: &mut F,
    size: u64,
) -> std::io::Result<HashStatus> {
    HashProtocol::default().hash_file(file, size)
}

/// Given a stream to read and its size, compute both the tip hash and the piece hash with the fuji protocol in one pass, telling `on_progress` as each piece is hashed; [`HashProtocol::hash_stream`] takes another protocol.
///
/// # Panics
///
/// Panics if `size` is zero, a bug in the caller: an empty file has no fuji hash.
pub fn hash_stream<R: std::io::Read>(
    stream: &mut R,
    size: u64,
    on_progress: impl FnMut(&HashStatus),
) -> std::io::Result<HashStatus> {
    HashProtocol::default().hash_stream(stream, size, on_progress)
}

impl HashProtocol {
    /// [`hash_file`] with this protocol.
    pub fn hash_file<F: std::io::Read + std::io::Seek>(
        &self,
        file: &mut F,
        size: u64,
    ) -> std::io::Result<HashStatus> {
        // based on the file size, pick stripes at the start, middle, and end for us to hash quickly
        let measure = hash_measure(size, self.tips.size);
        let mut status = new_status(size);

        // for tip hashing, the summary we'll hash is the title followed by stripes of file data (hashing file data, not hashes); different sized files hash differently even with identical tips
        let title = Data::from_text(&format!("{}{size}.", self.tips.title));
        let mut tips = Bin::new(title.size() + count_usize(measure.stripe_size));
        tips.add(&title);
        for &(start, end) in &measure.stripes {
            file.seek(std::io::SeekFrom::Start(start))?;
            let mut stripe = vec![0; count_usize(end - start)];
            file.read_exact(&mut stripe)?;
            tips.add(&Data::from(stripe));
        }

        // hash the summary of the file in the bin, which must be exactly full, a check on our prediction of how much data we'd hash
        status.tip_hash = Some(sha256(&full(&tips)));
        status.hashed_size = measure.stripe_size; // we actually hashed more because of the title, but don't count this as to speed
        finish(&mut status);
        Ok(status)
    }

    /// [`hash_stream`] with this protocol.
    pub fn hash_stream<R: std::io::Read>(
        &self,
        stream: &mut R,
        size: u64,
        mut on_progress: impl FnMut(&HashStatus),
    ) -> std::io::Result<HashStatus> {
        let mut hasher = self.fuji_hasher(size);
        let mut status = new_status(size);
        let mut buffer = vec![0; 64 * size::KB as usize]; // each read from the stream lands here
        let mut fed = 0u64; // how many bytes we've given the hasher
        while fed <= size {
            // loop while boxes of data arrive from the stream; past the size, stop reading, and finalize will refuse
            let n = match stream.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => n,
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue, // a signal interrupted the read; read again
                Err(e) => return Err(e),
            };
            hasher.feed(&buffer[..n]); // a new delivery of bytes from the stream has arrived! 📦
            fed += n as u64;
            let hashed = fed - fed % self.pieces.size; // each update hashes every piece it fills, so this much is hashed
            if status.hashed_size < hashed && hashed <= size {
                status.hashed_size = hashed;
                status.update_time = now();
                on_progress(&status);
            }
        }

        // hash the summary of the file
        let hashes = hasher.finalize()?;
        status.hashed_size = size; // the fragment piece, if any, is hashed too
        status.tip_hash = Some(hashes.tip_hash);
        status.piece_hash = Some(hashes.piece_hash);
        finish(&mut status);
        on_progress(&status);
        Ok(status)
    }

    /// [`FujiHasher::new`] with this protocol.
    pub fn fuji_hasher(&self, size: u64) -> FujiHasher {
        // measurements for the piece hash, and for the tip hash, which we'll peek for as the pieces pass
        let measure_pieces = hash_measure(size, self.pieces.size);
        let measure_tips = hash_measure(size, self.tips.size);

        // for the pieces hash, the summary we'll hash is the title followed by hashes of pieces of file data, each SHA-256 hash 32 bytes (hashing hashes, not file data)
        let pieces_title = Data::from_text(&format!("{}{size}.", self.pieces.title));
        let mut pieces = Bin::new(pieces_title.size() + 32 * count_usize(measure_pieces.pieces));
        pieces.add(&pieces_title);

        // for the tips hash, the summary we'll hash is the title followed by the start, middle, and end stripes of data
        let tips_title = Data::from_text(&format!("{}{size}.", self.tips.title));
        let mut tips = Bin::new(tips_title.size() + count_usize(measure_tips.stripe_size));
        tips.add(&tips_title);

        let piece_size = count_usize(self.pieces.size);
        FujiHasher {
            size,
            piece_size,
            stripes: measure_tips.stripes,
            pieces,
            tips,
            piece: Vec::with_capacity(piece_size), // the piece we're filling, which moves into a Data without a copy once it's full
            address: 0,                            // how far into the file each box that arrives is
        }
    }
}

/// Both fuji hashes of a file, which a [`FujiHasher`] answers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FujiHashes {
    /// The tip hash, from the stripes at the file's start, middle, and end.
    pub tip_hash: Data,
    /// The piece hash, from the hashes of all of its pieces.
    pub piece_hash: Data,
}

/// A fuji hasher, a piece at a time: make one for a file of a given size, update it with each piece of the file in order, and finalize for both its tip hash and its piece hash.
///
/// It's for bytes that arrive as events, like the chunks of an upload; [`hash_stream`] reads a stream into one, and it's an io::Write, so io::copy can feed one too. update never fails, and finalize answers an error when the pieces didn't add up to the size, since that's a fact about the data that came in rather than a bug. Finalize doesn't finish it, as with [`Sha256`].
#[derive(Clone, Debug)]
pub struct FujiHasher {
    size: u64,
    piece_size: usize,
    stripes: Vec<(u64, u64)>,
    pieces: Bin,
    tips: Bin,
    piece: Vec<u8>,
    address: u64,
}

impl FujiHasher {
    /// A hasher for a file of `size` bytes with the fuji protocol; [`HashProtocol::fuji_hasher`] takes another protocol.
    ///
    /// # Panics
    ///
    /// Panics if `size` is zero, a bug in the caller: an empty file has no fuji hash.
    pub fn new(size: u64) -> Self {
        HashProtocol::default().fuji_hasher(size)
    }

    /// Hash the next piece.
    pub fn update(&mut self, data: &Data) {
        self.feed(data.array());
    }

    /// Take the next box of bytes 📦.
    fn feed(&mut self, delivery: &[u8]) {
        let n = delivery.len();
        let stripe = (self.address, self.address + n as u64); // where in the file this box of data is
        self.address += n as u64; // outside the box, move address past this box to use to mark the start of the next one
        if self.address > self.size {
            return; // more bytes than the size, which finalize will refuse; past the size, there's nothing to hash them into
        }

        // first, for the tip hash, we don't have random access through the file, but we do through this box!
        for (start, end) in hash_stripe_overlap(stripe, &self.stripes) {
            self.tips.add(&Data::from_array(
                &delivery[count_usize(start - stripe.0)..count_usize(end - stripe.0)],
            )); // subtract box address to clip in data
        }

        // second, for the piece hash, shovel bytes from the box into the piece until the box is empty 🪏
        let mut shoveled = 0; // we move this index over the bytes in the box as we process them
        while shoveled < n {
            let shovel = (n - shoveled).min(self.piece_size - self.piece.len());
            self.piece
                .extend_from_slice(&delivery[shoveled..shoveled + shovel]);
            shoveled += shovel;
            if self.piece.len() == self.piece_size {
                let full_piece =
                    std::mem::replace(&mut self.piece, Vec::with_capacity(self.piece_size));
                self.pieces.add(&sha256(&Data::from(full_piece)));
            }
        }
    }

    /// Both hashes of the pieces so far, from a copy, so the hasher can take more, or an error because they didn't add up to the size.
    pub fn finalize(&self) -> std::io::Result<FujiHashes> {
        if self.address != self.size {
            let kind = if self.address < self.size {
                std::io::ErrorKind::UnexpectedEof // the pieces ended early
            } else {
                std::io::ErrorKind::InvalidData
            };
            return Err(std::io::Error::new(
                kind,
                format!(
                    "the pieces were {} bytes, not the {} they were said to be",
                    self.address, self.size
                ),
            ));
        }

        // most files will end with a fragment piece smaller than a full piece; hash it into a copy of the summary, so the hasher is unchanged and finalize can be asked again
        let mut summary = self.pieces.clone();
        if !self.piece.is_empty() {
            summary.add(&sha256(&Data::from_array(&self.piece)));
        }

        // hash the summaries of the file in the bins, which must be exactly full, a check on our prediction of how much data we'd hash
        Ok(FujiHashes {
            tip_hash: sha256(&full(&self.tips)),
            piece_hash: sha256(&full(&summary)),
        })
    }
}

// the fuji hasher is an io::Write too, like the hashers above; a write never fails, and finalize tells of too many bytes or too few
impl std::io::Write for FujiHasher {
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        self.feed(data);
        Ok(data.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// The Data a Bin holds, which must be exactly full; a Bin that isn't means this code predicted wrong how much data it would hash, a bug here.
fn full(bin: &Bin) -> Data {
    assert_eq!(bin.size(), bin.capacity(), "a Bin isn't full");
    bin.data()
}

/// A count of bytes as a usize, which every machine this runs on holds a file's size in.
fn count_usize(n: u64) -> usize {
    usize::try_from(n).expect("a count of bytes fits in a usize")
}

/// The tick now, milliseconds since 1970.
fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as i64)
}

/// A status at the start.
fn new_status(size: u64) -> HashStatus {
    let t = now();
    HashStatus {
        total_size: size,
        hashed_size: 0,
        tip_hash: None,
        piece_hash: None,
        start_time: t,
        update_time: t,
        duration: None,
    }
}

/// A status at the end, with its duration.
fn finish(status: &mut HashStatus) {
    status.update_time = now();
    status.duration = Some(status.update_time - status.start_time);
}

//! ergo, short for ergonomic: simple helper functions, the same in JavaScript, Python, Rust, and Go.

#[cfg(feature = "driver")]
#[doc(hidden)]
pub mod driver;

// int

// An integer in ergo is one every edition holds exactly: a whole number from -(2^53 - 1) through 2^53 - 1, JavaScript's safe range. Past it, a double can't tell one integer from the next, so a number that crossed from Rust or Go into a page could quietly change. has_int asks whether a value is one, and asking never panics, so a number from outside can be asked about first; check_int is the guard that enforces the same answer.

/// The largest integer a double holds along with every integer below it, 2^53 - 1.
const INT_CEILING: i64 = (1 << 53) - 1;

/// True if `i` is an integer every edition holds exactly, a whole number from -(2^53 - 1) through 2^53 - 1. Never panics, so a number from outside can be asked about first.
///
/// ```
/// assert!(zootella_ergo::has_int(9_007_199_254_740_991));
/// assert!(!zootella_ergo::has_int(9_007_199_254_740_992));
/// ```
pub fn has_int(i: i64) -> bool {
    (-INT_CEILING..=INT_CEILING).contains(&i)
}

/// A guard: pass an integer through, or panic because the caller handed us something that isn't one.
///
/// # Panics
///
/// Panics if `i` is past 2^53 - 1 either way, which is a bug in the caller, not an ordinary answer to handle. Ask [`has_int`] first about a number from outside.
pub fn check_int(i: i64) -> i64 {
    assert!(has_int(i), "not an integer: {i}");
    i
}

// text

// A string is a type, and text is a string that holds something. has_string asks only about the type: is this a string at all, blank and whitespace included. In Rust and Go every string parameter already is one, so there it always answers true, and it's in all four editions to keep them alike. has_text asks about what's inside: a string with at least one character that isn't whitespace, so "a" and " a " are text, while "", " ", and a tab and a newline aren't.
//
// Whitespace means one thing everywhere in ergo: the 25 characters on Unicode's White_Space list, and trim takes them off both ends of a string. Each platform's own trim keeps a list of its own. JavaScript's differs from Unicode's at exactly two characters of the 1,112,064: it trims the byte order mark, U+FEFF, which Unicode doesn't call whitespace, and keeps the next line control, U+0085, which Unicode does. Python's strip also trims the four separator controls, U+001C through U+001F, and Rust's trim and Go's TrimSpace follow Unicode's list already, so here trim is Rust's own. So ergo uses Unicode's list in every edition, and trim, has_text, and deindent agree everywhere about what whitespace is.
//
// The two characters are worth knowing by name. Next line is a line break from IBM's mainframes, whose EBCDIC text ends each line with it, and Unicode gave it a place so that text could come over and go back. Nobody types it, and a browser never makes it. Its usual accident is the byte 0x85, the ellipsis in Windows-1252, read as Latin-1 instead, and browsers read anything labeled Latin-1 as Windows-1252. So it turns up only in old data, from a mainframe or from server code that decodes with a true Latin-1. The byte order mark is common. It sits invisibly at the very start of a file to say how the file is encoded, and Excel's CSV export and older Windows tools write one. The web platform drops it whenever it decodes text, in fetch, TextDecoder, and a File's text, but Rust's fs::read_to_string and String::from_utf8 keep it, and so does a Data, so that bytes and text round-trip exactly. A server that reads an uploaded spreadsheet's bytes and splits its first line finds the mark in front of the first column's name, and trim leaves it there, as Rust's own trim does. So code that reads files from people should take off a leading byte order mark on purpose.

/// True, since a `&str` is always a string: the type already says so. It's here so the four editions match; in JavaScript and Python, where a value can be anything, `hasString` checks the type at runtime.
///
/// ```
/// assert!(zootella_ergo::has_string(""));
/// ```
pub fn has_string(_s: &str) -> bool {
    true
}

/// Pass a string through. It never panics, since the type already guarantees a string; it's here so the four editions match, where in JavaScript and Python `checkString` is a guard.
pub fn check_string(s: &str) -> &str {
    s
}

// /*************************************************************************/
// /*                                                                       */
// /*                                  (`-.                                 */
// /*                                   \  `                                */
// /*      /)         ,   '--.           \    `                             */
// /*     //     , '          \/          \   `   `                         */
// /*    //    ,'              ./         /\    \>- `   ,----------.        */
// /*   ( \  ,'    .-.-._        /      ,' /\    \   . `            `.      */
// /*    \ \'     /.--. .)       ./   ,'  /  \     .      `           `.    */
// /*     \     -{/    \ .)        / /   / ,' \       `     `-----.     \   */
// /*     <\      )     ).:)       ./   /,' ,' \        `.  /\)    `.    \  */
// /*      >^,  //     /..:)       /   //--'    \         `(         )    ) */
// /*       | ,'/     /. .:)      /   (/         \          \       /    /  */
// /*       ( |(_    (...::)     (                \       .-.\     /   ,'   */
// /*       (O| /     \:.::)                      /\    ,'   \)   /  ,'     */
// /*        \|/      /`.:::)                   ,/  \  /         (  /       */
// /*                /  /`,.:)                ,'/    )/           \ \       */
// /*              ,' ,'.'  `:>-._._________,<;'    (/            (,'       */
// /*            ,'  /  |     `^-^--^--^-^-^-'                              */
// /*  .--------'   /   |                                                   */
// /* (       .----'    |   *************************************************/
// /*  \ <`.  \         |   */
// /*   \ \ `. \        |   */  Make sure s is a string that has some text,
// /*    \ \  `.`.      |   */  meaning it's not blank, and not just space
// /*     \ \   `.`.    |   */
// /*      \ \    `.`.  |   */
// /*       \ \     `.`.|   */
// /*        \ \      `.`.  */
// /*         \ \     ,^-'  */
// /*          \ \    |     */
// /*           `.`.  |     */
// /*              .`.|     */
// /*               `._>    */
// /*                       */
// /*       g o o d w i n   */
// /*                       */
// /*************************/
/// True if `s` is text: a string with at least one character that isn't whitespace, so " a " is text and " " isn't. Never panics.
///
/// ```
/// assert!(zootella_ergo::has_text(" a "));
/// assert!(!zootella_ergo::has_text(" "));
/// ```
pub fn has_text(s: &str) -> bool {
    !trim(s).is_empty()
}

/// A guard: pass the text through, or panic because the caller handed us a string that's blank or just whitespace.
///
/// # Panics
///
/// Panics if `s` isn't text, which is a bug in the caller. Ask [`has_text`] first about a string from outside.
pub fn check_text(s: &str) -> &str {
    assert!(has_text(s), "not text: {s:?}");
    s // pass the text through
}

/// Take whitespace off both ends of `s`, by Unicode's list. Rust's own `str::trim` takes off exactly that list, so this is Rust's own, under the name the four editions share.
///
/// ```
/// assert_eq!(zootella_ergo::trim("  a b  "), "a b");
/// ```
pub fn trim(s: &str) -> &str {
    s.trim()
}

// cut finds a tag in text and cuts the text around it: what's before, the tag itself, and what's after, along with whether it found the tag at all. cut_last does the same from the end, cut2 finds two tags in order and the middle between them, and cut_after_last is for text whose shape you already know, answering what comes after the last tag, and panicking when the tag isn't there.
//
// A tag is any string of at least one character, whitespace included, so a space, a tab, or a newline can be the tag to cut at. The blank string can't: it would be found at the start of everything, so a loop that cuts until nothing is left would never end.

/// What [`cut`] and [`cut_last`] answer: whether they found the tag, and the text before it, the tag itself, and the text after it, which assemble back into the text. Each part borrows from the text, so cutting copies nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cut<'a> {
    /// Whether the tag was there.
    pub found: bool,
    /// The text before the tag, or all of it when the tag wasn't found.
    pub before: &'a str,
    /// The tag, as it sat in the text, or blank.
    pub tag: &'a str,
    /// The text after the tag, or blank.
    pub after: &'a str,
}

/// What [`cut2`] answers: whether it found both tags, and the text before the first, the first tag, the middle between them, the second tag, and the text after it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cut2<'a> {
    /// Whether both tags were there, in order.
    pub found: bool,
    /// The text before the first tag, or all of it when they weren't both found.
    pub before: &'a str,
    /// The first tag, or blank.
    pub tag1: &'a str,
    /// The text between the two tags, or blank.
    pub middle: &'a str,
    /// The second tag, or blank.
    pub tag2: &'a str,
    /// The text after the second tag, or blank.
    pub after: &'a str,
}

/// Find the first tag in `s`, and cut `s` around it.
///
/// ```
/// let c = zootella_ergo::cut("key=value", "=");
/// assert_eq!((c.found, c.before, c.after), (true, "key", "value"));
/// ```
///
/// # Panics
///
/// Panics if `tag` is blank, which would be found at the start of everything. Any tag of at least one character is fine, whitespace included.
pub fn cut<'a>(s: &'a str, tag: &str) -> Cut<'a> {
    check_cut_tag(tag);
    match s.split_once(tag) {
        // rust's own cut, which answers the text on either side
        Some((before, after)) => Cut {
            found: true,
            before,
            tag: &s[before.len()..before.len() + tag.len()],
            after,
        },
        None => Cut {
            found: false,
            before: s,
            tag: "",
            after: "",
        },
    }
}

/// Find the last tag in `s`, and cut `s` around it.
///
/// # Panics
///
/// Panics if `tag` is blank.
pub fn cut_last<'a>(s: &'a str, tag: &str) -> Cut<'a> {
    check_cut_tag(tag);
    match s.rsplit_once(tag) {
        Some((before, after)) => Cut {
            found: true,
            before,
            tag: &s[before.len()..before.len() + tag.len()],
            after,
        },
        None => Cut {
            found: false,
            before: s,
            tag: "",
            after: "",
        },
    }
}

/// Find `tag1` in `s`, and then `tag2` after it, and cut `s` around both.
///
/// # Panics
///
/// Panics if either tag is blank.
pub fn cut2<'a>(s: &'a str, tag1: &str, tag2: &str) -> Cut2<'a> {
    let c1 = cut(s, tag1);
    let c2 = cut(c1.after, tag2);
    if c1.found && c2.found {
        return Cut2 {
            found: true,
            before: c1.before,
            tag1: c1.tag,
            middle: c2.before,
            tag2: c2.tag,
            after: c2.after,
        };
    }
    // both must be found
    Cut2 {
        found: false,
        before: s,
        tag1: "",
        middle: "",
        tag2: "",
        after: "",
    }
}

/// What comes after the last tag in `s`.
///
/// # Panics
///
/// Panics if `s` doesn't have the tag at all, since this is for text whose shape the caller already knows, or if `tag` is blank. Ask `cut_last(s, tag).found` first about text from outside.
pub fn cut_after_last<'a>(s: &'a str, tag: &str) -> &'a str {
    let c = cut_last(s, tag);
    assert!(c.found, "no {tag:?} in {s:?}"); // make sure s is what we expect
    c.after
}

/// A guard: panic because a tag to cut at is blank; whitespace is fine.
fn check_cut_tag(tag: &str) {
    assert!(!tag.is_empty(), "not a tag to cut at: {tag:?}");
}

/// Be able to indent multiline string literals with code so they don't mess up overall readability!
///
/// The indent of the first line comes off every line, deeper lines keep the rest of theirs, blank lines at the start and end go, and the text ends with one newline. In JavaScript it's a tag for a template literal; here it takes the text, like a raw string.
///
/// ```
/// let s = zootella_ergo::deindent(
///     "
///     A
///       B
///     C
/// ",
/// );
/// assert_eq!(s, "A\n  B\nC\n");
/// ```
pub fn deindent(s: &str) -> String {
    // temporarily normalize newlines to just \n, and split into lines there, rather than with str::lines, which takes a carriage return off a line's end but leaves one alone as part of the line
    let normal = s.replace("\r\n", "\n").replace('\r', "\n");
    // replace tabs at the start with two spaces, and take whitespace off the end
    let lines: Vec<String> = normal
        .split('\n')
        .map(|line| tabs_to_spaces(line).trim_end().to_string())
        .collect();
    // remove blank lines from the start and end; blank lines in the middle stay
    let joined = lines.join("\n");
    let lines: Vec<&str> = joined.trim_matches('\n').split('\n').collect();

    let indent = leading_spaces(lines[0]); // how many spaces indent the first line
    let mut out = String::new();
    for line in lines {
        out.push_str(&line[leading_spaces(line).min(indent)..]); // remove up to that many spaces from each line
        out.push('\n'); // reassemble lines with trailing newlines
    }
    out
}

/// Replace each tab among a line's leading spaces and tabs with two spaces.
fn tabs_to_spaces(line: &str) -> String {
    let rest = line.trim_start_matches([' ', '\t']);
    line[..line.len() - rest.len()].replace('\t', "  ") + rest
}

/// How many spaces a line starts with.
fn leading_spaces(line: &str) -> usize {
    line.len() - line.trim_start_matches(' ').len()
}

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
/// A u64 is never negative or a fraction, the counts the JavaScript and Python editions refuse, and past the integer ceiling this still says it right, so nothing here panics.
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

// say_duration says a span of time for a person to read: how long something took, or how long ago it happened, like "2 hours 5 minutes" or "about 3 days ago". The template is the sentence it writes, with a # wherever a number goes. A tag is that #, any whitespace, and the unit after it: years, months, weeks, days, hours, minutes, or seconds, or one of the short forms yrs, yr, y, mos, mo, wks, w, d, hrs, hr, h, mins, min, m, secs, sec, and s, each written in lowercase, title case, or all capitals, like days, Days, and DAYS. Seconds can also take a fraction, #.#, #.##, or #.###, for tenths, hundredths, or thousandths, and no other unit can. Each # after the point is a digit always written, so 1.5 seconds says "1.500" under #.###, the way Excel's 0.000 writes it rather than its #.###. The whitespace can be any character Unicode calls white space: a space, a tab, or the no-break space a typographer puts there to keep "5 days" on one line.
//
// The units a template names split the span between them, and a template lists them from largest to smallest. The largest takes everything above it, so a year said in days is 365, and each smaller unit takes what's left. Everything rounds down, so whatever is smaller than the smallest unit goes unsaid. A year and a month are time::YEAR and time::MONTH, the averages over the leap cycle, so say_duration agrees with the rest of ergo, and a span of 365 days is a quarter of a day short of a year.
//
// A bare m is the one word that could be two units. On its own it's minutes, the way "5m" is in a feed, but because a template runs from largest to smallest, an m right after years, or right before weeks, days, or hours, can only be months, so "#y #m #d" says "1y 2m 3d" and "#d #h #m" says "1d 2h 3m".
//
// A plural, whole or short, comes out singular for exactly 1, in the case it was written in, so "# days" says "1 day" and "2 days", "# hrs" says "1 hr", and "# DAYS" says "1 DAY"; a single letter, like d, never changes.
//
// Units that come out zero at the start drop away, along with the text after them, so a template ready to say days can count up a task half an hour old: "#d #h #m #.##s" says "32m 55.45s", never "0d 0h 32m 55.45s". From the first unit that isn't zero, everything stays. A zero between two others stays, so a day and a second is "1 day 0 hours 0 minutes 1 second" rather than "1 day 1 second". And the smallest unit a template names is the precision it asks for, so it always shows, even on an exact amount: "#m #s" says "1m 0s" for exactly a minute. If every unit is zero, the last and smallest one says so, like "0 minutes".
//
// A unit only counts right after a #, so words and numbers around the tags pass through as they are: "about # days ago" says "about 3 days ago", and the d in "and" is only a letter. Only those exact spellings count, so a mix like hOURS is the tag h and then text, the way #hiring is. Whatever isn't a tag is written as it is, with no escaping and nothing refused, so a broken template writes text that's only wrong. The span is what must be right: a number of milliseconds within the integer ceiling, and anything else panics. It's an i64, so it has no fraction of a millisecond to drop, the way the editions that take a double do. A negative span says its size with a minus sign in front of everything, like "-2 hours 5 minutes", unless what it says is zero.
//
// The standard library has no regular expressions, so a short scanner finds the tags, trying the words in the same order the other editions do, each longer word before its prefixes. Like a regular expression, it reads the template once, left to right, so a # it read as part of a fraction that makes no tag stays text rather than starting a tag of its own, and "#.##.#secs" says "#.##.3600secs" for an hour. The shared vectors hold it to the same cuts, and the fuzz test holds it to them on random templates.

/// Each unit's words, whole and short, with the size of the unit in milliseconds, which is also how the code tells units apart; each longer word comes before its prefixes, since the first that matches wins, and a template writes each in lowercase, title case, or all capitals.
const DURATION_WORDS: [(&str, i64); 24] = [
    ("years", time::YEAR),
    ("yrs", time::YEAR),
    ("yr", time::YEAR),
    ("y", time::YEAR),
    ("months", time::MONTH),
    ("mos", time::MONTH),
    ("mo", time::MONTH),
    ("weeks", time::WEEK),
    ("wks", time::WEEK),
    ("w", time::WEEK),
    ("days", time::DAY),
    ("d", time::DAY),
    ("hours", time::HOUR),
    ("hrs", time::HOUR),
    ("hr", time::HOUR),
    ("h", time::HOUR),
    ("minutes", time::MINUTE),
    ("mins", time::MINUTE),
    ("min", time::MINUTE),
    ("m", time::MINUTE), // or months, by where it sits; say_duration below decides
    ("seconds", time::SECOND),
    ("secs", time::SECOND),
    ("sec", time::SECOND),
    ("s", time::SECOND),
];

/// A tag in a duration template: its unit's size, the whitespace and word it was written with, and how many digits of a second it wants.
struct DurationTag<'a> {
    unit: i64,
    whitespace: &'a str,
    word: &'a str,
    digits: u32,
}

/// The tag a duration template has at `i`, where the text has a #, or None if a # there doesn't start one, and either way the index where the scan goes on. A tag whose fraction comes before a unit other than seconds still comes back, so the caller can keep all of it as text. With no tag, the scan goes on past the # and the fraction and whitespace it read, the way a regular expression moves past what it matched, so a # read as part of a fraction never starts a tag of its own.
fn duration_tag(template: &str, i: usize) -> (Option<DurationTag<'_>>, usize) {
    let b = template.as_bytes();
    let mut j = i + 1; // past the #
    let mut digits = 0; // a fraction for seconds: a point and one to three more #
    if b.get(j) == Some(&b'.') && b.get(j + 1) == Some(&b'#') {
        j += 1;
        while digits < 3 && b.get(j) == Some(&b'#') {
            digits += 1;
            j += 1;
        }
    }
    let whitespace_start = j;
    for c in template[j..].chars().take_while(|c| c.is_whitespace()) {
        j += c.len_utf8(); // any whitespace, which is_whitespace asks unicode about, a character at a time since most of it takes more than a byte
    }
    let rest = &b[j..];
    let Some((word, unit)) = DURATION_WORDS.iter().find(|(w, _)| spelled(rest, w)) else {
        return (None, j); // no unit's spelling starts what follows, so the # and any fraction it read are only text
    };
    let end = j + word.len();
    (
        Some(DurationTag {
            unit: *unit,
            whitespace: &template[whitespace_start..j],
            word: &template[j..end],
            digits,
        }),
        end,
    )
}

/// True if `rest` starts with `word` spelled exactly in lowercase, title case, or all capitals, like days, Days, or DAYS.
fn spelled(rest: &[u8], word: &str) -> bool {
    let w = word.as_bytes();
    if rest.len() < w.len() {
        return false;
    }
    let r = &rest[..w.len()];
    r == w // days
        || (r[0] == w[0].to_ascii_uppercase() && r[1..] == w[1..]) // Days
        || r.iter().zip(w).all(|(a, b)| *a == b.to_ascii_uppercase()) // DAYS
}

/// Say a span of milliseconds in the units a template names, like "# hours # minutes" for "2 hours 5 minutes"; one way, with no reader.
///
/// ```
/// use zootella_ergo::{say_duration, time};
/// assert_eq!(say_duration(2 * time::HOUR + 5 * time::MINUTE, "# hours # minutes"), "2 hours 5 minutes");
/// assert_eq!(say_duration(32 * time::MINUTE + 55_458, "#d #h #m #.##s"), "32m 55.45s");
/// ```
///
/// # Panics
///
/// Panics if `span` is past the integer ceiling, 2^53 - 1 either way, which is a bug in the caller. Ask [`has_int`] first about a number from outside.
pub fn say_duration(span: i64, template: &str) -> String {
    let span = check_int(span);

    // cut the template once, into its tags and the text around them
    let mut texts = Vec::new(); // the text before each tag, and then the text after the last one
    let mut tags = Vec::new();
    let mut at = 0; // where the text not yet cut begins
    let mut i = 0;
    while i < template.len() {
        if template.as_bytes()[i] != b'#' {
            i += 1; // a # is ascii, so stepping a byte at a time never lands it inside another character
            continue;
        }
        let (tag, end) = duration_tag(template, i);
        let Some(tag) = tag else {
            i = end; // past the # and any fraction it read, which stay text, so a # among them never starts a tag
            continue;
        };
        if tag.digits > 0 && tag.unit != time::SECOND {
            i = end; // a fraction only says seconds; before any other unit, the whole match stays text, so its last # can't make a tag of its own
            continue;
        }
        texts.push(&template[at..i]);
        tags.push(tag);
        at = end;
        i = end;
    }
    texts.push(&template[at..]);
    if tags.is_empty() {
        return template.to_string(); // no tags, so nothing to fill in
    }

    // a bare m is minutes, unless it sits where only months fit in a template that runs from largest to smallest: right after years, or right before weeks, days, or hours
    for i in 0..tags.len() {
        if tags[i].word != "m" && tags[i].word != "M" {
            continue;
        }
        let before = if i > 0 { tags[i - 1].unit } else { 0 }; // the unit of the tag before this one, if there is one
        let after = tags.get(i + 1).map_or(0, |t| t.unit);
        if before == time::YEAR || [time::WEEK, time::DAY, time::HOUR].contains(&after) {
            tags[i].unit = time::MONTH;
        }
    }

    // split the span among the units named, largest first, each taking all of itself that fits in what's left
    let mut units: Vec<i64> = tags.iter().map(|t| t.unit).collect();
    units.sort_unstable_by(|a, b| b.cmp(a));
    units.dedup();
    let mut counts = Vec::new();
    let mut left = span.abs(); // milliseconds not yet said; a negative span says its size, and gets its minus sign at the end
    for unit in units {
        counts.push((unit, left / unit));
        left %= unit;
    } // left now holds the remainder that rounds away, or the fraction of a second

    // write each tag, then keep everything from the first tag that isn't zero through the end
    let said: Vec<(String, bool)> = tags
        .iter()
        .map(|tag| {
            let count = counts
                .iter()
                .find(|(u, _)| *u == tag.unit)
                .map_or(0, |(_, c)| *c);
            let mut number = count.to_string();
            let mut zero = count == 0;
            if tag.digits > 0 {
                let fraction = left / 10_i64.pow(3 - tag.digits); // tenths, hundredths, or thousandths of the second, rounded down
                number += &format!(".{fraction:0width$}", width = tag.digits as usize);
                zero = zero && fraction == 0;
            }
            let mut word = tag.word;
            if number == "1" && word.len() > 1 && word.ends_with(['s', 'S']) {
                word = &word[..word.len() - 1]; // a plural, whole or short, is singular for exactly 1, like day, hr, and DAY; the single letter s is seconds, not a plural, and never changes
            }
            (format!("{number}{}{word}", tag.whitespace), zero)
        })
        .collect();
    let nonzero = said.iter().position(|(_, zero)| !zero); // zeros at the start drop away, along with the text after them
    let first = nonzero.unwrap_or(said.len() - 1); // if every unit is zero, say the last and smallest, like "0 minutes"
    let sign = if nonzero.is_some() && span < 0 {
        "-"
    } else {
        ""
    }; // a minus sign only in front of something that isn't zero
    let mut text = format!("{sign}{}", texts[0]); // text before the first tag always stays, like "about"
    for i in first..said.len() {
        text += &said[i].0; // from there to the end, every tag and the text after it, so the smallest unit always shows, and so does text after the last tag, like "ago"
        text += texts[i + 1];
    }
    text
}

// Three shortcuts sit on top of say_duration, each with its template chosen for one use, in place of a template from the caller. They come from ergo's ancestor from 2013, which said time these three ways, and here each is a few lines of ergo.
//
// say_time_remaining says how long until something finishes, or how long it has run, in the units that matter at its size: two units while the larger is a single digit, like "9m 59s" and "1h 0m", and one unit after, like "10m" and "10h". It stays in hours up to 71 before it turns to days, since "71h" is plainer than "2d 23h" for something that finishes this week. say_time_remaining_coarse says the same, with the seconds rounded down to a multiple of 5, so a countdown on a page steps rather than flickers; the last five seconds still count down one at a time.
//
// say_time_race says how long something took to the millisecond, the way a race is timed: 5'15"223 is five minutes, fifteen seconds, and 223 milliseconds, with the minutes taking everything above them, so an hour and a quarter is 75'00"000. Its seconds and milliseconds are padded, which say_duration never does, so it writes the three numbers itself.

/// Say how long remains in one or two units, like "45s", "9m 59s", "10m", "1h 0m", "10h", or "3d", for a progress bar or an estimate.
///
/// ```
/// use zootella_ergo::{say_time_remaining, time};
/// assert_eq!(say_time_remaining(9 * time::MINUTE + 59 * time::SECOND), "9m 59s");
/// assert_eq!(say_time_remaining(10 * time::MINUTE), "10m");
/// ```
///
/// # Panics
///
/// Panics if `span` is past the integer ceiling, 2^53 - 1 either way, which is a bug in the caller. Ask [`has_int`] first about a number from outside.
pub fn say_time_remaining(span: i64) -> String {
    say_time_remaining_from(span, false)
}

/// Say how long remains like [`say_time_remaining`], with the seconds rounded down to a multiple of 5, so a countdown isn't distracting.
///
/// ```
/// use zootella_ergo::{say_time_remaining_coarse, time};
/// assert_eq!(say_time_remaining_coarse(37 * time::SECOND), "35s");
/// ```
///
/// # Panics
///
/// Panics if `span` is past the integer ceiling, 2^53 - 1 either way, which is a bug in the caller. Ask [`has_int`] first about a number from outside.
pub fn say_time_remaining_coarse(span: i64) -> String {
    say_time_remaining_from(span, true)
}

/// Say a span with the template for the units that matter at its size.
fn say_time_remaining_from(span: i64, coarse: bool) -> String {
    let span = check_int(span);
    let mut size = span.abs(); // the size picks the template; a negative span gets its minus sign from say_duration
    if coarse && size > 5 * time::SECOND {
        size -= size % (5 * time::SECOND); // down to a multiple of 5 seconds, while the last five still count one at a time
    }
    let signed = if span < 0 { -size } else { size }; // the size, with the sign of the span

    if size < time::MINUTE {
        return say_duration(signed, "#s"); // "0s" to "59s"
    }
    if size < 10 * time::MINUTE {
        return say_duration(signed, "#m #s"); // "1m 0s" to "9m 59s"
    }
    if size < time::HOUR {
        return say_duration(signed, "#m"); // "10m" to "59m"
    }
    if size < 10 * time::HOUR {
        return say_duration(signed, "#h #m"); // "1h 0m" to "9h 59m"
    }
    if size < 3 * time::DAY {
        return say_duration(signed, "#h"); // "10h" to "71h"
    }
    say_duration(signed, "#d") // "3d" and up
}

/// Say how long something took to the millisecond, the way a race is timed, like 5'15"223 for five minutes, fifteen seconds, and 223 milliseconds.
///
/// ```
/// use zootella_ergo::{say_time_race, time};
/// assert_eq!(say_time_race(5 * time::MINUTE + 15 * time::SECOND + 223), "5'15\"223");
/// ```
///
/// # Panics
///
/// Panics if `span` is past the integer ceiling, 2^53 - 1 either way, which is a bug in the caller. Ask [`has_int`] first about a number from outside.
pub fn say_time_race(span: i64) -> String {
    let span = check_int(span);
    if span < 0 {
        return format!("-{}", say_time_race(-span)); // a negative span says its size, with a minus sign in front
    }
    let minutes = span / time::MINUTE; // the minutes take everything above them
    let seconds = span % time::MINUTE / time::SECOND;
    let ms = span % time::SECOND;
    format!("{minutes}'{seconds:02}\"{ms:03}") // seconds and milliseconds padded, so every time reads the same way
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

/// The months of the year, whole and in title case the way a sentence writes them, so a caller clips and then needs only `to_uppercase` or `to_lowercase` for the other two cases. In English, each short name is the first three letters, like "Aug", which the short form and [`say_tick`] both clip out.
pub const MONTH_NAMES: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

/// The days of the week, whole and in title case, with each short name the first three letters, like "Thu". Sunday is first in every edition, the way JavaScript's getUTCDay and Go's Weekday count from 0; Python's own weekday() starts with Monday, so a Python caller who wants the week the other three keep has it in ergo.
pub const DAY_NAMES: [&str; 7] = [
    "Sunday",
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
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
        // the short name in lowercase, like "aug", clipped in place
        text.extend(
            MONTH_NAMES[p.m as usize - 1][..3]
                .chars()
                .map(|c| c.to_ascii_lowercase()),
        );
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
        let short = &p1.as_bytes()[4..7];
        p.m = MONTH_NAMES.iter().position(|name| {
            name.as_bytes()[..3]
                .iter()
                .map(u8::to_ascii_lowercase)
                .eq(short.iter().copied()) // the short name in lowercase, compared in place
        })? as u32
            + 1;
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

// say_tick writes a tick for a person to read, in whatever shape a template of tags describes. "Mmm D, YYYY" says a date like "Aug 6, 2026" on a page, "YYYY-MM-DD hh24:mm:ss.ttt" stamps a line in a log, and the iso form is just "YYYY-MM-DDThh24:mm:ss.tttZ". It's the one-way companion to the two forms above, which each read back exactly. This one only writes, so a template can take any shape at all.
//
// Uppercase tags are the calendar, and lowercase tags are the clock. YYYY is the year. M and MM are the month's number, without and with a leading zero, and MMM and MMMM are its name, short and long. D and DD are the day of the month, and DDD and DDDD are the day of the week. A name comes out in the case of its tag, so MMMM is AUGUST, Mmmm is August, and mmmm is august.
//
// On the clock, h12 and h24 are the hour, each saying which clock it means, and hh12 and hh24 are the same with a leading zero. mm and ss are the minute and the second, always two digits, with no form that leaves the zero off, since five past one is 1:05 and never 1:5. t, tt, and ttt are tenths, hundredths, and thousandths of a second, as many digits as letters, rounded down so 59.999 never becomes 60. AA, aa, A, and a are PM, pm, P, and p, and it's up to the template to put them beside a twelve-hour hour.
//
// The two cases also keep the calendar and the clock apart on the page, so in "YYYY-MM-DD hh24:mm", a lowercase mm among the calendar's capitals, or a capital MM among the clock's lowercase, stands out as the mistake it is.
//
// The template is read once, left to right, taking the longest tag at each position, and whatever isn't a tag is written as it is. There's no escaping, so "YYYYY" writes the year and then a Y, like "2026Y". Each value goes straight to the output and is never read again, so a value can't make a tag: the A in AUG is only a letter. Words don't belong in a template, since a single M, D, t, A, or a is a tag, and so is a doubled mm or ss. Say the date with a small template, and put what it writes into the sentence, page, or log line around it.
//
// A tick is an i64, so it has no fraction of a millisecond to drop, the way the editions that take a double do.
//
// Some things are left out on purpose. There's no reader, since the two forms above are for reading. There's no two-digit year. There are no time zones: say_tick writes the fields of the tick it's given as they fall in UTC, and works the same for a moment anywhere on earth. Keep times in UTC, and if a person should see their own wall clock, shift the tick by their offset at the very end, just before saying it. And there's no localization: the names are English, the same everywhere, and a page that wants its reader's language can hand the tick to the browser's Intl.

/// Every tag, each longer one before its prefixes, since the first that matches at a position wins rather than the longest.
const SAY_TAGS: [&str; 30] = [
    "YYYY", "MMMM", "Mmmm", "mmmm", "MMM", "Mmm", "mmm", "MM", "M", "DDDD", "Dddd", "dddd", "DDD",
    "Ddd", "ddd", "DD", "D", "hh12", "hh24", "h12", "h24", "mm", "ss", "ttt", "tt", "t", "AA",
    "aa", "A", "a",
];

/// Write tick `t` for a person to read, in the shape a template of tags describes, like "Mmm D, YYYY" for "Aug 6, 2026"; one way, with no reader.
///
/// ```
/// use zootella_ergo::say_tick;
/// assert_eq!(say_tick(1_344_230_277_300, "Dddd, Mmmm D, YYYY"), "Monday, August 6, 2012");
/// assert_eq!(say_tick(1_344_230_277_300, "YYYY-MM-DD hh24:mm:ss.ttt"), "2012-08-06 05:17:57.300");
/// ```
///
/// # Panics
///
/// Panics if `t` isn't a tick, which is a bug in the caller: a tick is zero through the last millisecond of 9999. Ask [`has_tick`] first about a number from outside.
pub fn say_tick(t: i64, template: &str) -> String {
    let p = split_tick(t);
    let weekday = ((t / time::DAY + 4) % 7) as usize; // days since 1970, which began on a thursday, counted from sunday, 0
    let mut text = String::new();
    let mut rest = template;
    while let Some(c) = rest.chars().next() {
        match SAY_TAGS.iter().find(|tag| rest.starts_with(*tag)) {
            Some(tag) => {
                text += &say_tag(&p, weekday, tag); // one pass over the template; each value goes to the output and is never read again
                rest = &rest[tag.len()..];
            }
            None => {
                text.push(c); // not a tag, so written as it is
                rest = &rest[c.len_utf8()..];
            }
        }
    }
    text
}

/// The text for one tag, from the utc parts of a tick and its day of the week.
fn say_tag(p: &Parts, weekday: usize, tag: &str) -> String {
    let hour = p.h; // 0 through 23
    let half = if hour % 12 == 0 { 12 } else { hour % 12 }; // the hour on a twelve-hour clock, where midnight and noon are 12 rather than 0
    let ms = p.ms % 1000; // 0 through 999
    match tag {
        "YYYY" => return p.y.to_string(), // always four digits, since a tick stays within 1970 through 9999
        "MM" => return format!("{:02}", p.m),
        "M" => return p.m.to_string(),
        "DD" => return format!("{:02}", p.d),
        "D" => return p.d.to_string(),
        "hh24" => return format!("{hour:02}"),
        "h24" => return hour.to_string(),
        "hh12" => return format!("{half:02}"),
        "h12" => return half.to_string(),
        "mm" => return format!("{:02}", p.i),
        "ss" => return format!("{:02}", p.ms / 1000),
        "ttt" => return format!("{ms:03}"),
        "tt" => return format!("{:02}", ms / 10), // rounded down, like t
        "t" => return (ms / 100).to_string(),     // rounded down, so 59.99 is 59.9 and never 60.0
        _ => {}
    }

    // the rest are words: the month's name, the weekday's name, or am and pm, each in the case of its tag
    let word = match tag.as_bytes()[0].to_ascii_lowercase() {
        b'm' => MONTH_NAMES[p.m as usize - 1],
        b'd' => DAY_NAMES[weekday],
        _ => {
            if hour < 12 {
                "AM"
            } else {
                "PM"
            }
        }
    };
    // a shorter tag writes as many letters as it has: Aug for Mmm, P for A, and both letters of PM for AA
    let word = if tag.len() < 4 {
        &word[..tag.len()]
    } else {
        word
    };
    case_like(word, tag)
}

/// Write a word in the case of the tag that asked for it: AUGUST for MMMM, August for Mmmm, and august for mmmm.
fn case_like(word: &str, tag: &str) -> String {
    if tag == tag.to_uppercase() {
        return word.to_uppercase(); // every letter a capital, like MMMM, AA, and A
    }
    if tag == tag.to_lowercase() {
        return word.to_lowercase(); // none, like mmmm, aa, and a
    }
    word.to_string() // only the first, like Mmmm, which is how the names are kept
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
// A Data can hold no bytes at all: the empty input every hash has a standard answer for, the bytes of blank text, a clip of nothing, an empty Bin.
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

// base32, for hash values in a database, so they can be short, but also always the same length

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

/// The SHA-256 hash of text, as base32, the shortcut for the common case: text to Data, hashed, and out in the form a database keeps hash values in.
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

// The fuji hashes are two of ergo's own, from the original, the JavaScript ergo grew from, and both are SHA-256 over a summary of a file rather than over the file itself. The piece hash covers every byte: it cuts the file into 4 MiB pieces, hashes each one, and hashes the list of those hashes. The tip hash covers almost none of them: it hashes the file's first 4 KiB, 4 KiB from its middle, and its last two, at most 16 KiB however large the file, so it's done in a few reads and finds likely copies of a file across millions of others for the cost of those few reads each. A file of 16 KiB or less is its own tip. Each summary starts with a title that names the protocol and the file's size, like "Fuji.Tips.SHA256.4KiB.5.", so two files of different sizes never share a hash, even with identical tips.
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

// outline

// Outline: A tree structure where each outline has a name (string), a value (Data, empty for none), and contents (other outlines).
//
// An Outline is like JSON, but stripped down to just one type of value: bytes. No strings vs numbers vs booleans vs nulls — just a name, binary data, and contents. This simplicity makes it useful as a canonical container for cryptographic data, configuration, and structured binary payloads where type richness would be overhead.
//
// The closest inspiration is Bencoding (Bram Cohen, 2001), the serialization format inside BitTorrent .torrent files. Bencoding is minimal and deterministic — dictionaries must have sorted keys, so the same data always produces the same bytes, which is critical when the SHA-1 hash of a bencoded dictionary is the torrent's identity. Outline borrows this philosophy: deterministic sort for canonical comparison, so two Outlines with the same information produce identical binary output and identical hashes. Where Outline improves on Bencoding: a text form that is visible and useful in source code and documentation (Bencoding is barely legible for anything nontrivial), clear separation of name and value (Bencoding overloads byte strings for both keys and data), and a clean tree structure rather than Bencoding's four types (strings, integers, lists, dictionaries).
//
// MessagePack (Furuhashi, 2008), CBOR (RFC 7049, 2013), and Protocol Buffers (Google, 2008) solve a different problem. They are general-purpose serialization formats with rich type systems — integers, floats, strings, arrays, maps, booleans, nulls, extension types. Powerful, but more complex than needed when all your values are just bytes. Protobuf additionally requires schema files compiled ahead of time. Outline is deliberately simpler.
//
// Outline has two serialization formats:
// - Text: indented lines like «name:"value"0a» using quoted encoding with data_to_quoted(). Visible in source code as raw string literals, explains itself in markdown documentation, diffs alongside code in version control. The parser is lenient about indentation width (2 spaces, 4 spaces, tabs), accepts trailing spaces and tabs, blank lines, and # comments at the end of a line or on a line of their own, and takes contents in any order, so pasted or hand-edited text tolerates reasonable sloppiness. The writer is canonical, sorted at every level.
// - Binary: span-prefixed length encoding (name size, name, value size, value, contents size, contents) with no delimiters or escaping. Compact and unambiguous — data stays the same size on the wire, unlike JSON which inflates binary with escape sequences. Strict, too: the reader takes only what the writer writes, sorted at every level.
//
// One value type (bytes), one tree structure, deterministic sort, a text form and a binary form. It runs anywhere Rust does, with no dependencies, no schema files, and no build step.
//
// Rules for designing an outline that you're going to use in your application:
// - Names are short, lowercase, a-z and 0-9 only. Blank names are fine, duplicate names are fine.
// - Insertion order cannot matter. Sorting is deterministic and automatic, so the same information always serializes to the same bytes. If your data has ordering that matters, encode the order explicitly--for example, wrap each item in an outline named "1", "2", "3".
// - Names are labels, not data. Don't generate names from data or encode meaning in them; that's what values are for.
// - Values are bytes, as raw and granular as possible. No compression, no structured formats, no length-prefixed sub-records; if you have structure, use nested contents instead of packing it into a value.
// - No version numbers or vendor codes. An outline grows by adding new names; readers ignore names they don't recognize.
// - Outlines are short packets, not bulk storage. A typical outline is well under 8 KB as binary.
// - Outlines must be strict trees. No cycles, no shared subtrees. In Rust an outline owns its contents, so the compiler rules out both before anything runs. This matches the text form, which has no way to express sharing.

/// True if `s` is a name an outline can have: lowercase letters and numerals, or blank. Never panics, so a name from outside can be asked about first.
///
/// ```
/// assert!(zootella_ergo::has_outline_name("key1"));
/// assert!(!zootella_ergo::has_outline_name("Key1"));
/// ```
pub fn has_outline_name(s: &str) -> bool {
    s.bytes()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
}

/// A guard: pass an outline name through.
///
/// # Panics
///
/// Panics if `s` has a character outside a-z and 0-9, which is a bug in the caller; ask [`has_outline_name`] first about a name from outside.
pub fn check_outline_name(s: &str) -> &str {
    assert!(has_outline_name(s), "not an outline name: {s:?}");
    s
}

/// The limits a reader holds an outline to. The defaults, [`OutlineLimits::default`], are ceilings far past any outline's intended use, and far below what a system can bear, so reading bytes or text from a peer you don't trust is safe as it comes, and a program with outlines of its own size sets what it needs and takes the rest from the defaults.
///
/// ```
/// use zootella_ergo::{Outline, OutlineLimits, size};
/// let limits = OutlineLimits { size: 64 * size::MB as usize, ..OutlineLimits::default() };
/// assert_eq!(limits.data_to_outline(&Outline::new("x").data()).unwrap().name(), "x");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OutlineLimits {
    /// The most bytes a reader takes, of binary, or of text as UTF-8, 4 MiB by default.
    pub size: usize,
    /// The most levels of contents below the root, which alone is depth 0, 256 by default.
    pub depth: usize,
    /// The most bytes in any one name, 2 KiB by default.
    pub name_size: usize,
}

/// 4 MiB in all, 256 levels below the root, and 2 KiB for any one name.
impl Default for OutlineLimits {
    fn default() -> OutlineLimits {
        OutlineLimits {
            size: (4 * size::MB) as usize,
            depth: 256,
            name_size: (2 * size::KB) as usize,
        }
    }
}

impl OutlineLimits {
    /// Parse a text outline into an [`Outline`] within these limits, or an [`OutlineError`] because the text isn't one; [`text_to_outline`] says how it reads.
    pub fn text_to_outline(&self, s: &str) -> Result<Outline, OutlineError> {
        outline_from_text(s, self)
    }
    /// Parse bytes into an [`Outline`] within these limits, or an [`OutlineError`] because they aren't one; [`data_to_outline`] says how it reads.
    pub fn data_to_outline(&self, d: &Data) -> Result<Outline, OutlineError> {
        outline_from_data(d, self)
    }
}

/// 268,435,455, the most a span of 4 bytes can say, and so the most bytes in any one name, value, or contents.
const SPAN_CAP: usize = 0x0fff_ffff;

/// The value of an outline made or cleared without one: a Data of no bytes, the same one for all of them, made on first use, so a program that never makes an outline never makes it. A reader can give an outline an empty Data of its own, which is no value just the same.
fn no_value() -> Data {
    static EMPTY: std::sync::OnceLock<Data> = std::sync::OnceLock::new();
    EMPTY.get_or_init(|| Data::from(Vec::new())).clone()
}

/// A tree of outlines, each with a name, a value, and contents, made with [`Outline::new`] and filled with [`add`](Outline::add), [`add_name`](Outline::add_name), [`add_value`](Outline::add_value), and [`m`](Outline::m), which makes as it navigates, so a chain builds as it goes. The essay above it in the source says what it's for and how to design one.
///
/// ```
/// use zootella_ergo::{Data, Outline, text_to_outline};
/// let mut o = Outline::new("orchard");
/// o.m("basket").m("color").set_value(Data::from_text("red"));
/// assert_eq!(o.text(), "orchard:\n  basket:\n    color:\"red\"\n");
/// assert_eq!(text_to_outline(&o.text()).unwrap().data(), o.data());
/// ```
pub struct Outline {
    /// Outlines can have names, \[a-z0-9\] only, or no name at all.
    name: String,
    /// And hold one Data value, which holds no bytes when there's no value.
    value: Data,
    /// And hold contents, lower nested outlines, which this one owns.
    contents: Vec<Outline>,
}

impl Outline {
    /// An outline with a name, or none with `""`, ready for contents.
    ///
    /// # Panics
    ///
    /// Panics if `name` isn't a name, which is a bug in the caller; ask [`has_outline_name`] first.
    pub fn new(name: &str) -> Outline {
        Outline {
            name: check_outline_name(name).to_string(),
            value: no_value(),
            contents: Vec::new(),
        }
    }

    /// How many outlines this one holds in its contents.
    pub fn length(&self) -> usize {
        self.contents.len()
    }

    // getters and setters for the name and value of this outline
    //
    // o.name() returns o's current name
    // o.set_name("") sets o's name to blank, totally fine and useful in Outline
    // o.set_name("key1") sets o's name to "key1" only numerals and lowercase letters are allowed
    //
    // o.value() gets the value, a Data, which holds no bytes if there's no value here (could still be contents, though!)
    // o.set_value(Data::from_text("alice@example.com")) sets the value
    // o.set_value(Data::from(Vec::new())) clears the value, since a Data of no bytes is no value; outlines absolutely can be nameless and/or valueless
    //
    // the name [a-z0-9] requirement may seem restrictive, but prevents the case ambiguity you've likely encountered when examining a JavaScript object of HTTP headers (did the cloud provider lowercase? could there be "origin" *and* "Origin" in here?!!) and discourages overloading values into keys (if you want dots and slashes, build that into nested contents)

    /// This outline's name, blank if it has none.
    pub fn name(&self) -> &str {
        &self.name
    }
    /// Set this outline's name, where `""` is no name.
    ///
    /// # Panics
    ///
    /// Panics if `k` isn't a name, which is a bug in the caller; ask [`has_outline_name`] first.
    pub fn set_name(&mut self, k: &str) {
        self.name = check_outline_name(k).to_string();
    }
    /// This outline's value, a Data, which holds no bytes if there's no value.
    pub fn value(&self) -> &Data {
        &self.value
    }
    /// Set this outline's value, where a Data of no bytes is no value.
    pub fn set_value(&mut self, d: Data) {
        self.value = d;
    }

    /// The outline at index `i` in contents.
    ///
    /// # Panics
    ///
    /// Panics if there's no outline at `i`, which is a bug in the caller; ask [`length`](Outline::length) first.
    pub fn get(&self, i: usize) -> &Outline {
        assert!(
            i < self.contents.len(),
            "no outline {i} in {}",
            self.contents.len()
        );
        &self.contents[i]
    }
    /// The outline at index `i` in contents, to change.
    ///
    /// # Panics
    ///
    /// Panics like [`get`](Outline::get).
    pub fn get_mut(&mut self, i: usize) -> &mut Outline {
        assert!(
            i < self.contents.len(),
            "no outline {i} in {}",
            self.contents.len()
        );
        &mut self.contents[i]
    }
    /// Add an outline you've already built to this outline's contents.
    pub fn add(&mut self, c: Outline) {
        self.contents.push(c);
    }
    /// Add a new outline with this name and no value to this outline's contents.
    ///
    /// # Panics
    ///
    /// Panics if `k` isn't a name, which is a bug in the caller.
    pub fn add_name(&mut self, k: &str) {
        self.add(Outline::new(k));
    }
    /// Add a new unnamed outline with this value to this outline's contents.
    pub fn add_value(&mut self, d: Data) {
        let mut c = Outline::new("");
        c.set_value(d);
        self.add(c);
    }
    /// True if contents includes an outline with name `k`.
    ///
    /// # Panics
    ///
    /// Panics if `k` isn't a name, since lookups validate too, so a mistake is a loud error instead of a silent "not found".
    pub fn has(&self, k: &str) -> bool {
        check_outline_name(k);
        self.contents.iter().any(|c| c.name == k)
    }

    // o.n() for "navigate": find the first outline in contents with name k, panic if not found
    // o.m() for "make": like n, but creates the outline if it doesn't exist yet
    // o.list("") returns all unnamed contents
    // o.list("type1") or filter out a named sublist, all contents named "type1"

    /// Navigate to the first outline in contents with name `k`.
    ///
    /// # Panics
    ///
    /// Panics if there isn't one, or `k` isn't a name, which is a bug in the caller; ask [`has`](Outline::has) first.
    pub fn n(&self, k: &str) -> &Outline {
        check_outline_name(k);
        match self.contents.iter().find(|c| c.name == k) {
            Some(c) => c,
            None => panic!("no outline named {k:?}"),
        }
    }
    /// Navigate to the first outline in contents with name `k`, to change.
    ///
    /// # Panics
    ///
    /// Panics like [`n`](Outline::n).
    pub fn n_mut(&mut self, k: &str) -> &mut Outline {
        check_outline_name(k);
        match self.contents.iter_mut().find(|c| c.name == k) {
            Some(c) => c,
            None => panic!("no outline named {k:?}"),
        }
    }
    /// Make: like [`n_mut`](Outline::n_mut), but creates the outline if it doesn't exist yet, so a chain like `o.m("basket").m("color").set_value(d)` builds as it goes.
    ///
    /// # Panics
    ///
    /// Panics if `k` isn't a name, which is a bug in the caller.
    pub fn m(&mut self, k: &str) -> &mut Outline {
        if !self.has(k) {
            self.add_name(k);
        }
        self.n_mut(k)
    }
    /// All the contents named `k`, or with `""`, all the unnamed ones.
    ///
    /// # Panics
    ///
    /// Panics if `k` isn't a name, which is a bug in the caller.
    pub fn list(&self, k: &str) -> Vec<&Outline> {
        check_outline_name(k);
        self.contents.iter().filter(|c| c.name == k).collect()
    }
    /// Remove all outlines in contents with name `k`.
    ///
    /// # Panics
    ///
    /// Panics if `k` isn't a name, which is a bug in the caller.
    pub fn remove(&mut self, k: &str) {
        check_outline_name(k);
        self.contents.retain(|c| c.name != k);
    }
    /// Discard this outline's value and contents (but keep the name!).
    pub fn clear(&mut self) {
        self.value = no_value();
        self.contents = Vec::new();
    }

    /// Sort the contents at every level into the order the writers use, for when the order an outline holds matters, like reading it with [`get`](Outline::get); the writers sort a copy of their own, so writing never needs this first.
    pub fn sort(&mut self) {
        // sort with a stack of our own, of outlines taken out of the outline above them, each with the contents it has yet to sort and the ones it has, rather than calling ourselves for each level down
        struct Frame {
            o: Outline,
            pending: Vec<Outline>, // the outline's contents yet to sort, last first, so the first comes off the end
            done: Vec<Outline>,    // and the ones sorted all the way down, in their original order
        }
        let frame = |mut o: Outline| {
            let mut pending = std::mem::take(&mut o.contents);
            pending.reverse();
            let done = Vec::with_capacity(pending.len());
            Frame { o, pending, done }
        };
        let mut stack = vec![frame(std::mem::take(self))];
        while let Some(top) = stack.last_mut() {
            if let Some(c) = top.pending.pop() {
                stack.push(frame(c)); // go down into the next outline in contents
            } else {
                let mut f = stack.pop().expect("the frame just looked at"); // or back up, having sorted them all
                f.done.sort_by(outline_compare); // sort from the deepest levels up, since an outline comes back up only after all its contents, then sort this level
                f.o.contents = f.done;
                match stack.last_mut() {
                    Some(parent) => parent.done.push(f.o),
                    None => *self = f.o, // the root, sorted, back where it was
                }
            }
        }
    }

    /// The outline as indented lines of text, its text serialization, visible in source and documentation, sorted at every level from a copy, so this outline stays as it is.
    pub fn text(&self) -> String {
        outline_to_text(self)
    }
    /// The outline as bytes, its binary serialization, compact on the wire, sorted the same way.
    ///
    /// # Panics
    ///
    /// Panics if a name, a value, or contents would pass the span's cap of about 256 MB.
    pub fn data(&self) -> Data {
        outline_to_data(self)
    }
}

/// An outline with no name, no value, and no contents.
impl Default for Outline {
    fn default() -> Outline {
        Outline::new("")
    }
}

/// Free an outline's contents with a stack of its own, rather than letting each outline drop its contents in turn, which is a recursion that a deep enough outline would run off the end of the stack.
impl Drop for Outline {
    fn drop(&mut self) {
        let mut pending = std::mem::take(&mut self.contents);
        while let Some(mut o) = pending.pop() {
            pending.append(&mut o.contents); // take its contents before it goes, so it goes holding none
        }
    }
}

/// An outline's name, its value, and how many outlines it holds, rather than all of them, which a deep outline would make a recursion of.
impl std::fmt::Debug for Outline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Outline")
            .field("name", &self.name)
            .field("value", &self.value)
            .field("length", &self.contents.len())
            .finish()
    }
}

/// Text or bytes an outline reader refused, because they aren't an outline, with why, like a line without a colon, contents out of order, or a span written longer than it needs to be, so a log can show it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutlineError(pub String);

impl std::fmt::Display for OutlineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for OutlineError {}

/// A value in quoted encoding the text reader couldn't read, as why the text isn't an outline.
impl From<DataError> for OutlineError {
    fn from(e: DataError) -> OutlineError {
        OutlineError(e.to_string())
    }
}

// text and data

/// Express an outline as indented lines of text for code, documentation, and diffs.
fn outline_to_text(o: &Outline) -> String {
    let o = sorted_copy(o); // write a sorted copy, so the text is canonical and the outline the caller has stays as it is
    let mut s = String::new();
    let mut indents = vec![String::new()]; // the indent at each depth, two spaces per level, each made once from the one above it
    walk(&o, |step| {
        if let Step::Enter(x, depth) = step {
            if depth == indents.len() {
                let indent = format!("{}  ", indents[depth - 1]);
                indents.push(indent);
            }
            // one line for each outline as the walk reaches it, so its contents follow it at a deeper indent
            s.push_str(&indents[depth]);
            s.push_str(&x.name);
            s.push(':');
            s.push_str(&data_to_quoted(&x.value));
            s.push('\n');
        }
    });
    s
}

/// Parse a text outline into an [`Outline`], within [`OutlineLimits::default`], or an [`OutlineError`] because the text isn't one. It reads leniently, for text a person or an agent wrote by hand: any indent, as long as an outline's contents share one, blank lines, `#` comments at the end of a line or on a line of their own, and contents in any order, which it keeps.
///
/// ```
/// let o = zootella_ergo::text_to_outline("orchard:\n  tree:\"apple\"  # a comment\n").unwrap();
/// assert_eq!(o.n("tree").value().text().unwrap(), "apple");
/// ```
pub fn text_to_outline(s: &str) -> Result<Outline, OutlineError> {
    OutlineLimits::default().text_to_outline(s)
}

/// Parse a text outline within limits, for [`text_to_outline`] and [`OutlineLimits::text_to_outline`].
fn outline_from_text(s: &str, limits: &OutlineLimits) -> Result<Outline, OutlineError> {
    if s.len() > limits.size {
        return Err(OutlineError(format!(
            "a text outline of {} bytes is past the size limit of {}",
            s.len(),
            limits.size
        ))); // before reading any of it; len counts a string's bytes
    }
    // accept both \r\n and \n line endings, splitting at the line feed alone rather than at every line ending unicode knows, and skip blank lines, and lines that are only a comment, at any indent
    let lines: Vec<&str> = s
        .split('\n')
        .map(|line| line.strip_suffix('\r').unwrap_or(line))
        .filter(|line| has_text(line) && !line.trim_start_matches([' ', '\t']).starts_with('#'))
        .collect();
    if lines.is_empty() {
        return Err(OutlineError(format!("no outline in {s:?}")));
    }

    // first pass: parse each line into (o, indent), where indent is the count of leading spaces or tabs
    // each space or tab counts as one, so 2-space, 4-space, or tab indentation all work--but must be consistent within a parent's contents
    let mut parsed = Vec::with_capacity(lines.len());
    for line in lines {
        let indent = line.len() - line.trim_start_matches([' ', '\t']).len();
        let c = cut(&line[indent..], ":");
        if !c.found {
            return Err(OutlineError(format!("no colon in {line:?}"))); // every line must have a colon separating name from quoted value
        }
        if c.before.len() > limits.name_size {
            return Err(OutlineError(format!(
                "a name of {} bytes is past the name size limit of {}",
                c.before.len(),
                limits.name_size
            )));
        }

        // find where the value ends: walk forward through c.after tracking quote state, stop at the first whitespace or # outside quotes
        // a byte at a time, since each character it looks for is ascii, and in utf-8 an ascii byte is never part of another character
        let mut value_end = c.after.len();
        let mut in_quote = false;
        for (j, ch) in c.after.bytes().enumerate() {
            if ch == b'"' {
                in_quote = !in_quote;
                continue;
            }
            if !in_quote && matches!(ch, b' ' | b'\t' | b'#') {
                value_end = j;
                break;
            }
        }
        let value_part = &c.after[..value_end];
        let tail = c.after[value_end..].trim_start_matches([' ', '\t']);
        if !(tail.is_empty() || tail.starts_with('#')) {
            return Err(OutlineError(format!("more after the value in {line:?}"))); // trailing content after the value must be whitespace, or a # comment to end of line, whatever characters it holds
        }

        if !has_outline_name(c.before) {
            return Err(OutlineError(format!("not an outline name: {:?}", c.before)));
        }
        let mut o = Outline::new(c.before);
        o.set_value(quoted_to_data(value_part)?); // a blank quoted form is no bytes, which is no value
        parsed.push((o, indent));
    }

    // second pass: build the tree with a stack that tracks the current ancestry and enforces indent consistency
    // each entry is the outline, its own indent, and the indent its children will use (set when the first child appears, then required to match for further children); an entry owns its outline until the line after its last child, when it goes into the outline above it
    struct Entry {
        o: Outline,
        indent: usize,
        child_indent: Option<usize>,
    }
    let mut parsed = parsed.into_iter();
    let (root, root_indent) = parsed.next().expect("a line, since the text has one");
    if root_indent != 0 {
        return Err(OutlineError(format!(
            "the first line of an outline is indented: {s:?}"
        ))); // the root must be at zero indent--no leading whitespace
    }
    let mut stack = vec![Entry {
        o: root,
        indent: 0,
        child_indent: None,
    }];
    for (o, indent) in parsed {
        // pop entries that can't be this line's parent (at our level or deeper), each into the outline above it, now that it has all its contents
        while stack.last().expect("the root, which stays").indent >= indent {
            if stack.len() == 1 {
                return Err(OutlineError(format!("more than one root in {s:?}"))); // outdented past the root, which means there's a second root or worse
            }
            let done = stack.pop().expect("an entry above the root");
            stack.last_mut().expect("the root").o.add(done.o);
        }
        if stack.len() > limits.depth {
            return Err(OutlineError(format!(
                "an outline nested past the depth limit of {}",
                limits.depth
            ))); // this line would be as many levels down as it has outlines above it
        }
        let parent = stack.last_mut().expect("the root, which stays");
        match parent.child_indent {
            None => parent.child_indent = Some(indent), // first child establishes the indent for this parent's contents
            Some(i) if i != indent => {
                return Err(OutlineError(format!("uneven indent in {s:?}"))); // siblings must share the same indent; a mismatched indent is ambiguous
            }
            Some(_) => {}
        }
        stack.push(Entry {
            o,
            indent,
            child_indent: None,
        });
    }
    // and the rest, each into the outline above it, down to the root
    while stack.len() > 1 {
        let done = stack.pop().expect("an entry above the root");
        stack.last_mut().expect("the root").o.add(done.o);
    }
    let root = stack.pop().expect("the root").o;
    if outline_measure(&root).is_none() {
        return Err(OutlineError(format!(
            "an outline with a name, a value, or contents past the span's cap of {SPAN_CAP} bytes"
        ))); // refuse an outline the writer couldn't write, which only limits raised past the cap would let through
    }
    Ok(root)
}

/// Serialize an outline to binary data for disk or wire.
fn outline_to_data(o: &Outline) -> Data {
    let o = sorted_copy(o); // sort a copy depth-first before we walk the tree, so the output is deterministic, and the outline the caller has stays as it is

    // first pass: measure the exact size we'll need, so we can allocate one buffer and write into it
    let Some((contents_sizes, total)) = outline_measure(&o) else {
        panic!(
            "an outline with a name, a value, or contents past the span's cap of {SPAN_CAP} bytes"
        );
    };

    // second pass: write the bytes in the same order we just measured, into one buffer made the exact size
    let mut buffer = Vec::with_capacity(total);
    walk(&o, |step| {
        if let Step::Enter(x, _) = step {
            buffer.extend_from_slice(&span_bytes(x.name.len())); // name span and bytes
            buffer.extend_from_slice(x.name.as_bytes());
            buffer.extend_from_slice(&span_bytes(x.value.size())); // value span and bytes
            buffer.extend_from_slice(x.value.array());
            buffer.extend_from_slice(&span_bytes(contents_sizes[&std::ptr::from_ref(x)])); // contents span, then each contained outline, which the walk reaches next
        }
    });
    Data::from(buffer)
}

/// Measure the bytes an outline's binary form takes, and each outline's contents, or None because a name, a value, or contents is past the span's cap, which the writer can't write.
fn outline_measure(
    o: &Outline,
) -> Option<(std::collections::HashMap<*const Outline, usize>, usize)> {
    let mut fits = true;
    let mut contents_sizes: std::collections::HashMap<*const Outline, usize> =
        std::collections::HashMap::new(); // each outline's contents size, measured on the way out of it, so the second pass looks it up in O(1) instead of recomputing
    let mut measured: Vec<usize> = Vec::new(); // at each depth, the bytes measured so far of the contents of the outline at that depth
    let mut total = 0;
    walk(o, |step| match step {
        Step::Enter(_, depth) => {
            measured.truncate(depth);
            measured.push(0);
        }
        Step::Leave(x, depth) => {
            let name_bytes = x.name.len(); // len counts a string's bytes
            let value_bytes = x.value.size();
            let contents_bytes = measured[depth]; // every outline in x's contents has been measured, since the walk leaves x after them
            if name_bytes.max(value_bytes).max(contents_bytes) > SPAN_CAP {
                fits = false;
            }
            contents_sizes.insert(std::ptr::from_ref(x), contents_bytes);
            let size = span_size(name_bytes) // three (span, bytes) pairs: name, value, contents
                + name_bytes
                + span_size(value_bytes)
                + value_bytes
                + span_size(contents_bytes)
                + contents_bytes;
            if depth > 0 {
                measured[depth - 1] += size; // count x in the contents of the outline above it
            } else {
                total = size;
            }
        }
    });

    fits.then_some((contents_sizes, total))
}

/// How many bytes a span takes to say n, 1 to 4, or 5 past the cap, which only measuring an outline the writer will refuse can reach.
fn span_size(n: usize) -> usize {
    match n {
        0..0x80 => 1,
        0x80..0x4000 => 2,
        0x4000..0x20_0000 => 3,
        0x20_0000..=SPAN_CAP => 4,
        _ => 5,
    }
}

/// Parse bytes into an [`Outline`], within [`OutlineLimits::default`], or an [`OutlineError`] because they aren't exactly what [`Outline::data`] writes: contents sorted at every level, every span in its shortest form, and nothing after. It's strict, for bytes from the wire.
///
/// ```
/// use zootella_ergo::{Data, Outline, data_to_outline};
/// let mut o = Outline::new("x");
/// o.add_name("a");
/// assert_eq!(data_to_outline(&o.data()).unwrap().text(), "x:\n  a:\n");
/// assert!(data_to_outline(&Data::from_base16("000000ff").unwrap()).is_err()); // a byte after the outline
/// ```
pub fn data_to_outline(d: &Data) -> Result<Outline, OutlineError> {
    OutlineLimits::default().data_to_outline(d)
}

/// Parse bytes within limits, for [`data_to_outline`] and [`OutlineLimits::data_to_outline`].
fn outline_from_data(d: &Data, limits: &OutlineLimits) -> Result<Outline, OutlineError> {
    if d.size() > limits.size {
        return Err(OutlineError(format!(
            "an outline of {} bytes is past the size limit of {}",
            d.size(),
            limits.size
        ))); // before reading any of it, so bytes past the limit cost nothing
    }
    let mut r = Reader {
        a: d.array(),
        w: 0,
        name_size: limits.name_size,
    };

    // parse outlines inside outlines with a stack of our own, rather than calling ourselves for each level down: the outlines still being parsed, from the root down, each with where its contents end
    let mut open = vec![r.start()?];
    let root = loop {
        let end = open.last().expect("the outline being parsed").1;
        if r.w < end {
            if open.len() > limits.depth {
                return Err(OutlineError(format!(
                    "an outline nested past the depth limit of {}",
                    limits.depth
                ))); // the next outline would be this many levels down, past the limit
            }
            open.push(r.start()?); // parse contained outlines until we've consumed exactly the contents span's bytes
            continue;
        }
        if r.w != end {
            return Err(OutlineError(
                "an outline's contents ran past their size".to_string(),
            )); // overran the contents region, since a reader short of the end goes on to the next outline inside, so the input is malformed
        }

        // top is whole, its contents all parsed, so now it can be compared and added to the outline above it
        let (mut top, _) = open.pop().expect("the outline being parsed");
        top.contents.shrink_to_fit(); // hold no more room than its contents fill, now that they're all here, since a vector grows in steps and a deep outline would keep the spare room at every level
        let Some((parent, _)) = open.last_mut() else {
            break top; // top is the root
        };
        if let Some(previous) = parent.contents.last() {
            if outline_compare(previous, &top) == std::cmp::Ordering::Greater {
                return Err(OutlineError(
                    "the contents of an outline in binary are out of order".to_string(),
                )); // the binary form is the canonical one, sorted at every level, so the same information is always the same bytes, and an outline's hash is its identity
            }
        }
        parent.add(top);
    };
    if r.w != r.a.len() {
        return Err(OutlineError(format!(
            "{} bytes after the outline",
            r.a.len() - r.w
        ))); // trailing bytes after the root outline mean malformed input
    }
    Ok(root)
}

/// The bytes the binary reader reads, the cursor that moves forward through them as it reads, and the most bytes a name can take.
struct Reader<'a> {
    a: &'a [u8],
    w: usize,
    name_size: usize,
}

impl<'a> Reader<'a> {
    /// Read a size prefix, advancing the cursor.
    fn span(&mut self) -> Result<usize, OutlineError> {
        let s = read_span(self.a, self.w)?;
        self.w += s.bytes_read;
        Ok(s.value)
    }
    /// Read n bytes, advancing the cursor.
    fn read(&mut self, n: usize) -> Result<&'a [u8], OutlineError> {
        if self.w + n > self.a.len() {
            return Err(OutlineError(format!(
                "an outline needs {n} bytes at {} of {}",
                self.w,
                self.a.len()
            )));
        }
        let b = &self.a[self.w..self.w + n];
        self.w += n;
        Ok(b)
    }
    /// Start parsing one outline: name span + bytes, value span + bytes, then the contents span, which says where its contents end.
    fn start(&mut self) -> Result<(Outline, usize), OutlineError> {
        let mut o = Outline::new("");

        let name_bytes = self.span()?;
        if name_bytes > self.name_size {
            return Err(OutlineError(format!(
                "a name of {name_bytes} bytes is past the name size limit of {}",
                self.name_size
            ))); // before reading the name
        }
        if name_bytes > 0 {
            let b = self.read(name_bytes)?;
            let name = Data::from_array(b).text()?; // a name's bytes are its text, which must be a-z and 0-9
            if !has_outline_name(&name) {
                return Err(OutlineError(format!("not an outline name: {name:?}")));
            }
            o.set_name(&name);
        }

        let value_size = self.span()?;
        if value_size > 0 {
            o.set_value(Data::from_array(self.read(value_size)?)); // zero size stays as the default, no value
        }

        let contents_size = self.span()?;
        Ok((o, self.w + contents_size))
    }
}

// Every walk through an outline keeps a stack of its own, rather than calling itself for each level down. The writers walk with walk, which calls its step with each outline as it enters it, and again as it leaves once all of that outline's contents are done, so what a recursive function would do before calling itself goes in an enter, and what it would do after, in a leave. Sort keeps a stack of outlines it takes out of their parents to sort and put back, which is how Rust changes a tree it owns without holding two references into it at once. Compare keeps a stack of the pairs it has yet to compare, the binary reader a stack of the outlines it's still reading, each with where its contents end, and freeing an outline a stack of the contents it has yet to free. The text reader has kept a stack of the outlines above the line it's reading since it was first written.
//
// The other walks were written recursively at first, which reads more simply, and moved to loops when depth came to matter. A walk that calls itself spends a frame of the language's own stack on every level, and that stack is small: a few thousand levels in JavaScript and in Rust, about a thousand in Python. An outline's binary form arrives from the wire, so a few kilobytes of nesting from anyone could run it out. Past that point, JavaScript and Python throw, Go, whose stacks grow to a gigabyte before it gives up, stops with a fatal error, and Rust aborts the whole process, which nothing can catch. In Rust, freeing a tree is a walk too, one the compiler writes as a recursion unless an outline frees its contents itself, so the Drop above does. A walk with a stack of its own needs only memory, so how deep an outline goes is limited by memory alone, how large its bytes can be by the span's cap, and no nesting from anywhere can crash it. In JavaScript, timed against the recursion they replaced, the loops are no slower.

/// A step of a walk through an outline: entering an outline, with how many levels down it is, or leaving it, once all its contents are done.
enum Step<'a> {
    Enter(&'a Outline, usize),
    Leave(&'a Outline, usize),
}

/// Walk an outline depth first, calling `step` with each outline as the walk enters it, and again as it leaves.
fn walk<'a>(root: &'a Outline, mut step: impl FnMut(Step<'a>)) {
    let mut path = vec![(root, 0)]; // the outlines from the root down to where the walk is, each with the index of the next outline to visit in its contents
    step(Step::Enter(root, 0));
    while let Some(top) = path.last_mut() {
        let o = top.0;
        if top.1 < o.contents.len() {
            let c = &o.contents[top.1]; // go down into the next outline in contents
            top.1 += 1;
            path.push((c, 0));
            step(Step::Enter(c, path.len() - 1));
        } else {
            path.pop(); // or back up, having visited them all
            step(Step::Leave(o, path.len()));
        }
    }
}

/// A copy of an outline with its contents sorted at every level, deepest first, for the writers; the values are shared rather than copied, since a Data's bytes are the same either way.
fn sorted_copy(o: &Outline) -> Outline {
    let mut copies: Vec<Outline> = Vec::new(); // the copies the walk is making, from the root down to where it is
    let mut root = None;
    walk(o, |step| match step {
        Step::Enter(x, _) => {
            let mut c = Outline::new(&x.name);
            c.set_value(x.value.clone());
            copies.push(c);
        }
        Step::Leave(_, _) => {
            let mut c = copies
                .pop()
                .expect("the copy of the outline the walk is leaving");
            c.contents.sort_by(outline_compare); // sort from the deepest levels up, since the walk leaves an outline only after all its contents, then sort this level
            match copies.last_mut() {
                Some(parent) => parent.add(c), // and add this copy, now whole, to the copy above it
                None => root = Some(c),
            }
        }
    });
    root.expect("the root, which the walk leaves last")
}

/// Compare two outlines for sort order: name first, then value, then nested contents pairwise down the whole tree; pairwise gives the canonical order only when the levels below are already in it, so every caller sorts or checks them first.
fn outline_compare(o1: &Outline, o2: &Outline) -> std::cmp::Ordering {
    use std::cmp::Ordering::Equal;
    let c = outline_compare_here(o1, o2);
    if c != Equal {
        return c; // names or values differ, which settles nearly every comparison a sort makes, with nothing made to do it
    }

    // value tied, so we examine contents pairwise, with a stack of our own of the pairs still to compare, rather than calling ourselves for each level down
    // the pair on top is next; a pair marked counts compares how many contents its two outlines hold, once all the pairs of their contents have tied
    let mut pending = Vec::new();
    /// Push a pair whose names and values tied, then the pairs of their contents above it, the first pair on top, so it and everything inside it is compared before the second.
    fn push<'a>(
        pending: &mut Vec<(&'a Outline, &'a Outline, bool)>,
        a: &'a Outline,
        b: &'a Outline,
    ) {
        pending.push((a, b, true));
        for (x, y) in a.contents.iter().zip(&b.contents).rev() {
            pending.push((x, y, false)); // compare contents pairwise at each index
        }
    }
    push(&mut pending, o1, o2);
    while let Some((a, b, counts)) = pending.pop() {
        if counts {
            let c = a.contents.len().cmp(&b.contents.len());
            if c != Equal {
                return c; // all shared contents match, fewer contents sorts first
            }
        } else {
            let c = outline_compare_here(a, b);
            if c != Equal {
                return c; // every caller sorts or checks deeper levels first, so contents at each level are already in order
            }
            push(&mut pending, a, b);
        }
    }
    Equal // a complete and deep tie: identical name, value, and contents
}

/// Compare two outlines by their names and values alone, for compare as it goes down the tree.
fn outline_compare_here(o1: &Outline, o2: &Outline) -> std::cmp::Ordering {
    // name first, where the lighter name wins; then, with the same name, value, where no value is an empty Data, which sorts before any value, since a shorter value sorts first
    // slices compare the way an outline sorts: byte by byte up to the shorter length, where the first differing byte decides the order, and then the shorter value first
    o1.name
        .cmp(&o2.name)
        .then_with(|| o1.value.array().cmp(o2.value.array()))
}

// span

// Span: a variable-length integer encoding for size prefixes in the binary format. Each byte contributes 7 bits of value; the high bit is 1 to continue, 0 to stop. Least-significant byte first (little-endian), matching LEB128 (WebAssembly, DWARF debug info) and Protocol Buffers' varint. Capped at 4 bytes (28 bits), giving a maximum value of 268,435,455 — about 256 MB. Outlines are designed to be small packets, not bulk storage, so this ceiling is generous. An Outline that approaches this size is almost certainly a misuse.
//
// An outline's name, its value, and its contents each come after a span of their size, the root's included, so the cap holds each of them to about 256 MB: the writer refuses an outline with a name, a value, or contents past it, and no span can tell the reader to expect more, though the root's three together can come to about three times that. That cap is the format's own, and it's more than a program should take from a peer it doesn't trust. An outline takes as few as 3 bytes in its binary form and about 72 in memory, so reading bytes can take more than 20 times their size, and when Rust can't get the memory it asks for, it aborts the process, which nothing can catch. So both readers hold what they read to limits, the defaults in OutlineLimits::default(): 4 MiB in all, 256 levels below the root, and 2 KiB for any one name. They're ceilings far past any outline's intended use, and far below what a system can bear, so reading from a peer you don't trust is safe as it comes, and a program whose outlines run larger sets limits of its own and reads with them, like OutlineLimits { size, ..OutlineLimits::default() }.data_to_outline(&d). Each limit refuses as early as it can: the size before reading anything, a name at its span before reading it, and depth the moment the reader goes one level past it. Whatever the limits, a reader still refuses an outline its writer couldn't write.

/// What [`span_decode`] reads: the size the span says, and how many bytes the span took, so a reader can move past it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    /// The size the span says.
    pub value: usize,
    /// How many bytes the span took, 1 to 4.
    pub bytes_read: usize,
}

/// A size as a span, a Data of 1 to 4 bytes, the way an outline's binary form writes each size.
///
/// ```
/// assert_eq!(zootella_ergo::span_encode(300).base16(), "ac02");
/// ```
///
/// # Panics
///
/// Panics unless `n` is at most 268,435,455, which is a bug in the caller.
pub fn span_encode(n: usize) -> Data {
    Data::from(span_bytes(n))
}

/// Read the span at an offset into a Data, answering its value and how many bytes it took, so a reader can move past it, or an [`OutlineError`] because the bytes there aren't a span written the one way a span is.
pub fn span_decode(d: &Data, offset: usize) -> Result<Span, OutlineError> {
    read_span(d.array(), offset)
}

/// A size as a span's bytes, at most 4.
fn span_bytes(n: usize) -> Vec<u8> {
    assert!(n < 0x1000_0000, "not a size a span can hold: {n}"); // max 268,435,455 (~256 MB)
    let mut n = n;
    let mut b = Vec::with_capacity(4);
    loop {
        let mut y = (n & 0x7f) as u8; // take the lowest 7 bits
        n >>= 7; // shift them off, ready to take the next 7
        if n > 0 {
            y |= 0x80; // set the high bit to signal "more bytes follow" to the decoder
        }
        b.push(y);
        if n == 0 {
            return b;
        }
    }
}

/// The span's value and how many bytes it took, so the caller can advance its cursor.
fn read_span(a: &[u8], offset: usize) -> Result<Span, OutlineError> {
    let mut n = 0;
    let mut shift = 0;
    let mut i = offset;
    for count in 0..4 {
        // at most 4 bytes; more than that means malformed input
        let Some(&y) = a.get(i) else {
            return Err(OutlineError(format!("a span runs past the end at {i}")));
        };
        i += 1;
        n |= usize::from(y & 0x7f) << shift; // accumulate the 7 value bits at their proper place
        if y & 0x80 == 0 {
            // high bit clear means this is the last byte
            if count > 0 && y == 0 {
                return Err(OutlineError(format!(
                    "a span written longer than it needs to be at {offset}"
                ))); // reject non-canonical encoding: a multi-byte span whose top byte contributes no bits could have fit in fewer bytes
            }
            return Ok(Span {
                value: n,
                bytes_read: i - offset,
            });
        }
        shift += 7;
    }
    Err(OutlineError(format!(
        "a span longer than 4 bytes at {offset}"
    ))) // ran past the 4 byte limit without finding a terminating byte
}

// quoted encoding

// "Quoted" Encoding: A text format for binary data that mixes quoted ASCII text with base16
//
// «"Hello"0a» is easier to read than base16 '48656c6c6f0a'
// «"A"00» shows the bits of a null terminator in a language-agnostic way, unlike 'A\0'
// «» blank text is valid quoted encoding to express nothing, size zero bytes
//
// unlike the base encodings of Data, quoted is *not* one to one! bytes come back from their quoted text, but many texts read to the same bytes
// a heuristic below balances brevity and readability when choosing which ASCII bytes to expose in quotes

/// True if the given character c is understandable to read in quotes: space through tilde, but not the double quotation mark itself, which is our delimiter!
fn quotable(c: u8) -> bool {
    (0x20..=0x7e).contains(&c) && c != 0x22
}

/// Encode the given Data as text in quoted encoding, where no bytes at all are blank text.
///
/// ```
/// use zootella_ergo::{Data, data_to_quoted};
/// assert_eq!(data_to_quoted(&Data::from_text("Hello\r\n")), "\"Hello\"0d0a");
/// ```
pub fn data_to_quoted(d: &Data) -> String {
    let a = d.array();

    let text_count = a.iter().filter(|&&c| quotable(c)).count();
    if text_count * 2 < a.len() {
        return d.base16(); // under half quotable? it's random or binary, just use base16; ties go to quoted
    }

    let mut s = String::new();
    let mut i = 0;
    while i < a.len() {
        // alternate between:
        let start = i;
        if quotable(a[i]) {
            // runs of quotable text, and
            while i < a.len() && quotable(a[i]) {
                i += 1;
            }
            s.push('"');
            s.extend(a[start..i].iter().map(|&c| char::from(c)));
            s.push('"');
        } else {
            // runs of unquotable bytes in base16
            while i < a.len() && !quotable(a[i]) {
                i += 1;
            }
            s.push_str(&bytes_to_base16(&a[start..i]));
        }
    }
    s
}

/// Convert quoted text back to Data, where blank text is valid quoted encoding to describe zero bytes of nothing, or a [`DataError`] because the text isn't quoted encoding. It reads leniently, so many texts read to the same bytes.
pub fn quoted_to_data(s: &str) -> Result<Data, DataError> {
    quoted_bytes(s)
        .map(Data::from)
        .ok_or_else(|| refuse("quoted encoding", s))
}

/// The bytes quoted text holds, or None when it isn't quoted encoding.
fn quoted_bytes(s: &str) -> Option<Vec<u8>> {
    // walk through the string, alternating between quoted sections and base16 pairs, a byte at a time, since anything that isn't ascii is refused either way
    let a = s.as_bytes();
    let mut b = Vec::new();
    let mut i = 0;
    while i < a.len() {
        if a[i] == b'"' {
            // opening quote starts a text section
            i += 1;
            let end = a[i..].iter().position(|&c| c == b'"')?; // no closing quote
            for &c in &a[i..i + end] {
                // strict: every character inside must be something data_to_quoted would have quoted
                if !quotable(c) {
                    return None; // rejects control chars, high bytes, unicode, and the quote char itself
                }
                b.push(c); // quotable guarantees ASCII, one byte per char
            }
            i += end + 1;
        } else {
            // outside quotes, consume base16 digit pairs
            if i + 1 >= a.len() {
                return None; // a lone base16 digit with no pair is invalid
            }
            let digit = |c: u8| match c {
                b'0'..=b'9' => Some(c - b'0'),
                b'a'..=b'f' => Some(c - b'a' + 10),
                _ => None, // lowercase base16 only
            };
            b.push((digit(a[i])? << 4) | digit(a[i + 1])?);
            i += 2;
        }
    }
    Some(b)
}

# date history

How software has turned a moment into text a person can read, from early Unix to Temporal, and what went wrong at each step. Written in October 2026 as research before ergo designs any display format for dates and times. It may grow into a longer historical article of its own, apart from ergo's design or introducing it. Where a fact was checked against a source, the source is listed at the end. The rest comes from long familiarity with these systems and should be checked again before anything here is published.

## A field of apologies

There may be no other area of programming where implementations are made so readily and condemned so openly, often by their own authors:

- **C** standardized `asctime` and `ctime` in 1989 and deprecated both in C23.
- **Java** shipped `java.util.Date` in 1996 and deprecated most of it one release later. It then replaced its replacement in 2014.
- **JavaScript** copied Java's Date at its birth, "bugs and all," in the words of its own history. It spent thirty years getting Temporal through its standards body.
- **Python** added `datetime.utcnow()` in 2003 and deprecated it in 2023.
- **PHP** deprecated its own `strftime` and added a second DateTime class because the first one could change under its callers.
- **Ruby's** documentation calls its own DateTime class deprecated.
- **Moment.js**, once among the most depended-on packages on npm, put a notice on its own site: "We now generally consider Moment to be a legacy project in maintenance mode. It is not dead, but it is indeed done."
- **Go's** contributors have declined to add a second formatting syntax "every time" it was asked. They then added named constants for the layouts people kept typing out by hand.
- **Rust's** `time` crate has a function named `set_soundness`, whose documentation explains that it permits undefined behavior and that "use of this function is heavily discouraged."

Clearly there's a need, since every language, platform, and framework keeps trying. And clearly it's harder than it looks, since every attempt ends with a mea culpa. This document follows those attempts in roughly the order they happened. For each it records when it came, in what language, what it was built for, how it met its market, and what went wrong. The failures are the most instructive part. Each one shows what the designers assumed, and the next design shows what they learned.

## Five problems inside one call

"Show this date" looks like string concatenation. It's actually five problems, and most of the designs below are an answer to one or two of them. Their footguns are where they ran into the others.

1. **Which day.** A count of seconds or milliseconds since an epoch has to become a year, a month, and a day. That means leap years, the Gregorian calendar, and the choice of where months start counting.
2. **Whose clock.** The same instant is 3 PM in one city and midnight in another. A fixed offset like −07:00 is arithmetic. A named zone like America/Los_Angeles is a database of rules that governments change several times a year.
3. **Which fields, how wide.** 7 or 07 or Jun or June. A 12-hour clock with AM and PM, or 24 hours. Seconds or not, milliseconds or not.
4. **Whose language and order.** June or juin, 6/7 or 7/6, a comma or a period, 3 PM or 15 h. This is the locale, and the data for it changes too.
5. **Exact or friendly.** A form a program can read back exactly, a form a person reads once, or "3 minutes ago."

## A: early computing

### Unix time and asctime, the 1970s

The first edition of the Unix manual, in 1971, defined the system time as sixtieths of a second since the start of 1971. A 32-bit counter of sixtieths overflows in a little over two years, so within a few years Unix counted whole seconds since the start of 1970. That choice made 1970 the epoch of nearly every system that followed. It also set a deadline: a signed 32-bit count of seconds runs out on January 19, 2038.

To show that count to a person, Unix had `ctime` and `asctime`, which write one fixed string: `Thu Jan  1 00:00:00 1970`, followed by a newline. There were no options. It was built for log lines, `ls`, and the `date` command, and it met its market completely; the order of its fields is still the default output of `date` today. Its footguns are the ones of its era:

- **A static buffer.** It returned a pointer into one shared buffer, so it isn't safe across threads; POSIX later added `asctime_r` and `ctime_r`.
- **A trailing newline** that every caller strips.
- **No room for big years.** The C standard's own sample code writes into a 26-character buffer and overflows for a year past 9999.

C23 marked both functions deprecated, after more than thirty years in the standard.

Beneath them sat `struct tm`, the broken-down time that C standardized in 1989. It counts `tm_year` from 1900 and `tm_mon` from 0 through 11, while `tm_mday` runs from 1. On January 1, 2000, web pages around the world showed the year as "19100." Their code had written "19" followed by `tm_year`, which had just become 100. The zero-based month went further. Java copied it, and JavaScript copied Java, so `new Date(2026, 9, 20)` in a browser today is October 20.

### Email and HTTP dates, 1977 onward

ARPANET mail needed a date in its headers that both people and programs could read. RFC 733 (1977) and RFC 822 (1982) defined one: `Tue, 06 Aug 12 05:17:57 GMT`, with a two-digit year and zone names like EST and EDT. RFC 822 also included the military's single-letter zones, A through Z. RFC 1123 (1989) moved to four-digit years. It also noted that RFC 822 had given the military zones the wrong sign, counting the wrong way from UT, so those zones "carry no information." Later revisions, RFC 2822 (2001) and RFC 5322 (2008), say a reader must still accept the obsolete forms and a writer must never produce them.

HTTP took the same date for its headers, and its leniency shows how such things compound. HTTP/1.1 told a server to send one format and accept three:

- **RFC 1123:** `Sun, 06 Nov 1994 08:49:37 GMT`
- **RFC 850:** `Sunday, 06-Nov-94 08:49:37 GMT`, with a two-digit year
- **asctime:** `Sun Nov  6 08:49:37 1994`

RFC 9110 (2022) still carries all three. Cookies went further: RFC 6265 (2011) specifies a deliberately forgiving algorithm for reading a cookie's expiry date. It picks tokens out of whatever text it finds, and it maps a two-digit year from 70 to 99 into the 1900s and from 0 to 69 into the 2000s. That's what the web's cookies actually contained. Once a reader is lenient, every writer comes to rely on it, and the leniency can never be taken back.

### strftime, 1977 to 1989

This is the design the field calls standard. Its format codes trace back at least to the `date` command of PWB/UNIX 1.0 in 1977. System V had `cftime` and `ascftime`. ANSI C standardized `strftime` in 1989, in the same standard that introduced `setlocale` and `locale.h`, and POSIX adopted it.

It was built for a C programmer writing a log line or a report: one call, a fixed buffer, and the locale set once for the whole process. The design is a percent sign and a letter, one field each:

- **Numbers:** `%Y` the year, `%m` the month, `%d` the day, `%H` the hour, `%M` the minute, `%S` the second.
- **Names:** `%b` and `%B` for the month, `%a` and `%A` for the weekday.
- **The 12-hour clock:** `%I` for the hour, with `%p` for AM and PM.

Three codes hand the whole arrangement to the locale: `%c` for the date and time, `%x` for the date, and `%X` for the time. Display chosen by the locale, which later designs would make the center of everything, was in strftime from the start. C99 added `%F` for the ISO date, `%z` for the offset, and `%G` and `%V` for ISO weeks.

It met its market completely. The same codes run in:

- **The languages:** Perl's POSIX module, Python, Ruby, Tcl, and PHP's `strftime`.
- **The tools:** the shell's `date`, gawk, and SQLite's `strftime()`.
- **Rust:** two of its three main date crates.

It's the most widely half-remembered date syntax there is. Its footguns come from that same compact design:

- **Width isn't in the standard.** `%d` always writes 07. For 7 there's `%-d` in glibc and the BSDs, `%#d` on Windows, and nothing in standard C. Python hands most codes to the platform's C library, so a script with `%-d` runs on Linux and macOS and raises a ValueError on Windows.
- **The locale is global.** `setlocale` changes the month names for every thread in the process, so a library can't write English without changing everyone's locale. POSIX added a per-thread `uselocale` in 2008.
- **The C locale's `%x` and `%c` use a two-digit year,** `10/20/26`. GCC has a warning, `-Wformat-y2k`, for exactly this.
- **The letters collide.** `%m` is the month and `%M` the minute. `%d` is the day and `%D` is the whole American date, `%m/%d/%y`. `%y`, `%Y`, and `%G` are three different years, the last being the year an ISO week belongs to.
- **No sub-seconds.** `struct tm` has no field for them, so every descendant invented its own: `%f` for microseconds in Python, `%L` and `%N` in Ruby, `%N` in GNU date, `%.3f` in chrono.
- **The reader is weaker.** `strptime`, the inverse, came later and varies by platform, especially with zone names.

What came next, within the lineage: GNU added flags (`-`, `_`, `0`, `^`, `#`) and field widths. Ruby wrote its own strftime so those flags behave the same everywhere. PHP deprecated its strftime in 8.1, pointing users to ICU.

### Spreadsheets and databases

Two other code systems grew up beside strftime, each for its own market, and each shows a lesson more clearly than strftime does.

**Spreadsheets.** Lotus 1-2-3 stored a date as a serial number of days and treated 1900 as a leap year, which it wasn't. Excel copied the bug for compatibility, so February 29, 1900 exists in Excel to this day. The Mac edition long counted from 1904 instead, so the same serial number meant different days on the two platforms. Excel's format codes repeat letters: `m`, `mm`, `mmm`, `mmmm`, and `mmmmm` for the month's first letter. The same `m` means minutes when it sits after an hour or before a second.

Excel's reader is so eager that a cell holding SEPT2 or MARCH1 becomes a date. A 2016 study found such errors in the supplementary files of about a fifth of genomics papers that included gene lists. In 2020 the HUGO Gene Nomenclature Committee renamed 27 human genes, SEPT1 to SEPTIN1 and MARCH1 to MARCHF1 among them, partly so spreadsheets would stop converting them. A lenient date reader got human genes renamed.

**Databases.** Oracle's `TO_CHAR` (1980s) spells fields as words: `YYYY-MM-DD HH24:MI:SS`, with `MI` for minutes so it can't be mistaken for months. It added `FM` to drop padding and, for the year 2000, `RR`, a two-digit year read through a sliding century window. PostgreSQL copied it. MySQL's `DATE_FORMAT` uses percent codes like strftime but with different letters: `%i` is the minute, `%M` is the month's name, and `%D` is the day with an English suffix, "20th."

**Microsoft Exchange, January 1, 2022.** A version number shaped like a date, `yymmddHHMM`, was kept in a signed 32-bit integer. The first update of 2022, 2201010001, exceeded 2,147,483,647, and mail stopped moving on servers around the world.

### ISO 8601, 1988, and RFC 3339, 2002

ISO 8601 came out in 1988, consolidating several earlier ISO standards from the 1970s. It was built for unambiguous interchange across languages and borders: numeric, ordered from the year down, and sortable as text. That made `2026-10-20` the one date order no country misreads.

It met its market slowly and then completely, though nobody implements all of it. The standard includes:

- **Two forms of everything:** basic and extended, `20261020T1504` and `2026-10-20T15:04`.
- **Other calendars of days:** week dates like `2026-W43-2` and ordinal dates like `2026-293`.
- **Partial values:** reduced precision, and decimal fractions on the smallest unit, so `T15.5` is half past three.
- **Spans:** durations like `P1Y2M`, intervals, repeating intervals, and `24:00` for the end of a day.

Later editions came in 2000, 2004, and 2019, the last splitting into a core and a part 2 of extensions. ISO 8601 is also a paid document, so most programmers who cite it have never read it. "ISO 8601 compliant" always means some subset.

Two profiles narrowed it:

- **The W3C note** "Date and Time Formats" (1997, Misha Wolf and Charles Wicksteed) defined six levels of precision, and XML Schema adopted it.
- **RFC 3339** (2002, Graham Klyne and Chris Newman) defined "Date and Time on the Internet: Timestamps": a four-digit year, a full date and time, a required offset, and an optional fraction.

Even the strict profile has variants. It allows a lowercase `t` and `z`. It permits a space in place of the T, for readability. It defines `-00:00` for a time whose local offset is unknown, and it allows second 60 for a leap second. Python's `datetime` is a good example of how a canonical form drifts. `str()` writes a space and `isoformat()` writes a T. Both leave out the fraction when the microseconds are zero, so the length of the output depends on the value, and a reader expecting a fraction fails on one value in a million. Python 3.6 added a `timespec` argument to fix the width on request.

JavaScript's `toISOString`, standardized in ES5 in 2009, made the other choice: always milliseconds, always Z, always 24 characters for the years 0 through 9999. That's the form ergo's tick writes.

**RFC 9557** (April 2024, Ujjwal Sharma and Carsten Bormann) came out of the IETF's SEDATE working group to give Temporal a standard text. It extends RFC 3339 with bracketed annotations, like `2022-07-08T00:14:07+01:00[Europe/Paris]`. A `!` marks an annotation as critical, so a reader must refuse a string whose offset and zone disagree. It also changed what Z means, 22 years on. Under RFC 3339, Z, like `+00:00`, implied that UTC was the preferred reference point. Under RFC 9557, Z means what `-00:00` used to mean: the instant is known, and the local offset isn't.

### The repeated-letter lineage: Taligent, Java, ICU, and CLDR, 1990s

The second great lineage began at Taligent, the joint venture Apple and IBM formed in 1992, whose engineers wrote frameworks for international software. Their work reached the world through Java.

**Java 1.0** (1996) shipped `java.util.Date`: a mutable object, years counted from 1900, months from 0, and a fixed `toString`. JDK 1.1 (1997) deprecated most of it, one release later, and brought in `java.text` from the Taligent team. That package contains:

- `Calendar` and `TimeZone`
- `DateFormat`, with four styles: `SHORT`, `MEDIUM`, `LONG`, and `FULL`
- `SimpleDateFormat`, with patterns like `yyyy.MM.dd G 'at' HH:mm:ss z`

It was built for internationalized applications, written once and run in any locale.

The pattern design repeats a letter to choose a width: `M` writes 6, `MM` 06, `MMM` Jun, `MMMM` June, and later `MMMMM` J. That handles the 7 / 07 / Jun / June axis more regularly than strftime ever did. IBM released the engine beneath it as ICU, International Components for Unicode, in C, C++, and Java, around 1999. ICU now formats dates under:

- Java
- Apple's `NSDateFormatter`
- Android
- the JavaScript engines of Chrome and Firefox
- Node
- .NET on Linux
- PHP's intl extension

The Unicode Consortium began CLDR, the Common Locale Data Repository, in 2003, to hold every locale's names, orders, and patterns. It specified the pattern letters in LDML, Unicode Technical Standard #35. CLDR publishes new data twice a year.

Two ideas in this lineage matter more than the letters. A **style**, like medium, lets the locale decide everything: "Oct 20, 2026" in the United States and "20 oct. 2026" in France. A **skeleton**, like `yMMMd`, names the fields wanted, and ICU's pattern generator finds the locale's arrangement of exactly those fields. This answers the fourth problem, whose language and order, and it's where the field's best thinking has gone since.

The letters, though, are where this lineage's famous footguns live:

- **`YYYY` is the week-numbering year.** It's the year a week belongs to, which differs from the calendar year for a few days around New Year. So `YYYY-MM-dd` passes every test written for most of the year and then prints the wrong year for a few days at its edge. On December 28 and 29, 2014, Twitter's servers believed it was already 2015, and users were signed out and couldn't sign back in. Twitter called it a bug in its front-end code. The episode became the standard cautionary tale for `YYYY`.
- **`DD` is the day of the year.** It agrees with the day of the month all through January, so the bug ships in January and appears on February 1.
- **Neighboring pairs.** `mm` is minutes and `MM` is months. `hh` is the 12-hour clock and `HH` the 24-hour one.
- **SimpleDateFormat is mutable and not thread-safe.** A formatter kept in a static field on a server corrupts its output under load, a rite of passage for a generation of Java engineers.
- **Its reader is lenient by default,** so month 13 rolls into the next year.
- **A user setting can rewrite a fixed format.** On iOS, a user's choice of 24-hour or 12-hour time could make `NSDateFormatter` rewrite a fixed format like `HH:mm`, breaking code that read dates from the Internet. Apple's Technical Q&A 1480 tells developers to set the locale to `en_US_POSIX` first, a locale "specifically designed to yield US English results regardless of both user and system preferences."
- **The data moves.** CLDR 42, in October 2022, replaced the plain space before AM and PM in English with U+202F, a narrow no-break space. Java 20 picked it up, and code that read "3:00 PM" by matching a plain space broke; JDK 23 added loose matching of spaces to cope. The browsers patched the change back out for web compatibility. V8 carries a function that turns U+202F back into a plain space, and a web-platform test now requires the plain space. So on one machine, Java and the browser format the same time differently.

What came next was more of the same lineage, with each footgun fixed in a second API beside the first:

- **Java.** Joda-Time began in 2002, by Stephen Colebourne, and became `java.time` in Java 8 (2014). It's immutable and thread-safe, with predefined ISO formatters and localized styles. Its letters brought a new trap: `y` is the year of the era and `u` the plain year, so in strict mode `yyyy-MM-dd` refuses to read a date without an era. Java now holds three date APIs, two of them largely deprecated.
- **.NET.** It took the same letters and added its own trap. In a custom format, `/` and `:` aren't literal characters but placeholders for the current culture's separators, so `MM/dd/yyyy` writes `10.20.2026` on a German machine. Its standard formats are single letters, including `o`, which Microsoft calls the round-trip format. `DateTimeOffset` came in 2008 because `DateTime` couldn't say which clock it meant. Jon Skeet's Noda Time ported Joda's ideas, and .NET 6 (2021) added `DateOnly` and `TimeOnly`.
- **Apple.** Swift's `FormatStyle` (2021) drops the pattern string: `date.formatted(.dateTime.year().month(.wide).day())` names the fields, and the locale arranges them.
- **ICU4X.** Unicode's own new implementation, in Rust, reached 2.0 in May 2025 with "semantic skeletons." A caller picks a field set like `YMD`, then options like its length, and a combination that makes no sense, like a month with a minute, can't be expressed at all. The Unicode blog describes it as easier to understand and harder to make mistakes with. The consortium that standardized the pattern letters now leads with typed field sets instead.

## B: the web

### JavaScript's Date, 1996

JavaScript 1.0 shipped in Netscape Navigator 2.0 in 1996 with a Date object. Allen Wirfs-Brock and Eich's history of the language says it plainly: "JavaScript 1.0 Date was a direct transliteration, bugs and all, of the java.util.Date class of Java 1.0." That included a millisecond time value counted from the start of 1970, months numbered 0 through 11, and the Year 2000 ambiguities of the Java design, all for interoperability with Java.

Java deprecated most of its Date a year later. JavaScript couldn't, because the web never removes anything. So `getYear()` still returns 126 in 2026, and `getFullYear()` exists beside it. ergo's tick, an integer of milliseconds since 1970, is that same count. It reached ergo from Unix by way of Java 1.0 and Netscape.

Date's text forms were left to each browser. What `toString` and `toLocaleString` wrote, and what `Date.parse` would read, was up to the implementation until ES5 (2009) specified a simplified ISO 8601 form. ES5 said a missing offset meant UTC. ES2015 changed that to local time, and the web broke. ES2016 split the difference: a date alone, `2026-10-20`, is read as UTC, and a date with a time, `2026-10-20T00:00`, as local. So `new Date("2026-10-20")` shows as October 19 anywhere west of Greenwich.

### Datejs, 2007

Geoffrey McGill's Datejs was built for reading what people type. `Date.parse("next thursday")` worked, and so did `Date.today().add(3).days()`. It met its market as a delight, and it did its work by patching JavaScript's built-in Date and Number. A library that patches built-ins collides with every other library that does, and with whatever the standard adds later. McGill stopped maintaining it in May 2008, and it lives on in old codebases and a community fork.

### Moment.js, 2011

Tim Wood's Moment was built to make Date bearable:

- **Formatting:** `moment().format("MMMM Do YYYY, h:mm:ss a")`
- **Arithmetic:** `add(1, 'day')` and its relatives
- **Relative time:** `fromNow()` writes "a few seconds ago"
- **Locales,** and later Moment Timezone, which shipped the time zone database in JavaScript

It met its market completely. For most of the 2010s Moment was what "dates in JavaScript" meant, and it was among the most depended-on packages on npm.

Its footguns were structural:

- **Mutable.** A moment was mutable, so `add` changed the object in place, and a moment shared between two parts of a program changed under one of them.
- **Lenient.** Its reader fell back to the browser's Date for text it didn't recognize, until a deprecation warning around 2015 began telling people so.
- **Heavy.** Every locale shipped in one bundle, and nothing could be tree-shaken.
- **Look-alike tokens.** Its letters look like LDML's but differ: in Moment, `YYYY` is the calendar year and `D` the day of the month. A developer moving from Moment to any LDML library writes `YYYY` and gets the week-numbering year, the reverse of the usual trap.

In September 2020 its maintainers published the project status quoted at the top of this document. They discouraged Moment for new projects, recommended Luxon, Day.js, date-fns, and js-Joda, and said they wouldn't redesign the mutable API. Moment's maintainers were also among Temporal's first champions. The Moment team's mea culpa took the form of fixing the platform.

### Moment's successors

- **date-fns** (2014, Sasha Koss) is functions over plain Dates, so a bundle carries only what it uses. Version 2 (2019) adopted Unicode's letters. It also throws on `YYYY` and `DD` unless the caller passes an option saying they meant the week-numbering year or the day of the year: a library guarding against the footgun of the standard it adopted.
- **Luxon** (2017, Isaac Cambron, a Moment maintainer) is immutable and built on the platform's Intl. It has tokens of its own and presets like `DateTime.DATE_MED` that hand display to the locale.
- **Day.js** (2018) is Moment's API in about 2 KB, immutable. Its popularity is the market's verdict on familiarity: developers chose the tokens they already knew over better ones.
- **js-Joda** ports Java's `java.time`.

### PHP, 1990s onward

PHP's `date()` uses bare letters with no percent sign:

- **Numbers:** `Y-m-d H:i:s`, where `i` is the minute, `j` the day without padding, and `n` the month without padding.
- **Names:** `D` and `l` for the weekday's name, `M` and `F` for the month's.
- **`S`** for an English ordinal suffix.

Any letter that's a code is replaced and any other passes through, so literal text has to be escaped letter by letter, as in `date("\T\o\d\a\y \i\s l")`. Its names are English only. The localized path was `strftime`, deprecated in PHP 8.1 (2021) in favor of ICU's `IntlDateFormatter`. PHP 5.2 (2006) added a DateTime class that was mutable. Its methods changed the object in place, which couldn't be fixed without breaking every caller, so PHP 5.5 (2013) added DateTimeImmutable beside it.

### Intl, 2012

The platform's answer was ECMA-402, the internationalization API, whose first edition came in December 2012. `Intl.DateTimeFormat` takes no pattern string at all. It takes a locale and an object of options naming fields and widths, like `{month: 'short', day: 'numeric', hour: 'numeric'}`, or a style, `{dateStyle: 'medium'}`, and a `timeZone` by its IANA name. CLDR decides the order, punctuation, names, and clock.

- **`formatToParts`** came later. It returns the pieces as a list, for a caller who wants to arrange them a particular way while keeping the locale's names and numbers.
- **`Intl.RelativeTimeFormat`** came later still and writes "3 days ago" in any language.

ECMA-402 has long declined requests to accept pattern strings, on the ground that a pattern fixes one locale's arrangement in place.

### Temporal, 2017 to 2026

Temporal began as a TC39 proposal in 2017, reached stage 3 in March 2021, and shipped in Firefox 139 (May 2025) and Chrome and Edge 144 (January 2026). It reached stage 4 at TC39's March 2026 plenary, and Node 26 enables it by default. Safari is the gap, as of October 2026.

Its design separates the kinds of time into types:

- **`Instant`:** a point on the timeline
- **`ZonedDateTime`:** an instant in a named zone
- **The plain types:** `PlainDate`, `PlainTime`, `PlainDateTime`, `PlainYearMonth`, and `PlainMonthDay`, for wall-clock values with no zone
- **`Duration`**

A program says which of the five problems it's solving by which type it holds. On text, Temporal does exactly two things. `toString()` writes one exact form that reads back, RFC 9557's, which Temporal's own champions took to the IETF. `toLocaleString()` hands display to Intl. There's no format-pattern API, on purpose. After thirty years of the web's libraries, the design its standards body settled on is one exact machine form and display chosen by the locale, with nothing in between.

## C: Python, Ruby, Go, and Rust

### Python

Python's `time` module wraps C's `strftime` and `strptime` directly, and Python inherits their quirks, including `%-d`'s failure on Windows. Python 2.3 (2003) added `datetime`. Its objects are either naive, with no zone, or aware, with a `tzinfo`. For years the standard library had no concrete zones at all: `timezone.utc` arrived in 3.2.

Its footguns came from that split:

- **`utcnow()`** returns a naive datetime holding UTC values. Many APIs then treat a naive datetime as local time, so it was a reliable way to be off by your offset. Python 3.12 (2023) deprecated it, and `utcfromtimestamp()` with it, twenty years after they arrived.
- **pytz** filled the zone gap for over a decade. Python's `tzinfo` interface didn't fit real zones, so pytz required a `localize()` call. Passing a pytz zone directly as `tzinfo=` produces that zone's local mean time from the 1800s, so New York comes out four hours and 56 minutes behind UTC. pytz's own documentation warns about it.
- **zoneinfo** (Python 3.9, 2020, PEP 615, Paul Ganssle) finally put the zone database behind the standard interface.
- **Readers.** `fromisoformat` (3.7) at first read only what `isoformat` wrote; 3.11 widened it to most of ISO 8601.

Display for people lives outside the standard library. Babel brings CLDR, with LDML patterns and styles. Arrow and Pendulum bring Moment-style tokens and phrases like "3 days ago." An f-string reaches strftime directly: `f"{d:%Y-%m-%d}"`.

### Ruby and Rails

Ruby (1995, Yukihiro Matsumoto) implements `Time#strftime` itself rather than calling the C library, so its flags behave the same on every platform:

- **Padding:** `%-d` for none, `%_d` for spaces.
- **Case:** `%^b` for an uppercase month, `%P` for a lowercase am or pm.
- **Fractions:** `%L` for milliseconds, and `%N` for nanoseconds, or any number of digits with a width like `%3N`.

It's probably the most polished strftime there is. The standard library also has Date and DateTime, and Ruby's own documentation now says DateTime "is considered deprecated."

Rails (2004, David Heinemeier Hansson) didn't invent a pattern language. It added three ideas on top of strftime, and they're the Rails contribution:

- **Named formats.** ActiveSupport's `Time::DATE_FORMATS` maps a name to a strftime string or a lambda: `:db` for `2026-10-20 15:04:05`, `:short`, `:long`, `:rfc822`, `:number`, and any an application registers. A format is declared once and used everywhere by name, so it becomes a word in the application's vocabulary rather than a string repeated at each call site. Rails 7 (2021) moved the feature from `to_s(:short)` to `to_fs(:short)`, off the `to_s` that Ruby calls for every string interpolation.
- **Translations.** `I18n.l` (Rails 2.2, 2008) looks the named format up per locale, in YAML, and month and day names come from the locale file instead of the C library.
- **Words.** ActionView's `time_ago_in_words` and `distance_of_time_in_words` write "less than a minute," "about 1 hour," "over 2 years."

Rails 2.1 (2008) added `TimeWithZone` and `config.time_zone`, and with them a footgun of its own. `Time.now` ignores the configured zone, while `Time.zone.now` and `Time.current` honor it, so rubocop-rails has a cop whose whole job is catching `Time.now`.

Basecamp's `local_time` gem is the most honest answer to the second problem anywhere in this survey. The server writes UTC into a `<time>` element, and JavaScript in the reader's browser rewrites it into the reader's zone and language. The server stops pretending to know where its reader is.

### Go

Go, released in 2009, formats by example. A layout is the reference moment, `Mon Jan 2 15:04:05 MST 2006`, written the way any moment should look. The reference moment is the Unix `date` command's default output, in that order, and written numerically it's `01/02 03:04:05PM '06 -0700`: the numbers one through seven. So `"Jan 2, 2006 3:04 PM"` is a layout, and the same layout reads text back.

It was built for readability at the call site, a layout that looks like its output, with nothing to memorize but one moment. The package names the common layouts: `ANSIC`, `UnixDate`, `RFC1123`, `RFC3339`, `RFC3339Nano`, and `Kitchen` for `3:04PM`. Go's defenders love it, and its critics say they've memorized a date instead of a set of letters. The footguns are distinctive:

- **Silent swaps.** Writing `2006-02-01` swaps the month and day with no error; a 2021 proposal noted that 01 and 02 "are easy to mix up." And the mnemonic itself is American, month first.
- **Foreign layouts print verbatim.** `YYYY-MM-DD` contains no reference values, so Go writes it out literally, and every date becomes the text "YYYY-MM-DD."
- **Lookalike literals.** A digit in literal text that happens to match a reference value gets replaced.
- **Small spellings with big differences.** `.000` keeps trailing zeros and `.999` trims them, `_2` pads the day with a space, and `15`, `03`, and `3` are three different hours.
- **Zone abbreviations.** Reading one the given location doesn't know, Go records the time "in a fabricated location with the given zone abbreviation and a zero offset," so EST can come back as UTC wearing EST's name.

What came next shows a project holding its line. Proposals to add strftime-style tokens kept arriving. To one in 2020, a contributor answered, "This issue has been raised a lot of times already, and it has been declined every time," and the resolution was a rewrite of the layout documentation. Then in 2022 Joe Tsai surveyed all the Go code on the module proxy:

- `2006-01-02 15:04:05` was written out 13,312 times.
- `2006-01-02` was written out 12,332 times.

That was more often than most layouts that already had names, so Go 1.20 (2023) added `DateTime`, `DateOnly`, and `TimeOnly`. A suggestion to name them for ISO 8601 was turned down because the space in `DateTime` isn't ISO 8601. Go's standard library has no localized names; months are English.

### Rust

Rust's standard library has `SystemTime` and `Duration` and no calendar, by design, so the community's crates carry it, in three schools.

**chrono** (2014) is strftime, its own implementation with the GNU flags and `%.3f`, plus `to_rfc3339` and optional locales. To find the local offset it called the C library's `localtime_r`. That function reads the `TZ` environment variable, which another thread may be writing, and Rust assumes the environment is writable. The race became advisory RUSTSEC-2020-0159. chrono 0.4.20 (2022) fixed it by reading the time zone database in Rust.

**time** (0.3, from 2021, maintained by Jacob Pratt) faced the same race. For a period its local-offset methods simply failed on Unix rather than risk it, and it still offers `set_soundness` for a caller who insists. Its format descriptions are a bracketed language of named fields, parsed and checked at compile time by a macro:

- `[year]-[month]-[day] [hour]:[minute]:[second]` for the common case
- modifiers like `[month repr:short]`, `[day padding:none]`, `[hour repr:12]`, `[period case:lower]`, and `[subsecond digits:3]`

It's wordy, there's nothing to memorize, and a typo is a compile error.

**jiff** (2024, Andrew Gallant) is Temporal for Rust:

- **Text:** values display in RFC 9557's form, which reads back exactly.
- **Formatting:** `jiff::fmt::strtime` is strftime and strptime, chosen for familiarity.
- **Durations:** a "friendly" format like `1h 30m` that also reads back.
- **Zones:** it reads or bundles the time zone database.
- **Locales:** display goes to ICU4X through `jiff-icu`.

It's at 0.2 as of 2026. The 1.0 planned for summer 2025 slipped.

## Same letter, different meaning

The clearest way to see the field's disagreement is to ask what one letter means in each system.

| | `M` | `m` | `D` | `Y` |
|---|---|---|---|---|
| strftime | `%M` the minute | `%m` the month, 10 | `%D` the whole date, 10/20/26 | `%Y` the year |
| LDML: ICU, Java, Apple, date-fns | the month | the minute | the day of the year | the week-numbering year |
| Moment, Day.js | the month | the minute | the day of the month | the year |
| PHP `date()` | Oct | the month, 10 | Tue | the year |
| MySQL `DATE_FORMAT` | `%M` October | `%m` the month, 10 | `%D` 20th | `%Y` the year |
| Excel | the month, or the minute beside an hour or second | the same | the day | the year |

The month alone shows how far the lineages have drifted. Here's July in each well-known system, as a number, padded, short, long, and as the single letter a calendar's column header uses:

| | 7 | 07 | Jul | July | J | Also |
|---|---|---|---|---|---|---|
| C `strftime` | none | `%m` | `%b` | `%B` | none | POSIX adds `%h`, the same as `%b` |
| GNU, BSD, Ruby, chrono | `%-m` | `%m` | `%b` | `%B` | none | glibc and Ruby write JUL for `%^b` |
| Windows C runtime | `%#m` | `%m` | `%b` | `%B` | none | |
| Python `strftime` | `%-m` or `%#m` | `%m` | `%b` | `%B` | none | it hands the pattern to the platform's C library, so no way of dropping the zero works everywhere |
| LDML: ICU, Java, Apple, date-fns | `M` | `MM` | `MMM` | `MMMM` | `MMMMM` | `L` through `LLLLL` for the month standing alone, which some languages inflect differently |
| Moment, Day.js | `M` | `MM` | `MMM` | `MMMM` | none | Moment's `Mo` writes 7th |
| Luxon | `M` | `MM` | `MMM` | `MMMM` | `MMMMM` | `L` through `LLLLL`, as in LDML |
| .NET | `%M` | `MM` | `MMM` | `MMMM` | none | a lone `M` is the standard month-and-day format, July 4, so the number alone is `%M` |
| Excel | `m` | `mm` | `mmm` | `mmmm` | `mmmmm` | `m` and `mm` are the minute beside an hour or a second |
| PHP `date()` | `n` | `m` | `M` | `F` | none | |
| MySQL `DATE_FORMAT` | `%c` | `%m` | `%b` | `%M` | none | `%M` is the minute in every other strftime |
| Oracle, PostgreSQL `TO_CHAR` | `FMMM` | `MM` | `Mon` | `Month` | none | the pattern's case sets the output's, so `MONTH` writes JULY; `Month` pads to nine characters without `FM`; `RM` writes VII |
| Go | `1` | `01` | `Jan` | `January` | none | the reference time's own month |
| Rust `time` | `[month padding:none]` | `[month]` | `[month repr:short]` | `[month repr:long]` | none | |
| `Intl.DateTimeFormat` | `month: 'numeric'` | `month: '2-digit'` | `month: 'short'` | `month: 'long'` | `month: 'narrow'` | options, not a pattern, and the locale arranges them |
| Swift `FormatStyle` | `.month(.defaultDigits)` | `.month(.twoDigits)` | `.month(.abbreviated)` | `.month(.wide)` | `.month(.narrow)` | named fields, and the locale arranges them |

And the 12-hour clock and milliseconds, in five of them:

| | strftime | LDML | Moment | Go | Rust `time` |
|---|---|---|---|---|---|
| 3, as in 3 PM | `%-I`, an extension | `h` | `h` | `3` | `[hour repr:12 padding:none]` |
| PM | `%p` | `a` | `A` | `PM` | `[period]` |
| .026 | none in C | `SSS` | `SSS` | `.000` | `[subsecond digits:3]` |

## The classic collisions

The collisions that bite come in a few shapes, and the same ones turn up in system after system.

**Within one system**, two fields that sit side by side in a pattern differ only by case or by what's next to them:

- **Month and minute.** `M` and `m` in LDML, `%m` and `%M` in strftime, and in Excel one `m`, decided by whether an hour or a second stands beside it.
- **Second and its fraction.** LDML's `s` is the second and `S` a fraction of it. In Java's SimpleDateFormat, `S` was a count of milliseconds rather than a fraction, so `ss.SS` wrote 1.007 seconds as `01.07`.
- **Year and week-numbering year.** `yyyy` and `YYYY`, above.
- **Day of the month, of the year, and of the week.** LDML's `d`, `D`, and `E`, where `DD` agrees with `dd` all through January.
- **The 24-hour and 12-hour clocks.** `HH` and `hh` in LDML, `%H` and `%I` in strftime, and `hh:mm` without AM or PM writes a time that's ambiguous half the day. LDML adds `k` and `K`, a 24-hour clock from 1 to 24 and a 12-hour clock from 0 to 11.
- **The zone's name and its offset.** In strftime, `%Z` writes PDT and `%z` writes -0700. LDML has them the other way around, `z` and `Z`, along with `X`, `x`, `O`, `v`, and `V`.

**Letters with no mnemonic** have to be looked up every time, by the writer and by every reader after:

- strftime: `%b` for the short month, `%e` for the day padded with a space, `%I` for the 12-hour hour, `%j` for the day of the year, and `%p` for AM or PM.
- PHP: `F` for the month's name and `M` for its short form, `n` for its number, `j` for the day, `i` for the minute, `z` for the day of the year counted from zero, `N` for the weekday's number, and `l`, a lowercase L, for its name.
- MySQL: `%i` for the minute, `%c` for the month's number, and `%W` for the weekday's name.
- LDML: `E` for the weekday, `a` for AM or PM, `G` for the era, and `F` for which Wednesday of the month it is.

**strftime and LDML flip.** The two big families give many of the same letters to different fields, and some pairs swap their cases between the two:

| Letter | strftime | LDML |
|---|---|---|
| `m` | the month | the minute |
| `M` | the minute | the month |
| `s` | seconds since 1970, an extension | the second |
| `S` | the second | a fraction of a second |
| `h` | the short month, like `%b` | the hour, 1 to 12 |
| `y` | the year in two digits | the year |
| `Y` | the year | the week-numbering year |
| `D` | the date, like 10/20/26 | the day of the year |
| `a` | the short weekday | AM or PM |
| `e` | the day of the month, padded with a space | the weekday as a local number |
| `u` | the weekday, 1 for Monday | the extended year |
| `w` | the weekday, 0 for Sunday | the week of the year |
| `z` | the offset, -0700 | the zone's name, PDT |
| `Z` | the zone's name, PDT | the offset, -0700 |

**Across systems**, a letter familiar from one means something else in the next:

- MySQL kept strftime's `%` and moved the letters. Its `%M` is the month's name, `%h` the 12-hour hour, `%c` the month's number, `%D` the day with a suffix, like 20th, and `%W` the weekday's name. In strftime those are the minute, the short month, the whole date and time, the date like 10/20/26, and the week of the year.
- PHP's `D` is the short weekday, Tue, where LDML's is the day of the year and Moment's the day of the month. Its `S` is a suffix like th, where LDML's is a fraction of a second, and its `Z` is the offset in seconds, -25200.
- Moment's `YYYY` and `DD` are the calendar year and the day of the month, and its users carry the habit to LDML libraries, where they're the week-numbering year and the day of the year; that's why date-fns throws on both. Moment's `d` is the weekday's number and `dd` its two-letter name, Tu, where LDML's `d` is the day of the month.
- Luxon's `D` is a whole localized date, like 10/20/2026.
- Oracle's and PostgreSQL's `HH` is the 12-hour hour. The 24-hour hour is `HH24`.
- .NET's `K` writes the zone as Z, an offset, or nothing, depending on the kind of DateTime it's given, and its `z` writes the offset's hours, -7, where LDML's `z` is the zone's name. A lone `M` is a whole standard format, not the month.
- java.time's `u` is the year, where SimpleDateFormat's was the weekday's number, so a pattern moved from one to the other changes meaning without an error.
- Go reads anything that looks like part of its reference time as a field and the rest as text. `YYYY-MM-DD` comes out unchanged, `Version 1: 2006` comes out `Version 10: 2026`, and the word Monday in a layout becomes the date's weekday.

**Two root causes** account for nearly all of it:

1. **The alphabet runs out.** A moment has more parts than there are letters, once widths, case, the 12-hour clock, week-numbering years, and zones are counted. So every system reuses letters: by case, like `M` and `m`; by count, like `M` through `MMMMM`; and by neighbors, like Excel's `m`. Each reuse is a pair a reader has to keep apart.
2. **Lineages drift.** Each system copied an earlier one and changed what bothered it. MySQL kept strftime's `%` and moved its letters, Moment kept LDML's letters and moved `Y`, `D`, and `d`, and java.time kept SimpleDateFormat's and moved `u`. The letters stay familiar while their meanings change, so a habit that serves in one system misleads in the next.

## Relative time

"3 minutes ago" is everywhere and standardized nowhere:

- **Ruby and JavaScript libraries:** Rails' `time_ago_in_words`, the timeago plugin for jQuery, and Moment's `fromNow`
- **Python libraries:** `humanize` and Pendulum's `diff_for_humans`
- **The platform:** `Intl.RelativeTimeFormat`, which supplies the words in every language but leaves the thresholds to the caller

Every library picks its own thresholds, and none is wrong. Moment switches from seconds to "a minute" at 45 seconds, to hours at 45 minutes, to days at 22 hours, to months at 26 days, and to years at 11 months, all configurable. Rails draws its lines elsewhere. The other footgun is time itself: the text is true when it's written, it goes stale on the page unless something rewrites it, and it can never be read back.

## What the footguns teach

Sorted by kind, the failures above fall into a few families, and each family teaches one thing.

1. **Letters collide.** Every pattern language spends a small alphabet on many fields, and the difference between two fields is often only capitalization. The worst of these bugs pass every test, because they agree with the right answer most of the year: `YYYY` is wrong for a few days in December, and `DD` is right all of January.
2. **Hidden state changes the answer.** Several things outside the call change its output: `setlocale`, .NET's current culture, the `TZ` environment variable, a user's 24-hour setting, and a server's region. The same code writes different text on a different machine, a different thread, or a different user's phone.
3. **Lenient reading can never be undone.** HTTP's three formats, the cookie algorithm, `Date.parse`, Moment's fallback, SimpleDateFormat's rollover, and Excel's genes all show it. Once a reader accepts a variant, writers produce it, and the reader can never be made strict again.
4. **Numbers get mistaken for dates.** The year counted from 1900, the month counted from zero, a date packed into a 32-bit integer, 2038, and Lotus's leap day in 1900 are all one mistake: a convention about a number that nobody wrote down where the number goes.
5. **Variable width breaks readers.** Python's missing fraction and Go's trimmed zeros make the length of the text depend on the value, and a reader built from examples fails on the rare value.
6. **The data moves underneath you.** CLDR publishes twice a year, and the time zone database changes whenever a government does. "Format this date" has an answer that depends on data the library doesn't own, versioned separately on every machine.
7. **Mutable objects get shared.** `asctime`'s static buffer, SimpleDateFormat, Moment, and PHP's DateTime each let one part of a program change what another part was holding.

## The shape of the progress

Across fifty years the designs move in one direction, from strings toward structure. Each step makes another kind of mistake impossible to write:

- **Codes,** strftime, 1977 to 1989: compact and universal, with arbitrary letters and global state.
- **Repeated letters,** Taligent and Java to LDML, the 1990s: width by repetition, and the locale's styles and skeletons beside them.
- **An example,** Go, 2009: the layout looks like its output.
- **Named fields:**
  - Intl's options object, 2012
  - the `time` crate's brackets, Swift's FormatStyle, 2021
  - ICU4X's field sets, 2025, where a nonsensical combination doesn't compile
- **Types for kinds of time,** Joda-Time and `java.time`, 2014, then Temporal and jiff: a program says which of the five problems it's solving by which type it holds.

The pattern string never disappears. Every modern system keeps one for the caller who needs an exact shape. But it moves from the front door to an escape hatch. The front door becomes two things: one exact form that reads back, like RFC 3339 and RFC 9557, and display chosen by the locale from fields and a style.

## Why everyone keeps trying, and keeps apologizing

Every language needs this on its first day, and the first version is easy. strftime is an afternoon's work, and it looks finished. The failures arrive later and on a schedule nobody tests against: on New Year's Eve, on the first of February, on a server in Germany, on a phone set to 24-hour time, on the day CLDR or a government changes its data. By then the first API has callers, and it can't change without breaking them. So the fix is always a second API beside the first:

- Java's three date APIs
- Python's deprecation of `utcnow` after twenty years
- PHP's DateTimeImmutable
- Ruby's deprecated DateTime
- C's deprecated `asctime`
- JavaScript's Temporal after thirty years

The mistake was public and permanent before it was visible, so the mea culpa is the normal ending of the story rather than a rare one. Each new attempt is also easy to start, which is why there are always more of them.

## Where ergo already stands

ergo has the first layer, the exact form, and two of them:

- **The iso form** is `toISOString`'s: always 24 characters, always milliseconds, always Z. That avoids the variants RFC 3339 allows, a space, a lowercase t, a missing fraction. Its Z now says, under RFC 9557, exactly what a tick is: an instant, with no claim about anyone's local clock.
- **The short form** came with ergo from the code it grew from, like `2012aug6.0517.57300`, which reads back as exactly as the iso form does. It's also readable by a person, which is more than the machine forms in this history usually manage.

Both readers accept only what their writers produce. That's the strictness the third lesson above argues for, and the one RFC 3339's and HTTP's histories show is impossible to add later.

The tick, an integer of milliseconds since 1970, is the count Java 1.0 chose and JavaScript copied. It's held within 1970 through 9999, so no form ever needs a fifth digit of year.

A display format for people would face the same five problems. The first and third are pure arithmetic and text, and ergo's vectors can pin them in four editions. The second and fourth are where every system above found its hardest failures. They depend on the time zone database and on CLDR, data that every platform carries at its own version, and that only the JavaScript edition has on the platform.

## Sources

- Allen Wirfs-Brock and Brendan Eich, [JavaScript: The First 20 Years](https://zenodo.org/record/3707008), HOPL IV, 2020, on Date as a transliteration of java.util.Date
- [C date and time functions](https://en.wikipedia.org/wiki/C_date_and_time_functions), on strftime's roots in PWB/UNIX 1.0 and ANSI C 1989, and the [ANSI C draft man page](https://stuff.mit.edu/afs/sipb/project/sipb-old/src/inbsd/tin/strftime.3) noting the System V `cftime` and `ascftime` conversions
- [RFC 1123](https://www.rfc-editor.org/rfc/rfc1123), [RFC 3339](https://www.rfc-editor.org/rfc/rfc3339), and [RFC 9557](https://www.rfc-editor.org/rfc/rfc9557.html), on military zones, the internet timestamp, and the change to Z
- [Moment.js project status](https://momentjs.com/news/), and [The Register's coverage](https://www.theregister.com/2020/09/15/moment_js_javascript_library_future/), September 2020
- [Datejs: Evolved](https://unpkg.com/datejs@1.0.0-rc3/README.md), on the original's history
- [Temporal Reaches Stage 4](https://www.igalia.com/2026/03/13/Temporal-Reaches-Stage-4.html), Igalia, March 2026
- [ICU4X 2.0 released](https://blog.unicode.org/2025/05/icu4x-20-released.html), Unicode blog, May 2025, on semantic skeletons
- [Technical Q&A 1480](https://developer.apple.com/library/archive/qa/qa1480/_index.html), Apple, on `en_US_POSIX`
- [JDK 20 to 23 and CLDR 42](https://nipafx.dev/unicode-cldr-42/), on the narrow no-break space, and the [web-platform test](https://asuth.searchfox.org/firefox-main/source/testing/web-platform/tests/ecmascript/locale-compat.html) requiring a plain space
- Go issues [#52746](https://github.com/golang/go/issues/52746) (DateTime, DateOnly, TimeOnly), [#38871](https://github.com/golang/go/issues/38871) (declined format tokens), and [#45645](https://golang.org/issue/45645) (01 and 02)
- [chrono issue #499](https://github.com/chronotope/chrono/issues/499) and [RUSTSEC-2020-0159](https://rustsec.org/advisories/RUSTSEC-2020-0159.html), on `localtime_r`, and the `time` crate's [local_offset utilities](https://docs.rs/time/0.3.36/time/util/local_offset/index.html)
- [jiff](https://github.com/BurntSushi/jiff), on its design and its 1.0 timeline
- Matt Johnson-Pint, [JavaScript Date Parsing Changes in ES6](https://codeofmatt.com/javascript-date-parsing-changes-in-es6/)
- Coverage of Twitter's sign-in outage, [Engadget](https://www.engadget.com/2014-12-28-twitter-login-bug.html) and [TechCrunch](https://techcrunch.com/2014/12/28/twitters-engineers-currently-fixing-widespread-sign-in-issue), December 2014

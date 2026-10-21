# say time design

Why `sayTick` and `sayDuration` work the way they do. Written in October 2026, right after the two were designed in a long conversation, so the reasons are on file while they're fresh. date-history.md tells what the rest of the field has done. This tells what ergo chose, what it gave up, and which mistakes it was built to avoid. The tests in the JavaScript suite show both functions working, and what happens under the reasonable mistakes a caller will make. This is the part the tests can't say: why.

## Where they sit

ergo already had two text forms for a tick, the iso form and the short form like `2012aug6.0517.57300`, and both read back exactly. `sayTick` and `sayDuration` are different in kind. They're one way, written for a person, and nothing ever reads them back. That freedom shapes everything below. A reader has to resolve every ambiguity a writer could create, so the round-trip forms each allow exactly one shape. A writer with no reader can let a template take any shape at all, and leave the meaning to whoever reads the result.

The names come from `saySize4`, ergo's first function that writes a value for a person and has no reader. `say` is the prefix for that promise. `sayTick` says a moment, and `sayDuration` says a span.

Both started in JavaScript alone, to be ported to Python, Rust, and Go once the design proved itself. Both are pure functions of a number and a template, so every case can become a shared vector when they're ported. Because nothing reads them back, there's no round trip to fuzz, so their tests are sanity checks and demonstrations rather than speed and fuzz.

## Four things left out

Four exclusions came first, and they're what make everything else possible.

**No reading.** The two round-trip forms are for reading. A reader for any template would have to accept every ambiguity a template can create, and ergo's rule is one text for one value.

**No two-digit years.** A tick runs from 1970 through 9999, so a year is always four digits, and `YYYY` is the only year there is.

**No time zones.** A tick is UTC, and `sayTick` writes the fields of the tick it's given as they fall in UTC. The stance is plain: get your data in UTC, keep it that way, and if a person should see their own wall clock, shift the tick by their offset at the very end, just before saying it. `sayTick` works the same for a moment anywhere on earth, and doesn't know or care which zone the caller has in mind. The reason is the vectors. Time zone rules come from a database that differs by platform and by version, and Rust's standard library has none at all, so a formatter that read the machine's zone would give answers no shared suite could pin. Getting an offset wrong, like the sign of `getTimezoneOffset`, is a pitfall of the platform, not of ergo.

**No localization.** The month and day names are English, one form the same everywhere, versioned with ergo. A page that wants its reader's language hands the tick to the browser's Intl. CLDR, the data behind every localized formatter, changes twice a year, and it once changed the space before "PM" in English to a different character. Only the JavaScript edition has it on the platform. Localized output belongs to the platform's data, and ergo stays out of it.

With those four gone, every answer is plain arithmetic on a number, and the same in four editions.

## sayTick

### Why the calendar is uppercase and the clock is lowercase

This is the question a new user asks first, and the answer is the oldest footgun in the field. Month and minute both start with M. strftime gives the month `%m` and the minute `%M`; LDML, the lineage behind Java and ICU, gives them `M` and `m`. A swap is the one mistake that writes believable wrong output: a minute of 05 in the month's place reads as May, and a month of 08 in the minute's place reads as eight past.

ergo spends case on a single distinction, calendar against clock, and that turns the swap into something you can see. In `YYYY-MM-DD hh24:mm:ss`, the calendar is a run of capitals and the clock is a run of lowercase. `MM` fits between `YYYY` and `DD`. A lowercase `mm` among the calendar's capitals, as in `YYYY-mm-DD`, sticks out, and so does a capital `MM` among the clock's lowercase. The case is a safety catch on the footgun. It agrees with LDML, so a Java or ICU habit holds, and someone with strftime habits, where the month is lowercase, gets a template that looks wrong before it runs.

Since case is spent, the hour can't use it to say which clock it means, the way LDML's `H` and `h` do. So the hour says it in digits: `h12` and `h24`, with `hh12` and `hh24` for a leading zero. Nobody has to remember which case was the 24-hour clock.

### The minute and second are always two digits

The minute is `mm` and the second is `ss`, always two digits, and there's no tag for one digit. Five past one is 1:05, and 1:5 is never what anyone wants. The first draft had `m` and `s` for these, and changing them to `mm` and `ss` paid three times. `mm` is what every other system's habit types, so the trap where `mm` wrote the minute twice, `05:1717`, disappeared. A lone `m` or `s` became just a letter, so `a.m.` and `p.m.` keep their periods, the way the Associated Press writes them. And the log format of the original, the JavaScript ergo grew from, became expressible, below.

The fraction of a second is `t`, `tt`, and `ttt`, as many digits as letters, rounded down. Rounding down is the only correct choice for a clock: rounding up would turn 59.999 into 60.

`AA`, `aa`, `A`, and `a` write PM, pm, P, and p. Pairing them with a twelve-hour hour is the template's job, and there's no hidden coupling. Put `h24` beside `AA` and you get `15 PM`, which is wrong in a way you can see.

### How many letters, and which case

One or two letters write a number and three or four write a name: `M` and `MM` are the month's number, `MMM` and `MMMM` its name. The day follows the same scheme, and a day's name is its weekday, so `D` and `DD` are the day of the month and `DDD` and `DDDD` the day of the week. Excel works this way, and so does Moment, so it's familiar to more people than any other scheme.

There's no day of the year and no week of the year. Leaving them out keeps `YYYY` the calendar year and `DD` the day of the month, which is what people guess. In LDML, `YYYY` is the year a week belongs to, wrong for a few days each December, and `DD` is the day of the year, right all through January. Those two traps can't happen here.

A name comes out in the case of its tag: `MMMM` writes AUGUST, `Mmmm` writes August, and `mmmm` writes august. The tag looks like its output, which is Oracle's old idea. The cost is that every other system's `MMMM` writes "August", so habit gets AUGUST the first time. That mistake is loud, not silent.

The names are exported as `monthNames` and `dayNames`, whole and in title case, the way a sentence writes them, so code only ever needs uppercase or lowercase to get the other two. Every short name in English is the first three letters of the whole one, so one list makes every form, and the tick's short form clips its `jan` through `dec` from the same list. They're frozen, so no caller can change them under ergo. Sunday comes first in every edition, the way JavaScript and Go count. Python's own `weekday()` starts with Monday, so a Python caller who wants the week the other three keep gets it from ergo. That's a feature.

### One pass, and no escaping

A template is read once, left to right, taking the longest tag at each position, and each value goes straight to the output, never to be read again. Picture cutting a candy bar: the pieces are cut from the template once, and never glued back and cut again. That rules out the two ways a chain of replacements goes wrong. A value can't become a tag, so the A in AUG stays a letter rather than turning into AM. And removing a tag can't push its neighbors together into a new one. The expression lists every tag by hand, longest first. That order matters, since a regular expression takes the first alternative that matches rather than the longest, and the hand-written line doubles as the readable list of every tag.

There's no escaping, on purpose. Anything that isn't a tag is written as it is, so `YYYYY` writes the year and a Y. Words don't belong in a template, since `M`, `D`, `t`, `A`, and `a` are tags, and so are `mm` and `ss`. Say the date with a small template, and put the result into the sentence, page, or log line around it, using the language's own strings. The language is the escape character.

### The name, and the original

The original already has a `sayTick`, in its guard layer, with one argument and a fixed log format, like `Fri04:09p39.470s`, and the application around it calls it in thirteen places. ergo took the name anyway. The general function deserves it, and the original's format is one template of the general one: `Dddhh12:mmass.ttts`, after shifting by the local offset. The trailing s is a letter only because the second became `ss`. When that application moves to ergo, each of those thirteen calls becomes a template or a one-line wrapper.

## sayDuration

### What it's for

A span of time off the calendar, for a person to read: how long something took, or how long ago it happened. The template is the sentence it writes, and `#` marks where each number goes, like `about # hours ago` or `#h #m #s`.

### Why #, after 0 and N

The placeholder took three tries, and each one taught something.

**0** came first, after Excel's and .NET's number formats, where 0 is a digit placeholder. It failed twice. A template like `0 years, 0 months, 0 days` reads like output already filled in. And a number of the template's own that ended in 0 grabbed the unit after it, so `over 10 days` came out `over 15 days`.

**N** read as a placeholder, and matched `sayTick`'s letters standing in for digits. But a capital N had to be told apart from a lowercase n, and once the units could be written in any case, a word like NY became a tag. `Ny` and `Nm` also read like syllables.

**#** means "number" everywhere, as in "# of items". It isn't a digit, so the template's own numbers stay text. It isn't a letter, so it can't collide with a word, and the rule about capital N left the code entirely. It costs two things. A hashtag like `#hiring` holds a tag, `#h`. And Excel's `#` is an optional digit, while every `#` after the point here is always written, so `#.###` says `1.500`, the way Excel's `0.000` would.

### A tag is the pair

The key idea came from a question: would the d in "and" be read as days? It isn't, because a tag is the `#` and its unit together, and a unit letter only means anything right after a `#`. So, unlike `sayTick`, a duration template can hold words: `about # hours ago`, `landed # days ago`, `# hours and # minutes`. A `#` with no unit after it is just a `#`.

Between the `#` and its unit can go whitespace, and nothing else. That's what keeps the pair a pair: a `#` with a word after it that isn't a unit, like `# of 10 days`, stays text, rather than reaching past the word for a unit further on and counting days. Whitespace is any character Unicode calls white space, so a tab and a line break count, and so does the no-break space a typographer puts there to keep "5 days" on one line, and its narrow and thin cousins. The byte order mark and the zero width space don't, since they're invisible without being space. Each of the four languages has its own idea of whitespace, and they disagree by a few characters: JavaScript's `\s` counts the byte order mark, Python's `isspace` counts four control separators, and Go's `\s` is five ASCII characters. So JavaScript and Rust ask Unicode's own property, and Python and Go write its list out, and every edition reads the same characters.

Only seconds take a fraction, `#.#`, `#.##`, or `#.###`. A fraction in front of any other unit isn't a tag, and the whole thing is kept as text, so that its last `#` can't pair with the unit on its own and write something like `0.05 minutes`. A fraction with no unit after it is text the same way, since the template is read once, left to right: a `#` read as part of a fraction never starts a tag of its own, so `#.##.#secs` says `#.##.3600secs` for an hour. Rust's scanner first backed up and tried the fraction's later `#` again, and a fuzz of random templates caught it disagreeing with the three regular expressions.

### Splitting the span

A template names some units and lists them from largest to smallest. The largest takes everything above it, so a year said in days is 365, and each smaller one takes what's left. Skipping units is fine: for a day, two hours, and five minutes, `# days # minutes` says `1 day 125 minutes`. Everything rounds down, so whatever is smaller than the smallest unit goes unsaid. That's also what the feeds do: a post 3 hours and 59 minutes old says `3h`.

A year and a month are `Time.year` and `Time.month`, exactly: 365 and a quarter days, and a twelfth of that. ergo has to agree with itself, and a span has no calendar to consult. The consequence was weighed and accepted. A span measured between two calendar dates a year apart is 365 days, which is a quarter day short, so `# years ago` says `0 years ago` and a full template says `11 months, 30 days`. A span can't know which calendar years it crossed; counting calendar years needs the two ticks, not their difference. It was accepted because reading something posted exactly a period ago is unusual, and because days and everything smaller are exact.

The arithmetic is exact too. Rounding down by dividing could, in principle, round the wrong way near a unit boundary, so it was checked: 4.2 million spans just above and below unit boundaries across the integer range, compared against exact big-integer division, and none were wrong. Within the integer ceiling, the quotient's rounding error is always smaller than its distance to the next whole number.

### Zeros: drop the start, keep the rest

Units that come out zero at the start drop away, along with the text after them, and from the first unit that isn't zero, everything stays. That lets one template count up a task of any length: `#d #h #m #.##s` says `32m 55.45s` half an hour in, never `0d 0h 32m 55.45s`. A zero between two others stays, so a day and a second is `1 day 0 hours 0 minutes 1 second`, never `1 day 1 second`, which reads like a slip. And the smallest unit a template names is the precision it asks for, so it always shows, even on an exact amount: `#m #s` says `1m 0s` for exactly a minute. If every unit is zero, the smallest says so, like `0 minutes`. A unit counts as zero by what's written, so `0.0 seconds` is zero. Text before the first tag and after the last one always stays, which is what lets `about # hours ago` work.

This rule is the second draft. The first trimmed zeros at both ends, so `#m #s` said `1m` for exactly a minute, and `# hours and # minutes` said `2 hours`. Reading the countdown in ergo's ancestor from 2013, which kept `1m 0s` for a steady width, showed the better meaning: the start of a template says how large a span it's ready for, and its end says how precisely to say it. Trimming the end threw that second meaning away. Keeping it fixed two pitfalls along the way: `# days (# hours)` no longer loses its parenthesis to say `1 day)`, and a list built with "and" keeps it at the end. A list can still stumble at the start, since `# hours, # minutes, and # seconds` says `5 minutes, and 3 seconds` when the hours drop away.

The cost lands on templates that name more precision than a span needs. The full template, `# years, # months, # days, # hours, # minutes, #.### seconds`, now says `1 year, 0 months, 0 days, 0 hours, 0 minutes, 0.000 seconds` for a year. Where a trailing zero would read oddly, the template should be chosen by the span's size, which is exactly what `sayTimeRemaining` does, below.

### Plurals, case, and short forms

Every template writes the plural, and the output picks the singular for exactly 1: `# days` says `1 day` and `2 days`. Only a bare 1 is singular, so `1.0 seconds` stays plural, the way English treats decimals. The short plurals, `yrs`, `mos`, `wks`, `hrs`, `mins`, and `secs`, follow the same rule, so `# yrs # mos` says `1 yr 1 mo` the way a profile does. A single letter like `d` never changes. Units can be written in lowercase, title case, or all capitals and keep it, so `# Days Left` and `# HOURS` work, in the spirit of `sayTick` following its tags' case. Those three spellings are matched exactly, so a mix like `hOURS` is only the tag `h` and then text, and ergo never matches anything case-insensitively, which would bring each language's own rules about case into it.

### The m problem

A bare `m` is the one word that means two units. Feeds use `5m` for minutes and LinkedIn writes months as `mo`, so on its own, `m` is minutes. But tenure is naturally `#y #m #d`, and there `m` can only be months. The rule uses something the design already requires: a template runs from largest to smallest, so an `m` right after years, or right before weeks, days, or hours, can only be months. One template can use both: `#y #m #w #d #h #m` reads its first m as months and its last as minutes.

This was a real tension. A rule that guesses from position is the kind of cleverness the style guide warns against, and the first instinct was to leave `m` as minutes and document it. But `#y #m #d` silently said `1y 1260m 63d`, which is worse. The rule fits in one sentence, rests on an order every template already follows, and touches only the one ambiguous word.

### Negative spans

The first draft refused a negative span, which under ergo's rule would have needed a `has` of its own. The second let it through and put a minus sign in front of everything it wrote, which is simple and predictable. The third added one condition: the minus sign appears only in front of something that isn't zero, since `-0 minutes` reads as nonsense and `-just now` worse. The sign goes in front of words too, so `-about 3 hours ago` tells the caller to choose the template by the sign, like `in # hours` for a time still to come.

### Three shortcuts, from 2013

Reading ergo's ancestor from 2013 turned up three ways of saying time that ergo now has. The first two are a few lines on top of `sayDuration`, so ergo is built from ergo, and the third writes its own three numbers.

`sayTimeRemaining` picks its own template by the span's size, by a rule of significant units: two units while the larger is a single digit, then one. It says `45s`, then `1m 0s` through `9m 59s`, `10m` through `59m`, `1h 0m` through `9h 59m`, `10h` through `71h`, and then `3d`. It stays in hours up to 71, because `71h` is plainer than `2d 23h` for something that finishes this week. A template can't do this on its own, since a template's precision is fixed; `sayTimeRemaining` decides how precise a duration should be at each size, which is the feed's ladder, made once.

`sayTimeRemainingCoarse` rounds the seconds down to a multiple of 5, so a countdown on a page steps instead of flickering, and lets the last five seconds count down one at a time. The idea, and the reason, that a countdown shouldn't be distracting, are that ancestor's.

`sayTimeRace` writes `5'15"223`, minutes, seconds, and milliseconds with prime marks, the minutes taking everything above them. It's ergo's first answer to durations shaped like a clock. Its seconds and milliseconds are padded, which `sayDuration` never does, so it writes the three numbers itself. A first draft borrowed `sayTick` for the padding, reading the rest of the minute as a tick in the first minute of 1970. That was ergo built from ergo, but a span isn't a tick, and a page importing `sayTimeRace` carried `sayTick` and its name lists along. ergo uses ergo where it helps, not everywhere it could, and here a shorter call stack was plainer.

Three of its other ideas stayed behind on purpose: thousands separators in big counts, one-letter weekdays, which nobody agrees on, and `sayFraction`'s rates and percentages, like `#/s` and `#%`. That last one puts the idea of a tag as a pair to work on other quantities, and may come back someday.

### What throws, and what doesn't

Both functions share one stance. A caller's bug in the input throws: a tick that isn't one, a span that isn't a whole number within the integer ceiling, a template that isn't text. Anything wrong inside a template comes out as text that's only wrong, as well as the function can manage. ergo's rule says every refused value has a `has` that asks first: `hasTick` does that for `sayTick`, and the one refusal `sayDuration` has left is exactly `checkInt`, so `hasInt` asks first for spans, which need no predicate of their own. One wrinkle is kept on purpose. In JavaScript and Python, which can hand over a fraction of a millisecond, a say function drops it before it checks, so it takes 1.5, which `hasTick` and `hasInt` refuse. The question to ask first about such a number is the `has` of its whole part: `hasTick` of its floor, or `hasInt` of its truncation. It's the one place a say function accepts more than its `has` does, and it's fine because nothing reads the text back.

`saySize4` takes the same stance without the wrinkle. A fraction of a millisecond is a clock's real precision, but a fraction of a byte is a mistake upstream, so it refuses any count that isn't a whole number zero or more within the ceiling, and `hasInt(n) && n >= 0` asks first. In Rust and Go it takes a u64 and a uint64, which can't hold a negative or a fraction, so the type does the refusing and nothing panics.

## The smoke test

The best test of `sayDuration` was the user's framing: could the people who build popular apps assemble the text they show today with it?

| App | Template | Says |
|---|---|---|
| Instagram | `#w` | `3w` |
| LinkedIn feed | `#mo`, `#yr` | `3mo`, `1yr` |
| LinkedIn tenure | `# yrs # mos` | `1 yr 1 mo`, `4 yrs 10 mos` |
| YouTube | `# days ago` | `1 day ago`, `2 years ago` |
| Facebook | `# mins`, `# hrs` | `1 min`, `5 mins`, `1 hr` |
| Reddit | `# min. ago` | `5 min. ago` |
| Stack Overflow | `asked # years, # months ago` | `asked 3 years, 2 months ago` |
| Google Calendar | `# hr # min` | `1 hr 30 min` |
| Google Maps | `# hr # min`, or `# days # hr` for a day or more | `1 hr 25 min`, `1 day 3 hr` |
| Screen Time | `#h #m` | `2h 35m` |
| Jira | `#w #d #h #m` | `1w 2d 3h 30m` |
| A build log | `Done in #m #s` | `Done in 1m 23s` |

Each is one call whose template reads like the app's own text. A feed picks its template by the post's age, and switches to a date once a post is a day old, which takes a few lines, the way every one of those apps writes it:

```js
if (age < Time.minute) return sayDuration(age, '#s')
if (age < Time.hour) return sayDuration(age, '#m')
if (age < Time.day) return sayDuration(age, '#h')
return sayTick(t, 'Mmm D') // 45s, 5m, 3h, Oct 18
```

That's how ergo says how long ago a tweet was tweeted or a LinkedIn post was posted. A feed's "just now" belongs in those same lines: the app decides how young a post has to be to be just now, says that itself, and calls `sayDuration` only past it. ergo doesn't build the ladder, since every app's rungs are different. The smoke test also settled three earlier worries. Rounding down is what the apps do. Dropping zeros at the start is right for every app that uses words, and an app that also drops a trailing zero, like Maps, picks its template by the span's size, the way `sayTimeRemaining` does. And "and" never appears in their durations.

It also found the gap: durations shaped like a clock. A video's length `12:34`, a call's timer `05:09`, a race `1:02:03`, a countdown. Apps that need a fixed shape use a clock, and that's where the fixed-shape need really lives. `sayTick` on a span almost works, but it writes a needless hour, `0:12:34`, and past a day it silently wraps, so 26 hours says `2:00:00`. `sayTimeRace` now covers a race, in a race's own notation. A video's `12:34` and a call's `05:09` are left out for now, as their own function or as tags inside `sayDuration`, until a caller needs one.

# Tick

A tick is an integer count of milliseconds since the start of 1970, the way `Date.now()` counts, kept between zero and the last millisecond of 9999. Ticks add and subtract with the [Time](/time) constants, and two text forms carry one exactly.

`tickToIso` writes the ISO 8601 form, `2012-08-06T05:17:57.300Z`: always 24 characters, always milliseconds, always UTC with a `Z`. Everything parses it and everyone reads it. `tickToText` writes the short form, `2012aug6.0517.57300`: the year, month, and day, then a dot and the clock, then a dot and the milliseconds into the minute, written only as deep as the moment needs. The start of 1990 is `1990`, the start of February is `1990feb`, and the longest text is 20 characters.

:::tabs key:lang
== JavaScript
<<< ../javascript/ergo.test.js#tick
== Python
<<< ../python/src/zootella/ergo/__init__.py#tick
== Rust
<<< ../rust/examples/tick.rs#tick
== Go
<<< ../go/example_test.go#tick
:::

`textToTick` and `isoToTick` read each form back. They're strict on purpose: a reader accepts exactly the text its writer produces and nothing else, no missing milliseconds, no offset in place of `Z`, no `1990feb1` where the writer would say `1990feb`. That's what makes each form one to one with its tick, so text to tick to text always returns the starting text. A text that fails is refused the way the language refuses: an Error in JavaScript, a ValueError in Python, an `Err` in Rust, and an `error` in Go.

`hasTick` asks whether a number is a tick, an integer from zero through the last millisecond of 9999, and never fails. `checkTick` is the guard: it passes a tick through, or fails the way its language fails, an Error in JavaScript, a ValueError in Python, a panic in Rust and Go. Every function that takes a tick goes through the same guard, because a caller handing one a number that isn't a tick has a bug. A number from outside, like a system clock or a file's modified time, isn't the caller's to vouch for, so it's asked about first: once `hasTick` says yes, nothing in ergo will fail on that tick. In Rust that's the difference between a log line and a crash, since a panic can't be caught the way an exception can.

## Saying a tick

`sayTick` writes a tick for a person to read, in whatever shape a template of tags describes. It's one way, with no reader, so a template can take any shape at all.

:::tabs key:lang
== JavaScript
<<< ../javascript/ergo.test.js#saytick
== Python
<<< ../python/src/zootella/ergo/__init__.py#saytick
== Rust
<<< ../rust/examples/tick.rs#saytick
== Go
<<< ../go/example_test.go#saytick
:::

Uppercase tags are the calendar and lowercase tags are the clock, so in `YYYY-MM-DD hh24:mm` a minute's `mm` among the calendar's capitals, or a month's `MM` among the clock's lowercase, stands out as the mistake it is.

| Tags | Say |
|---|---|
| `YYYY` | the year, always four digits |
| `M` `MM` | the month's number, without and with a leading zero |
| `MMM` `MMMM` | the month's name, short and long, in the case of the tag: `Mmm` is Aug, `MMMM` is AUGUST |
| `D` `DD` | the day of the month, without and with a leading zero |
| `DDD` `DDDD` | the day of the week, in the case of the tag: `Ddd` is Mon |
| `h12` `h24` `hh12` `hh24` | the hour, on the clock each names, with `hh` adding a leading zero |
| `mm` `ss` | the minute and the second, always two digits |
| `t` `tt` `ttt` | tenths, hundredths, and thousandths of a second, rounded down |
| `AA` `aa` `A` `a` | AM or PM, in the tag's case and length: `AA` is PM, `a` is p |

The template is read once, left to right, taking the longest tag at each position, and whatever isn't a tag is written as it is. There's no escaping, so keep words out of a template: say the date with a small one, and put what it writes into the sentence around it. In JavaScript and Python, which can hand over a fraction of a millisecond, `sayTick` drops it, down to the millisecond the moment falls in, so it takes a number like 1.5 that `hasTick` refuses; to ask first about such a number, ask `hasTick` about its floor. `sayTick` writes the tick's fields as they fall in UTC; to show someone their own wall clock, shift the tick by their offset at the very end. The names are English, the same everywhere, and a page that wants its reader's language can hand the tick to the browser's `Intl`.

Each example above is real code from its edition, verified by that edition's own tests: a vitest demonstration, a Python doctest, a Rust example that cargo compiles, and a Go example that `go test` runs. Beyond the shared vectors, a fuzz test drives random ticks through both forms and `sayTick` in all four editions at once, which is how Rust's hand-rolled calendar is measured against the other three platforms' date libraries.

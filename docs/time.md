# Time

Spans of time in milliseconds, the unit of a tick: `Time.second` is 1000, `Time.minute` is 60 of those, and so on through `Time.week`. A tick counts milliseconds since 1970, which is what `Date.now()` returns, so `now + 2 * Time.hour` is a deadline two hours out, in every edition, as the same integer.

:::tabs key:lang
== JavaScript
<<< ../javascript/ergo.test.js#time
== Python
<<< ../python/src/zootella/ergo/__init__.py#time
== Rust
<<< ../rust/examples/time.rs#time
== Go
<<< ../go/example_test.go#time
:::

`Time.year` is 365 and a quarter days, the average over the leap cycle, and `Time.month` is a twelfth of that. Both are exact integers of milliseconds, and both are averages: reach for them for rates and rough spans, and for a calendar when you need the first of next month.

## Saying a span

`sayDuration` says a span for a person to read, like how long something took or how long ago it happened. The template is the sentence it writes, with a `#` wherever a number goes and a unit after each `#`, with any whitespace between them, or none: years, months, weeks, days, hours, minutes, or seconds, or a short form like `yrs`, `mo`, `wks`, `d`, `hr`, `min`, or `s`. Three shortcuts choose their own template: `sayTimeRemaining` says one or two units, whichever matter at the span's size, `sayTimeRemainingCoarse` steps a countdown by five seconds, and `sayTimeRace` says a time to the millisecond, the way a race is timed.

:::tabs key:lang
== JavaScript
<<< ../javascript/ergo.test.js#sayduration
== Python
<<< ../python/src/zootella/ergo/__init__.py#sayduration
== Rust
<<< ../rust/examples/time.rs#sayduration
== Go
<<< ../go/example_test.go#sayduration
:::

The largest unit a template names takes everything above it, so a year said in days is 365, and everything rounds down. Zero units at the start drop away, so `#d #h #m` says `5m` for five minutes, while the smallest unit is the precision the template asks for and always shows, so `#m #s` says `1m 0s` for exactly a minute. A whole word turns singular for exactly 1, so `# days` says `1 day`, and a unit can be written in lowercase, title case, or all capitals, like `days`, `Days`, or `DAYS`. A bare `m` is minutes, the way a feed says `5m`, except right after years or right before weeks, days, or hours, where only months fit, so `#y #m #d` says `1y 2m 3d`.

Each example above is real code from its edition, verified by that edition's own tests: a vitest demonstration, a Python doctest, a Rust example that cargo compiles, and a Go example that `go test` runs.

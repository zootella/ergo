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

Each example above is real code from its edition, verified by that edition's own tests: a vitest demonstration, a Python doctest, a Rust example that cargo compiles, and a Go example that `go test` runs.

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

`textToTick` and `isoToTick` read each form back. They're strict on purpose: a reader accepts exactly the text its writer produces and nothing else, no missing milliseconds, no offset in place of `Z`, no `1990feb1` where the writer would say `1990feb`. That's what makes each form one to one with its tick, so text to tick to text always returns the starting text. A text that fails is refused the way the language refuses: an Error in JavaScript, a ValueError in Python, an `Err` in Rust, and an `error` in Go. A tick outside the range is a bug in the caller, and that fails loudly everywhere.

Each example above is real code from its edition, verified by that edition's own tests: a vitest demonstration, a Python doctest, a Rust example that cargo compiles, and a Go example that `go test` runs. Beyond the shared vectors, a fuzz test drives random ticks through both forms in all four editions at once, which is how Rust's hand-rolled calendar is measured against the other three platforms' date libraries.

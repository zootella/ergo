# Int

An integer in ergo is one every edition holds exactly: a whole number from −(2^53 − 1) through 2^53 − 1, the range JavaScript calls safe. Past it, a double can't tell one integer from the next, so a number that crossed from Rust or Go into a page could quietly change. Ticks, spans of milliseconds, and counts of bytes all live within it.

:::tabs key:lang
== JavaScript
<<< ../javascript/ergo.test.js#int
== Python
<<< ../python/src/zootella/ergo/__init__.py#int
== Rust
<<< ../rust/examples/int.rs#int
== Go
<<< ../go/example_test.go#int
:::

`hasInt` asks whether a value is such an integer, and never fails, so a number from outside can be asked about first. `checkInt` is the guard: it passes an integer through, or fails the way its language fails, an Error in JavaScript, a ValueError in Python, a panic in Rust and Go. In JavaScript and Python, `hasInt` checks the type too, and in Python it remembers that a bool is an int and never says yes to one. `sayDuration` and its shortcuts refuse a span past the ceiling, so `hasInt` is the question to ask before handing one a number from outside. In JavaScript and Python, which can hand over a fraction of a millisecond, they drop it before they check, so they take a number like 1.5 that `hasInt` refuses; to ask first about such a number, ask `hasInt` about its whole part.

Each example above is real code from its edition, verified by that edition's own tests: a vitest demonstration, a Python doctest, a Rust example that cargo compiles, and a Go example that `go test` runs.

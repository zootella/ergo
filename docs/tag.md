# Tag

A universally unique identifier of 21 letters and numerals, like `AgKxru95C7jFp5iPuK9O7`. Letters and numerals only, so a double-click selects the whole thing. It's nanoid's default length and alphabet, minus nanoid's `-` and `_`.

:::tabs key:lang
== JavaScript
<<< ../javascript/ergo.test.js#tag
== Python
<<< ../python/src/zootella/ergo/__init__.py#tag
== Rust
<<< ../rust/examples/tag.rs#tag
== Go
<<< ../go/example_test.go#tag
:::

Three functions and a constant. `Tag` makes a new one from the operating system's random source, with every character equally likely. `hasTag` answers yes or no. `checkTag` is a guard: it passes a tag through, or fails the way its language fails, an Error in JavaScript, a ValueError in Python, a panic in Rust and Go, because something that isn't a tag reaching a guard is a bug in the caller, not an ordinary answer to handle. `tagLength` is 21.

Each example above is real code from its edition, verified by that edition's own tests: a vitest demonstration, a Python doctest, a Rust example that cargo compiles, and a Go example that `go test` runs. A tag is random, so the shared vectors pin `hasTag` and `checkTag`, and a scripted test makes a hundred tags in every edition and checks that a tag from any edition validates in every other.

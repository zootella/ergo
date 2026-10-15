# Size

Numbers of bytes in the binary units that memory and storage are measured in: `Size.kb` is 1024 bytes, `Size.mb` is 1024 of those, and so on through `Size.pb`. They name the `1024 * 1024` that would otherwise sit bare in the code. Binary units, 1024 to each, not the decimal 1000.

:::tabs key:lang
== JavaScript
<<< ../javascript/ergo.test.js#size
== Python
<<< ../python/src/zootella/ergo/__init__.py#size
== Rust
<<< ../rust/examples/size.rs#size
== Go
<<< ../go/example_test.go#size
:::

`saySize4` says a count of bytes in four digits or less, with a unit: `0 bytes`, `1 byte`, `1023 bytes`, `9999 bytes`, then `9 KB`, and up through `8191 TB` at the integer ceiling. The count always rounds down, and the unit only climbs when the digits would reach five, so a mebibyte exactly is `1024 KB`. The labels are the everyday ones, KB and MB, while the arithmetic is in 1024s.

Each example above is real code from its edition, verified by that edition's own tests: a vitest demonstration, a Python doctest, a Rust example that cargo compiles, and a Go example that `go test` runs.

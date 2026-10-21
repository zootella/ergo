# Data

A Data holds bytes. Every language keeps bytes its own way, in arrays and buffers and views with their own rules about copying and size. A Data makes those choices once: it holds the bytes, and it speaks every form they travel in, text, base16, base32, base62, and base64, through methods with plain names. Make one from any form, read it out as any other, and it's the same bytes.

:::tabs key:lang
== JavaScript
<<< ../javascript/ergo.test.js#data
== Python
<<< ../python/src/zootella/ergo/__init__.py#data
== Rust
<<< ../rust/examples/data.rs#data
== Go
<<< ../go/example_test.go#data
:::

Every function in ergo that trades in bytes trades in Data. A hash takes a Data and answers one, so it never decides what text the caller wants next; the caller says, with the next method. Only Data knows the encodings.

| | JavaScript | Python | Rust | Go |
|---|---|---|---|---|
| from text | `Data({text: s})` | `Data(text=s)` | `Data::from_text(s)` | `DataFromText(s)` |
| from base16 | `Data({base16: s})` | `Data(base16=s)` | `Data::from_base16(s)?` | `DataFromBase16(s)` |
| from bytes | `Data({array: a})` | `Data(array=a)` | `Data::from(vec)` | `DataFromArray(b)` |
| random | `Data({random: n})` | `Data(random=n)` | `Data::random(n)` | `RandomData(n)` |
| out | `d.base16()` | `d.base16()` | `d.base16()` | `d.Base16()` |
| part | `d.clipView(i, j)` | `d.clip_view(i, j)` | `d.clip_view(i, j)` | `d.ClipView(i, j)` |

base32, base62, and base64 work the same as base16, both ways, and `text`, `size`, `get`, and `clipCopy` round it out. `clipView` looks at part of a Data without copying it, and `clipCopy` copies the part out, for when the bytes beneath could change.

Each form is exact, both ways. Every reader ends by writing the bytes back out and requiring the text it read, so it accepts only what its writer produces: base16 in lowercase, base32 without padding, base64 with it. Text becomes UTF-8 in Unicode Normalization Form C, so the same characters typed two ways become the same bytes, and reading bytes as text refuses bytes that aren't UTF-8, or aren't in that form. A reader that refuses fails the way its language refuses text: an Error in JavaScript, a ValueError in Python, a `DataError` in Rust, and an `error` in Go.

Why four encodings? base16 is simple, standard, and you can see the bytes. base32 is shorter, double-clickable, and always the same length, good for storing hash values in a database. base62 is shorter still and double-clickable, good for bytes in a query string or a location bar a person might see, and its alphabet puts narrow characters in the spots random data fills most often, so it renders as short as it can. base64 is about as short, standard, but not double-clickable, here for completeness.

## Bin

A Bin holds a set number of bytes, which you add a byte or a Data at a time until it's full. It's how to assemble bytes whose size you know before you start, and refusing to hold more than that catches a mistake in the prediction. `Bin(n)` in JavaScript and Python, `Bin::new(n)` in Rust, and `NewBin(n)` in Go; `add` takes a byte or a Data, except in Rust and Go, where a byte goes in with `add_byte` and `AddByte`; and `data` answers what it holds so far.

Each example above is real code from its edition, verified by that edition's own tests: a vitest demonstration, a Python doctest, a Rust example that cargo compiles, and a Go example that `go test` runs. The shared vectors pin every form against the cross-checks of the original, the JavaScript ergo grew from, a dozen values written in all four encodings, and pin every reader's refusals too: uppercase base16, base32 with padding, base64 without it, and a last character with bits that belong to no byte.

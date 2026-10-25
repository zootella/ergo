# Outline

An outline is a tree with one kind of value: bytes. Each outline has a name, lowercase letters and numerals or blank, a value, a Data that holds no bytes when there's no value, and contents, the outlines inside it. It's like JSON stripped down to bytes, with no strings, numbers, booleans, or nulls to tell apart, which makes it a plain container for cryptographic data, configuration, and small packets on the wire. It has two forms: text, indented lines that read in source code and documentation, and binary, compact bytes for disk and the wire.

:::tabs key:lang
== JavaScript
<<< ../javascript/ergo.test.js#outline
== Python
<<< ../python/src/zootella/ergo/__init__.py#outline
== Rust
<<< ../rust/examples/outline.rs#outline
== Go
<<< ../go/example_test.go#outline
:::

The text form is for people and agents to write by hand, so its reader is lenient. It takes any indent, two spaces, four, or tabs, as long as an outline's contents share one, blank lines, `#` comments at the end of a line or on a line of their own, and contents in any order, keeping the order it read. Each line is a name, a colon, and a value in quoted encoding, which writes bytes that are text as quoted text and the rest as base16, like `"Hello"0d0a`, and reads any mix of the two. The writer is canonical: it sorts every level, by name, then value, then contents, and indents two spaces a level, so whatever text a person wrote comes back from the writer one way.

The binary form is strict. Each outline is its name, its value, and its contents, each after its size as a span, a variable-length integer of one to four bytes, like the varint of Protocol Buffers. The reader takes only what the writer writes: contents sorted at every level, every span in its shortest form, nothing missing and nothing after. So the same information is always the same bytes, and an outline's hash is its identity, the way a BitTorrent infohash is the identity of a torrent's Bencoded dictionary. Order that matters goes into names, like contents named `1`, `2`, and `3`, since sorting puts insertion order aside.

| | JavaScript | Python | Rust | Go |
|---|---|---|---|---|
| make | `Outline(name)` | `Outline(name)` | `Outline::new(name)` | `NewOutline(name)` |
| name | `o.name()`, `o.name(k)` | `o.name()`, `o.set_name(k)` | `o.name()`, `o.set_name(k)` | `o.Name()`, `o.SetName(k)` |
| value | `o.value()`, `o.value(d)` | `o.value()`, `o.set_value(d)` | `o.value()`, `o.set_value(d)` | `o.Value()`, `o.SetValue(d)` |
| add | `o.add(x)` | `o.add(x)` | `o.add(c)`, `o.add_name(k)`, `o.add_value(d)` | `o.Add(c)`, `o.AddName(k)`, `o.AddValue(d)` |
| navigate | `o.n(k)`, `o.m(k)` | `o.n(k)`, `o.m(k)` | `o.n(k)`, `o.m(k)` | `o.N(k)`, `o.M(k)` |
| write | `o.text()`, `o.data()` | `o.text()`, `o.data()` | `o.text()`, `o.data()` | `o.Text()`, `o.Data()` |
| read | `textToOutline(s)` | `text_to_outline(s)` | `text_to_outline(s)?` | `TextToOutline(s)` |

`add` takes an outline you've built, a name for a new one, or a Data for a new unnamed one with that value; Rust and Go have no overloading, so those are three methods. `n` navigates to the first outline with a name and fails if there isn't one, and `m` makes it if it's missing, so a chain like `o.m('basket').m('color')` builds as it goes. `list` answers all the contents with a name, `has` asks whether there are any, `get` answers one by index, and `remove`, `clear`, and `sort` change an outline in place. The writers sort a copy, so writing an outline never reorders the one you hold. The blank name is `''`, and JavaScript and Python lookups also take null and None to mean it. Rust adds `get_mut` and `n_mut`, since changing an outline it holds takes a mutable reference, and `m` answers one.

`dataToQuoted` and `quotedToData` are quoted encoding on its own, and `spanEncode` and `spanDecode` the span. A name from outside is asked about with `hasOutlineName` before it's handed over, and `checkOutlineName` enforces it. A reader that refuses fails the way its language refuses text: an Error in JavaScript, a ValueError in Python, an `OutlineError` in Rust, or a `DataError` from `quoted_to_data`, and an `error` in Go.

Every walk through an outline keeps a stack of its own rather than calling itself for each level down, so no nesting, however deep, can run a language's stack out. The format bounds an outline by its spans: its name, its value, and its contents are each at most 268,435,455 bytes, about 256 MB, and the writer refuses an outline with one past that.

Outlines are meant to be small, well under 8 KB as binary, and one in memory takes some 20 to 40 times the bytes of its binary form, depending on the language. So both readers hold what they read to limits, safe as they come for reading from a peer you don't trust: 4 MiB in all, of binary or of text as UTF-8, 256 levels below the root, and 2 KiB for any one name. The defaults are ceilings far past any outline's intended use, and far below what a system can bear. A program whose outlines run larger passes limits of its own, and any limit it leaves out keeps its default. Each limit refuses as early as it can: the size before reading anything, a name at its span before reading it, and depth the moment the reader goes one level past it. Whatever the limits, a reader still refuses an outline its writer couldn't write.

| | JavaScript | Python | Rust | Go |
|---|---|---|---|---|
| the defaults | `outlineLimits` | `OUTLINE_LIMITS` | `OutlineLimits::default()` | `DefaultOutlineLimits` |
| reading within others | `dataToOutline(d, {size: 64 * Size.mb})` | `data_to_outline(d, OutlineLimits(size=64 * Size.MB))` | `OutlineLimits { size, ..OutlineLimits::default() }.data_to_outline(&d)?` | a copy of `DefaultOutlineLimits` with `Size` raised, then `limits.DataToOutline(d)` |

`textToOutline` takes limits the same way.

Each example above is real code from its edition, verified by that edition's own tests: a vitest demonstration, a Python doctest, a Rust example that cargo compiles, and a Go example that `go test` runs.

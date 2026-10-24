# Text

A string is a type, and text is a string that holds something. `hasString` asks only whether a value is a string at all, blank and whitespace included. `hasText` asks whether a string has at least one character that isn't whitespace, so `"a"` and `" a "` are text, while `""`, `" "`, and a tab and a newline aren't. `trim` takes whitespace off both ends.

:::tabs key:lang
== JavaScript
<<< ../javascript/ergo.test.js#text
== Python
<<< ../python/src/zootella/ergo/__init__.py#text
== Rust
<<< ../rust/examples/text.rs#text
== Go
<<< ../go/example_test.go#text
:::

Whitespace means one thing in every edition: the 25 characters on Unicode's White_Space list. Each language's own trim keeps a list of its own. JavaScript's `s.trim()` differs from Unicode's at exactly two characters: it takes off the byte order mark, U+FEFF, which Unicode doesn't call whitespace, and keeps the next line control, U+0085, which Unicode does. Python's `strip` also takes off the four separator controls, U+001C through U+001F. Rust's `trim` and Go's `TrimSpace` follow Unicode's list already, and are what those two editions use. The byte order mark is the one that matters in practice: it sits invisibly at the start of files that Excel and older Windows tools write, and while the web platform drops it as it decodes text, Node, Python, Rust, and Go keep it when reading a file's bytes as text, and so does a Data. `trim` leaves it where it is, so code that reads files from people takes it off on purpose.

`cut` finds the first tag in text and cuts the text around it, answering whether it found the tag, what's before it, the tag itself, and what's after, so the parts assemble back into the text. `cutLast` cuts at the last tag, and `cut2` finds two tags in order and the middle between them. When the tag isn't there, all of the text is before. A tag is any string of at least one character, so a space, a tab, or a newline can be one, while a blank tag is refused, since it would be found at the start of everything. `cutAfterLast` answers just what comes after the last tag, and fails when the tag isn't there, since it's for text whose shape you already know, like a key with a prefix before an underscore; to ask first, ask `cutLast(s, tag).found`.

`deindent` lets a multiline string be indented with the code around it. The first line's indent comes off every line, deeper lines keep the rest of theirs, a tab at the start of a line is two spaces of indent, whitespace comes off the end of each line, blank lines at the start and end go, and the text ends with one newline. In JavaScript it's a tag for a template literal; in Python, Rust, and Go it takes the text, which suits Rust's and Go's raw strings.

A cut answers a plain object in JavaScript, a `Cut` named tuple in Python, a `Cut` struct borrowing from the text in Rust, and a `CutResult` struct in Go, where a type can't share the function's name. Where an edition can't continue, it fails the way its language fails: an Error in JavaScript, a ValueError in Python, a panic in Rust and Go. `hasString` always answers true in Rust and Go, where the type already says so; it's there so the four editions match.

Each example above is real code from its edition, verified by that edition's own tests: a vitest demonstration, a Python doctest, a Rust example that cargo compiles, and a Go example that `go test` runs.

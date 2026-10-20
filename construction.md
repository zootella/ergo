# construction

How ergo is defined, tested, and shipped in each of its four ecosystems. concept.md says what ergo is; this says how it's built. Findings here are as of October 2026, checked against the live registries and a current set of toolchains: Node 22, Python 3.14 with uv, cargo 1.98, Go 1.26.

## The four parts

| | JavaScript | Python | Rust | Go |
|---|---|---|---|---|
| Registry | npm | PyPI | crates.io | none, see Go below |
| Tool | pnpm | uv | cargo | go |
| Manifest | package.json | pyproject.toml | Cargo.toml | go.mod |
| Name | @zootella/ergo | zootella-ergo | zootella-ergo | github.com/zootella/ergo |
| Install | `npm install @zootella/ergo` | `uv add zootella-ergo` | `cargo add zootella-ergo` | `go get github.com/zootella/ergo` |
| Ship | `npm publish` | `uv publish` | `cargo publish` | a tag like `go/v0.1.2` |
| Native tests | vitest | pytest | `cargo test` | `go test` |
| Comment docs | none built in | none built in | docs.rs, automatic | pkg.go.dev, automatic |

## Package names

The bare name `ergo` is unavailable everywhere it could be:

- npm holds `ergo` as a security placeholder from a package abandoned in 2022. The registry note says they'll probably hand it over on request. That's a support ticket and a wait, and we don't need it: scoped names are fully respectable, so `@zootella/ergo` is the name, and the ticket can go in separately.
- PyPI's `ergo` is an active microservice framework.
- crates.io's `ergo` is an alpha crate last touched in 2018. crates.io doesn't reassign names casually.

Neither PyPI nor crates.io has scopes. The convention for a taken name is to prefix the author, so `zootella-ergo` in both, and both are free. Go has no registry to collide in; the name is the repository path.

## Function names

The protocol's canonical names are the JavaScript edition's, and each edition spells them in its own case, which is what its linter and its readers expect. The words never change; only the case does, and the shape of a namespace. Failing when the edition can't continue is likewise the same idea in each language's own mechanism.

| JavaScript | Python | Rust | Go |
|---|---|---|---|
| `saySize4(n)` | `say_size4(n)` | `say_size4(n)` | `SaySize4(n)` |
| `Size.kb` | `Size.KB` | `size::KB` | `KB` |
| `Time.hour` | `Time.HOUR` | `time::HOUR` | `Hour` |
| `tickToText(t)` | `tick_to_text(t)` | `tick_to_text(t)` | `TickToText(t)` |
| `isoToTick(s)` | `iso_to_tick(s)` | `iso_to_tick(s)` | `ISOToTick(s)` |
| `Tag()` | `tag()` | `tag()` | `Tag()` |
| `checkTag(s)` | `check_tag(s)` | `check_tag(s)` | `CheckTag(s)` |
| `hasTick(t)` | `has_tick(t)` | `has_tick(t)` | `HasTick(t)` |
| `await sha256(d)` | `sha256(d)` | `sha256(d)` | `SHA256(d)` |
| `Sha256()` | `SHA256()` | `Sha256::new()` | `NewSHA256()` |
| `h.update(d)`, `h.digest()` | `h.update(d)`, `h.digest()` | `h.update(&d)`, `h.finalize()` | `h.Update(d)`, `h.Digest()` |
| `Data({text: s})` | `Data(text=s)` | `Data::from_text(s)` | `DataFromText(s)` |
| `Data({base16: s})` | `Data(base16=s)` | `Data::from_base16(s)` | `DataFromBase16(s)` |
| `d.base16()`, `d.clipView(i, j)` | `d.base16()`, `d.clip_view(i, j)` | `d.base16()`, `d.clip_view(i, j)` | `d.Base16()`, `d.ClipView(i, j)` |
| `Bin(n)`, `b.add(d)`, `b.add(byte)` | `Bin(n)`, `b.add(d)`, `b.add(byte)` | `Bin::new(n)`, `b.add(&d)`, `b.add_byte(byte)` | `NewBin(n)`, `b.Add(d)`, `b.AddByte(byte)` |
| `await hashFile({file, size})` | `hash_file(f, size)` | `hash_file(&mut f, size)?` | `HashFile(f, size)` |
| `await hashStream({stream, size})` | `hash_stream(f, size)` | `hash_stream(&mut f, size, \|_\| {})?` | `HashStream(f, size, nil)` |
| `FujiHasher({size})`, `await h.update(d)`, `await h.digest()` | `FujiHasher(size)`, `h.update(d)`, `h.digest()` | `FujiHasher::new(size)`, `h.update(&d)`, `h.finalize()?` | `NewFujiHasher(size)`, `h.Update(d)`, `h.Digest()` |
| `hashProtocol` | `HASH_PROTOCOL` | `HashProtocol::default()` | `DefaultHashProtocol` |
| a caller's bug: throws an `Error` | raises | panics | panics |
| text a reader refuses: throws an `Error` | raises a `ValueError` | returns an `Err` | returns an `error` |

A namespace of constants is a frozen object in JavaScript, a class in Python, a module in Rust the way `std::f64::consts` is, and in Go, which has no namespaces inside a package, the bare names `ergo.KB` and `ergo.Hour` as untyped constants that multiply any integer type. Go capitalizes an initialism whole, so `ISOToTick` where the others say `isoToTick`. The last two rows are the two ways a call fails, and they differ in Rust and Go on purpose: a tick outside the range is a bug in the caller and panics, the way `strings.Repeat` panics on a negative count, while text that isn't a tick's form is an ordinary answer from a reader, which those languages return as a value, the way `str::parse` and `strconv.Atoi` do. The protocol sees both as the call being refused. Every kind of value that fails the first way has a `has` beside it that asks and never fails, and a `check` that enforces, so a caller holding a number from outside, like a system clock, asks first: `hasTick` before `tickToIso`, the way Rust's own `is_char_boundary` comes before slicing a string. In JavaScript the all-at-once form is awaited, since Web Crypto, which `sha1` and `sha256` are, answers with a promise, and `blake3` does too, to match. A hasher, the piece-at-a-time form of a hash, is made the way each language makes a new object. In JavaScript it's a capitalized function that returns a plain object, the way `Tag()` makes a new tag. In Python it's a class, its acronym in capitals the way PEP 8 and the `cryptography` package spell `SHA256`. In Rust it's `new` on a struct, and `finalize` reads the hash without consuming the hasher, the way the blake3 crate's own does, so in every language a hasher can take more after its hash is read. In Go it's a `New` function that returns ergo's own `Hasher`, whose `Update` and `Digest` take and give a Data, and which is also an `io.Writer`, so `io.Copy` and `io.MultiWriter` can feed it raw bytes. The fuji hasher is made the same ways and answers both hashes, a `{tipHash, pieceHash}` object in JavaScript, a `FujiHashes` named tuple in Python, and a `FujiHashes` struct in Rust and Go, and since pieces that don't add up to its size are a fact about the bytes rather than a bug, its digest refuses them the way each language refuses: a thrown Error, a `ValueError`, an `io::Error` from `finalize`, and an `error` from `Digest`. In JavaScript its `update` answers a promise too, since Web Crypto does. A Data is made the way each language makes one from one form of several: an object naming the form in JavaScript, a keyword argument in Python, a `from_` constructor in Rust, and a `DataFrom` function in Go, and a reader of text refuses the way that language's readers do, so `from_base16` returns a `Result` with a `DataError` and `DataFromBase16` returns an error. Rust and Go have no overloading, so a Bin takes a byte through `add_byte` and `AddByte`, beside `add` and `Add` for a Data. Go capitalizes SHA as the acronym it is and writes `Blake3` as the name it is. The all-capital constants in Python and Rust are what those languages expect of a constant: Rust's compiler warns on anything else, and PEP 8 asks the same. quad-style.md's rule against them is its JavaScript reading of the deeper rule in the same guide, follow the environment's case. JavaScript could capitalize a constant too and stays lowercase on purpose, a house choice within the language's own range; concept.md has the note. Each dispatch table is the one place that maps a canonical name to the local one, so adding a function means one vector file, four implementations, and one line in each of the four tables.

## Layout

One repository, github.com/zootella/ergo, holding all four editions:

```
ergo/
	CLAUDE.md  concept.md  construction.md  ideas.md  quad-style.md  README.md  LICENSE
	release.js  package.json       one command that releases all four editions at a new version, run as pnpm release
	.editorconfig  .vscode/        indentation per language, format on save, recommended extensions
	.prettierrc                    the Prettier configuration, for the JavaScript edition, release.js, and the site
	.github/workflows/             test.yml runs all four editions and the suite; publish.yml releases on a version tag; docs.yml deploys the site
	tests/                         the shared json vectors, one file per function family, belonging to no edition
	javascript/                    package.json, ergo.js, driver.js, editions.js, ergo.test.js (the vitest suite that drives all four)
	python/                        pyproject.toml, src/zootella/ergo/__init__.py and driver.py, tests/test_vectors.py
	rust/                          Cargo.toml, src/lib.rs, src/driver.rs, src/bin/driver.rs, tests/vectors.rs, examples/ with one file per page of the site
	go/                            go.mod, ergo.go, ergo_test.go, example_test.go, internal/dispatch/, cmd/driver/
	docs/                          the VitePress site: .vitepress/config.js and theme/, index.md, one page per function family
```

Each edition directory also holds its own README.md, the short one its registry shows, and a copy of LICENSE, because npm, PyPI, crates.io, and pkg.go.dev each look for one inside the package.

Directory names are full words, following the style guide's rule against abbreviations, and the Go directory is `go` regardless, so its module path is `github.com/zootella/ergo/go`. The root belongs to none of the four: founding documents, the shared vectors, the site, and the workflows. Each edition's manifest points back at its own directory where the ecosystem has a field for that, like `repository.directory` in package.json.

## Format and lint

Style is not format; concept.md has the reasoning. quad-style.md governs structure, names, comments, and factoring in every edition, and leaves format to each language's standard formatter. Formatters run on save in the editor and are checked in CI, so an unformatted file fails the build. Linters are kept to each ecosystem's recommended rule set, the one that catches real mistakes, and no opinionated presets. The acceptance test: a mid-career engineer with years in one of the four languages looks at their edition, runs their regular linter against it, and things are fine.

| | Formatter | Linter | CI check |
|---|---|---|---|
| JavaScript | Prettier | ESLint, `recommended` rules only | `prettier --check .` and `eslint .` |
| Python | Ruff | Ruff | `ruff format --check` and `ruff check` |
| Rust | rustfmt | Clippy | `cargo fmt --check` and `cargo clippy` |
| Go | gofmt | `go vet` | `gofmt -l .` prints nothing, and `go vet ./...` |

**JavaScript.** Prettier is the formatter the whole ecosystem recognizes; Biome is the fast newer alternative that does both jobs and is also respectable. Configuration stays minimal, because the point of Prettier is that there's nothing to argue about. The one proposal: `semi: false` and `singleQuote: true`, which is the configuration Vite and Vitest use for their own code, keeps two of quad-style.md's choices, and is as mainstream as the defaults. Everything else is default: two-space indentation, wrapping at 80 columns, trailing commas. ESLint runs with its `recommended` rule set and nothing more. That set does not include `eqeqeq` or `prefer-const`, so quad-style.md's reserved signals, `let` by default and `==` by default, survive; only opinionated presets like Airbnb forbid them.

**Python.** Ruff replaced Black, flake8, and isort as the one tool; `ruff format` produces Black's output. Four-space indentation, 88 columns, PEP 8.

**Rust.** rustfmt and Clippy ship with the toolchain and need no configuration. Four-space indentation, 100 columns.

**Go.** gofmt is mandatory and unconfigurable: tabs, no line length. Of the four it is the friendliest to cold3's layout, since it never wraps a line and it aligns trailing comments on consecutive lines itself. `go vet` ships with the toolchain; staticcheck is the popular third-party addition and can come later.

**What survives from cold3's layout, and what doesn't.** Comment placement and voice survive everywhere, since formatters don't touch comment contents and keep trailing comments on their line. Single quotes, no semicolons, trailing commas, and a short guard like `if (!row) return false` on one line survive in JavaScript. Tabs give way to each language's indentation, except in Go. Vertical alignment across lines gives way, except in Go. Two statements in a one-line block, and `{key: 'value'}` without inner spaces, give way. Where alignment genuinely carries information, like a table of test vectors or the base62 alphabet with its caret diagram, every formatter has a standard escape hatch for one block: `// prettier-ignore`, `# fmt: off` and `# fmt: on`, `#[rustfmt::skip]`. Used sparingly, those are recognized and accepted.

**Editor.** A `.editorconfig` at the root sets each language's indentation and final newlines, which every editor reads before any tool runs. A committed `.vscode/settings.json` turns on format on save and names the formatter per language, which is normal in a multi-language repository.

## Tests

One suite, two layers, and every edition runs it.

**Vectors.** The majority of tests are input and exact expected output, and they live in JSON files that belong to no language. Each edition has a dispatch table, function name to function with JSON arguments decoded, and a reader that loads the vector files and runs every case through it. That reader is the entire content of each edition's native test files, so `vitest`, `pytest`, `cargo test`, and `go test` all run the same vectors and all mean something. Precedent: Google's Wycheproof crypto vectors, the CommonMark spec tests, TOML's conformance suite.

A vector file is a JSON array of cases, one file per function family, and a case looks like this:

```json
{"function": "saySize4", "input": [10000], "output": "9 KB", "note": "five digits of bytes, so the next unit up, rounded down"}
{"function": "Size.kb", "input": [], "output": 1024, "note": "a kibibyte is 1024 bytes; a kilobyte would be 1000"}
```

`function` is the JavaScript edition's name, which is the canonical name in the protocol everywhere; each dispatch table maps it to its own language's spelling. `input` is the argument list. `output` is compared as a JSON value, so an edition's integer and the file's number agree. `throws` means the edition fails loudly in its own way, a thrown Error in JavaScript, a raised exception in Python, a panic in Rust and Go, and the reader checks for exactly that. A constant like `Size.kb` is a function with no arguments, so it's vectorable, and its dotted name is the protocol's: each dispatch table maps it to its own namespace, and the JavaScript driver walks the dots. Numbers stay within JavaScript's safe integer range, ±(2^53 − 1), the one ceiling every edition honors, and a case at the ceiling itself is a good vector. Inputs must be well typed for every edition: a vector handing a number to a function that takes text, or a negative to one that takes a count, can't run where the type system refuses it, so guarding against wrong types is each language's own business, outside the vectors. The readers find the vectors by relative path from their own directory, which is why the Rust crate excludes its reader from the published package.

**Scripted tests.** Helpers with state or sequence, like Bin and Outline, need a script. Scripts are written once, in JavaScript, in the vitest suite. Each edition ships a small driver program in its own language that reads JSON lines on stdin and writes JSON lines on stdout. The driver is the same dispatch table as the vector reader, fed from a pipe instead of a file. The vitest suite talks to the JavaScript edition in-process and spawns the Python, Rust, and Go drivers once per run, and a test body loops over all four editions. Precedent: BoringSSL's ACVP `modulewrapper`, a wrapper binary per implementation driven by one harness over stdin and stdout, and WebDriver.

The protocol is one JSON object per line each way:

```json
{"id": 1, "call": "saySize4", "args": [123456789]}
{"id": 1, "result": "117 MB"}
{"id": 2, "call": "Size.kb", "args": []}
{"id": 2, "result": 1024}
{"id": 3, "call": "saySize4", "args": ["not a count"]}
{"id": 3, "error": "argument 0 must be a count"}
```

The harness matches responses to requests by `id`. An error is how a failure comes back, so a scripted test can expect a rejected call from any edition, and the driver keeps serving. The driver's loop is the top gate of its program in the sense quad-style.md uses: the one broad catch, reporting whatever escaped as the response. Each driver answers `Size.b` as its readiness check, since a constant is a call with no arguments. Where the dispatch table lives: `javascript/driver.js` exports `call` and serves when run as a program; `zootella.ergo.driver` in Python holds `functions` and `call` and runs with `uv run python -m`, inside the edition's own environment, which has its dependency; `rust/src/driver.rs` sits behind the crate's `driver` feature, with `src/bin/driver.rs` the binary; `go/internal/dispatch` serves `go/cmd/driver`. The vitest suite's `editions.js` starts the three foreign drivers and wraps each as an object whose functions return promises, so `await e.saySize4(n)` reads the same for every edition.

**Bytes and handles.** Bytes cross both the vector files and the driver pipe as tagged json, which each dispatch table decodes into a Data, through Data's own base16 reader. `{"base16": "616263"}` is those three bytes, and `{"repeat": "61", "length": 1000000}` is those bytes repeated to that length, which lets a vector name FIPS 180's million letter a's, or one of BLAKE3's official inputs, the bytes 0 through 250 over and over, in a single line. Both forms are strict in every edition: base16 is exactly what Data writes, an even number of lowercase digits and nothing else, and a repeat has a unit of at least one byte and a length that's a count within the integer ceiling. Anything else is refused, and vectors pin the refusals, so no edition reads a malformed value some other way. A result that is a Data comes back in the first form, as its base16, and compares as json like any other output. Things that keep state needed no handle so far, because each answers in one call. In the protocol, `Sha256` takes one argument, a list of pieces, and answers the hash of all of them fed in order, so a vector pins the piece-at-a-time form, splits at block and chunk boundaries included. `hashFile` takes a file's bytes as a Data and answers the tip hash, and `hashStream` takes a stream as a list of pieces, which each driver delivers as the stream's boxes in order, so the stripe-peeking crosses box boundaries wherever a vector puts them. `FujiHasher` takes the size it's told, the pieces to feed it, and a protocol, so a vector can tell it a size the pieces don't add up to and pin the refusal. A third tagged form, `{"mulberry": seed, "length": n}`, is n bytes from icarus's Mulberry32 generator, wrapping at 32 bits in every edition, so a vector names one of icarus's deterministic test files in a line; it matches icarus's own generator for any file under about 19.7 MB, past which icarus's seed outgrows the integers a double holds exactly. `Data` takes the form to make one from and a method with its arguments, like `[{"text": "ABC"}, "base16"]` or `[{"base16": "414243"}, "clipView", 1, 3]`, and `Bin` takes a capacity, a list of bytes and Data to add, and a method to answer with. Objects like Outline, which a test builds and navigates over many calls, will need a handle table in each driver: construct and return a handle, call a method on a handle. That arrives with the first function that needs it.

**What runs where.** Natively, each edition runs the vectors. The vitest suite is the only place everything runs: vectors and scripts, against all four editions at once. Demonstration tests that teach the API live in the vitest suite and are a source for the documentation site. A round trip is a pair of vectors side by side, one each way.

**Fuzz.** The unit suite is deterministic and fast: every case is pinned data, nothing depends on random input, and a run takes tens of milliseconds per edition, so it runs on every save and every push and means the same thing each time. Fuzz tests are the opposite on purpose. A fuzz test draws random inputs as fast as the workstation allows for a set number of seconds, usually one or four, and checks a property that must hold for all of them, like a tick round-tripping through both text forms and all four editions agreeing on every one. That's where Rust's hand-rolled calendar gets measured against JavaScript's Date, Python's datetime, and Go's time across millions of ticks. Fuzz tests live in the vitest suite beside the scripted tests, since they need the drivers, but they're skipped unless `ERGO_FUZZ` names a number of seconds: `ERGO_FUZZ=4 pnpm fuzz`, which runs only them and shows the count they reached, since vitest hides the output of passing tests by default. cold3 kept its fuzz loops behind `noop` for the same reason; here the environment variable is the switch. The centers and radii come from cold3: a broad sweep of decades, leap day, each unit boundary at a narrow and then a wide aperture, and the two edges of the range.

The cross-edition fuzz is about uniformity. It pays a trip over the pipes for every value, so it reaches tens of thousands of ticks in a few seconds, and that count measures the harness, not the functions. The fuzz in each language is about performance: the same loop, the same centers, written in each language inside its own test runner, skipped unless `ERGO_FUZZ` names the seconds, and printing its count when it finishes. That's the one exception to the rule that Python, Rust, and Go carry no hand-written tests beyond the reader and the driver, and it's the one place the four editions' speeds can be compared.

| | Where | Run |
|---|---|---|
| JavaScript | a second test in the vitest fuzz block, in-process | `ERGO_FUZZ=4 pnpm fuzz` |
| Python | `tests/test_fuzz.py` | `ERGO_FUZZ=4 uv run pytest -s -k fuzz` |
| Rust | `tests/fuzz.rs` | `ERGO_FUZZ=4 cargo test --release --test fuzz -- --nocapture` |
| Go | `fuzz_test.go` | `ERGO_FUZZ=4 go test -run Fuzz -v` |

The extra flags are each runner's way of showing a passing test's output, and Rust's `--release` measures the edition as it ships. Beside the tick loop, each edition has a hot loop making tags, in the spirit of a smoke test: making a tag is mostly a call to the operating system's random source, so an edition that asks for randomness more often than it needs to shows up at once as the slow one. And a hash loop hashes one mebibyte of random bytes again and again with each of the three hashes, checks once that the piece-at-a-time form agrees, and prints each one's speed. The cross-edition fuzz has a hash test of its own, random bytes of random lengths cut into random pieces, through all three hashes in both forms in every edition.

The two layers differ by about two orders of magnitude, and that gap is the design. On one ordinary machine in October 2026, four seconds of the tick round trip reached about 1,160,000 cycles a second in Rust, 775,000 in Go, 480,000 in JavaScript, and 110,000 in Python, while the cross-edition fuzz reached 7,600, each cycle a round trip over the pipes to three drivers. The hash loops on the same machine, in mebibytes a second for SHA-1, SHA-256, and BLAKE3: Go 2,500, 2,500, and 2,800; Rust 2,300, 2,300, and 1,900; Python 2,500, 2,400, and 1,900; JavaScript 2,200, 1,600, and 50, Web Crypto for the first two and noble's plain JavaScript for BLAKE3, which is the price of a hash the platform doesn't have; noble's SHA-1 and SHA-256, which the hashers use, ran about 310 and 200. Every number is one core's but Go's BLAKE3: lukechampine.com/blake3 hashes a large input's subtrees on separate goroutines of its own, and held to one core with `GOMAXPROCS=1` it does about 600, while Rust's crate and Python's binding, using the same NEON code on that machine, stay on one. On a processor with SHA instructions, like that one, SHA-1 and SHA-256 outrun BLAKE3 on a single core; BLAKE3's speed is in software, and in its tree. The cross-edition hash fuzz reached about 375 random inputs a second. The tight loops are for speed and vigor; the slow one is the only test that puts the same random value through all four editions and requires the same answer.

**The design pressure.** Vectors are cheaper than scripts in every way: no process, no handles, native in all four runners. So each new helper is asked whether its hard core can be a pure function with a vector, with the script layer reserved for the thin wrapper around it. The per-function cost of four editions is one dispatch entry in four places, however the tests are built.

## JavaScript, on npm

The manifest is package.json. The fields that signal a well-made library:

- `"type": "module"` and ESM only. Node 22 can `require()` an ESM module natively, so dual builds are no longer necessary, and skipping them means no build step: the file you edit is the file people import.
- `"exports": {".": "./ergo.js"}` rather than a bare `main`. This controls the public surface and lets subpaths like `@zootella/ergo/crypto` be added later without breaking anyone.
- `"files"` as a whitelist of what goes in the tarball.
- `"sideEffects": false`, so bundlers can drop unused functions. This is only true once tests are out of the source file, and it has a second half inside the file: every top-level statement has to be something a bundler can prove harmless. A function declaration, a regex, an array, or a frozen object literal is. Building an object property by property and then freezing it is not, and Rollup kept the whole Time namespace in a bundle that imported only `saySize4` until the namespaces became literals. The `/*#__PURE__*/` annotation on the `Object.freeze` call is the standard mark that esbuild and webpack need and Rollup doesn't, and the values inside have to be plain numbers: esbuild folds `1024 ** 2` but not `60 * 1000`, and won't drop a literal holding an expression it didn't fold, since a multiplication could in principle call a getter. With both, a page that imports one function carries that function and nothing else, which was checked by bundling a one-function import with Rollup and with esbuild and reading what survived. The rule for every future addition: no top-level statement that does anything, only declarations, with the numbers written out and the vectors pinning them.
- `"engines": {"node": ">=22"}`. Node 20 reached end of life in April 2026.
- `license`, `repository`, `description`, and `keywords`. If the repository holds four languages, `repository.directory` points at the JavaScript one.
- One dependency, `@noble/hashes`, for what Web Crypto can't do: SHA-1 and SHA-256 a piece at a time, and BLAKE3. `sha1` and `sha256` all at once are Web Crypto, at native speed and from a promise, so `blake3` returns a promise too; the hash section of ergo.js says why, and the shared vectors and the cross-edition fuzz test hold noble and Web Crypto alike to the other editions' answers. noble's `sideEffects: false` keeps the guarantee above: bundling a page that imports `saySize4` alone carries that function and nothing else, one that imports `sha256` alone carries Data, which every hash takes and answers, and a call to Web Crypto, about 2 KB gzipped and nothing of noble's, and one that imports `blake3` alone carries Data and noble's BLAKE3 and nothing else, about 6 KB, checked with Vite. The `devDependencies` are vitest, eslint, and prettier, and nothing of theirs reaches the package.

Layout is flat: package.json, README.md, LICENSE, ergo.js, and the vitest suite beside it. The `src` and `dist` convention is for compiled libraries.

Everything core.js touches is a global in both Node and browsers now: `crypto.subtle`, `TextEncoder`, `atob`, `ReadableStream`, `performance`. No polyfills and no environment checks. This is the strict bar for the whole project: the JavaScript edition runs completely in a browser tab and a web worker, on the platform and plain JavaScript, with no Node and no WebAssembly. Where the platform falls short, as Web Crypto does for hashing, which it offers only all at once and without BLAKE3, the gap is filled by a library that meets that bar.

The vitest suite holds the vector reader for the JavaScript edition, every scripted test, and the code that spawns and talks to the three foreign drivers. Named tests and `expect` assertions, sections mirroring the source. Demonstration and fuzz tests that cold3 kept behind `noop` become `test.skip` or run only when an environment variable asks for them. vitest's browser mode can later run the vector suite in a real Chrome, which is the honest proof that the platform calls behave identically on both sides.

Types: a hand-written `ergo.d.ts` beside the code, listed in `files` and `exports`. This leaves the JavaScript untouched by JSDoc annotations and gives editors completion. After the first publish, not before.

The tool is pnpm, which develops most of the large libraries on npm, including Vite, Vitest, and VitePress; the published tarball comes from package.json's `files` field and is the same whichever tool runs the publish, so consumers never see the choice. `packageManager` in package.json pins the version, and the workflows read it.

Local development: `pnpm link ../../ergo/javascript` in a consuming project, or a `link:` spec in its package.json, so the working tree is used directly. Before publishing, `pnpm pack --dry-run` lists exactly what will ship.

Shipping: `npm publish --access public`, from the publish workflow; scoped packages default to private, so the flag is required. Trusted publishing through GitHub's OIDC removes the need for a stored token, and npm attaches a signed provenance statement to every version published that way, from 0.1.1 on. npm processes a new version before serving it, which took from seconds to about six minutes for 0.1.1 and 0.1.2, so a version can be missing from the registry for a few minutes after a successful run. release.js sets the version here along with the other editions', as described under Versions and releases.

## Python, on PyPI

The manifest is pyproject.toml with a `[project]` table: name, version, description, readme, license, `requires-python = ">=3.12"`, and a `[build-system]` naming the backend. hatchling is the widely recognized lightweight backend; uv's own `uv_build` also works.

Distribution name and import name are separate things in Python. The parallel to a scoped npm package is a namespace package: the distribution is `zootella-ergo`, and code says `from zootella.ergo import base62` or `import zootella.ergo as ergo`. This is how Google and Azure ship their libraries. The layout is `src/zootella/ergo/` with the code in one module file. The `src/zootella/` directory deliberately has no `__init__.py`; that's what makes it a namespace other `zootella-*` packages could share.

The `tests/` directory holds the vector reader under pytest, declared as a dev dependency group, and nothing else. pytest is configured in pyproject.toml to also run doctests in `src`, which is where the Python examples live, and to put `src` on the path so the reader imports the package without an install. The driver is `zootella.ergo.driver`, run with `python -m`; it holds the dispatch table too, and the reader imports it from there.

When the edition can't continue it raises Python's own exceptions, a ValueError or a TypeError, never a custom type, and the vector reader expects any exception for a throws case, since which one is the edition's business. Ruff's default rule set is broader than it once was and includes flake8-bugbear and blind-except, so the two deliberately blind spots, `except Exception` at the driver's top gate and `pytest.raises(Exception)` in the reader, each carry a `# noqa` with the reason beside it. A namespace of constants is a class with upper-case attributes, `Size.KB` and `Time.HOUR`, the familiar shape for a group of named numbers.

The standard library covers SHA-1, SHA-256, HMAC, PBKDF2, and secure random through `hashlib`, `hmac`, and `secrets`, plus `base64`, `struct`, and `json`. It has no AES, no RSA, and no elliptic curves. Those come from the `cryptography` package, which is the universally used and trusted choice in Python and meets the dependency standard in concept.md. It has no BLAKE3 either, which CPython declined to add in 2022, so BLAKE3 comes from the `blake3` package, the official Python binding to the official Rust implementation, and the encodings and Normalization Form C are all the standard library's, with prebuilt wheels for every common platform, Python 3.14 included.

Local development: `uv add --editable ../ergo/python` in a consuming project.

Shipping: `uv build` produces a source tarball and a wheel in `dist/`, then `uv publish` uploads to PyPI. PyPI's trusted publishing from GitHub Actions is the standard, tokenless path.

## Rust, on crates.io

The manifest is Cargo.toml with a `[package]` table: name, version, `edition = "2024"`, license, description, repository, keywords, and categories. The crate name has a hyphen, `zootella-ergo`, and code refers to it with an underscore, `use zootella_ergo::base62`.

A library crate is `src/lib.rs`, and one long file is normal. The `tests/` directory holds the vector reader as an integration test, and nothing else. The dispatch table is `src/driver.rs`, a hidden module behind a cargo feature named `driver` that brings in `serde_json`, and the driver binary is `src/bin/driver.rs` with `required-features = ["driver"]`, so the library itself carries no JSON dependency. The crate lists itself as a dev-dependency with that feature on, which is the established way to turn a feature on for tests alone, so plain `cargo test` builds the dispatch table. The reader reads the shared vectors by relative path, which a published crate doesn't have, so `tests/` is excluded from the package. When the edition can't continue it panics, documented under a `# Panics` heading, and the dispatch table turns a panic into an error with `catch_unwind` so the driver survives it. A namespace of constants is a module, `size::KB` and `time::HOUR`, the way `std::f64::consts` does it, with sizes as `u64` like a file's length and spans as `i64` like a tick. A reader of text returns a `Result` with one small error type, `ParseError`, that carries the refused text, which is what `str::parse` taught every Rust engineer to expect. The standard library has no calendar, only `SystemTime`, so the twenty lines of arithmetic between a count of days and a year, month, and day live in the crate, and the shared vectors and the fuzz test pin them against the other three editions' platforms. It has no stable random source either, so `tag` draws its bytes through the `getrandom` crate, the one the ecosystem's randomness rests on. The three hashes come from RustCrypto's `sha1` and `sha2` and from the `blake3` crate, the official implementation by BLAKE3's authors, and text in Normalization Form C from `unicode-normalization`, the crate the ecosystem uses for it. The standard library has no base16, base32, or base64 either, and the few lines of each are in the crate, pinned by the vectors, rather than three more dependencies. The blake3 crate builds its SIMD code with the system's C compiler through `cc`, as most crates with fast native code do. Every crate in the lockfile that declares a minimum Rust asks for 1.85 or older, blake3 declaring none, so this crate's 1.85 holds, though sha2 0.11.1 asks for 1.88: the lockfile holds 0.11.0, and a project on 1.85 through 1.87 gets 0.11.0 too from Cargo's MSRV-aware resolver, the default for edition 2024 projects, while one on the older resolver can pin it with `cargo update -p sha2 --precise 0.11.0`. Doc comments written with `///` are markdown, and code examples inside them compile and run as tests; docs.rs renders them automatically. The longer examples that the site shows live in `examples/`, which `cargo test` compiles.

The standard library has no cryptographic hashing, no ciphers, and no key exchange: not even SHA-256. The RustCrypto crates (`sha2`, `hmac`, `pbkdf2`, `aes-gcm`, `rsa`, `p256`, `getrandom`) are the audited, universal answer, already in most experienced Rust developers' dependency trees, and the Rust edition uses them. A cargo feature flag that makes them optional is idiomatic and cheap, and can be added if anyone wants the pure functions alone.

Local development: a path dependency in the consuming project's Cargo.toml, `zootella-ergo = { path = "../ergo/rust" }`.

Shipping: `cargo login` once, `cargo package` to inspect the tarball, `cargo publish` to upload. docs.rs builds and hosts the documentation automatically on every publish. crates.io supports trusted publishing from GitHub Actions.

## Go, as a module

There is no registry and no publish command. The import path is the repository URL. go.mod declares `module github.com/zootella/ergo/go` and a minimum `go` version. A release is a git tag on a commit that contains go.mod, and because the module lives in the `go/` subdirectory, the tag carries that directory as a prefix: `go/v0.1.0`, not `v0.1.0`. That's the rule for every module below a repository's root, and a bare `v0.1.0` tag would be invisible to the Go tooling. The first time anyone fetches that version, proxy.golang.org mirrors it permanently and pkg.go.dev builds the documentation from the doc comments. Running `go list -m github.com/zootella/ergo@v0.1.0` with `GOPROXY=proxy.golang.org` triggers that first fetch on demand.

Layout is go.mod, ergo.go, ergo_test.go, and example_test.go at the module root, with `package ergo`. The test file holds the vector reader under the built-in `go test`, and nothing else; the example file holds `Example` functions that `go test` runs and the site shows. The dispatch table is `internal/dispatch`, importable by this module and no one else, and the driver is a `main` package in `cmd/driver/`, the Go convention for a module's executables. The test files are the external `ergo_test` package, which is normal and here necessary, since they import both the library and the dispatch package and an internal test package would make a cycle. When the edition can't continue it panics, the way `Must` functions do, and the dispatch table recovers a panic into an error. Constants are flat and untyped, `ergo.KB` and `ergo.Hour`, since a Go package has no namespaces inside it and an untyped constant multiplies an int, an int64, or a uint64 alike. Two direct dependencies. `lukechampine.com/blake3`, since neither the standard library nor x/crypto has BLAKE3, and through it `klauspost/cpuid`, which it asks about the processor; it's the implementation written with x/crypto in mind, with assembly for amd64 and portable Go elsewhere, and it hashes the subtrees of a large input on goroutines of its own. And `golang.org/x/text`, for text in Normalization Form C, which the standard library leaves out. Nothing more for tests. The `go` directive is 1.26, the current release, which x/text's own latest asks for; since 1.24, `crypto/rand.Read` is guaranteed never to fail, so `Tag` and `RandomData` read it without a check.

If the Go code lives in a `go/` subdirectory of a shared repository, the module path becomes `github.com/zootella/ergo/go`. That's established practice; Apache Arrow shipped its Go module that way for years. But it's set by where go.mod sits, so it has to be decided before the first tag.

Two quirks worth knowing. Major version 2 and beyond must carry a `/v2` suffix in the module path, so staying on v0 and then v1 avoids it. And module versions are immutable once the proxy has them; a bad tag is fixed by a new tag, never by moving the old one.

The standard library covers everything core.js uses: `crypto/sha1`, `crypto/sha256`, `crypto/hmac`, `crypto/pbkdf2`, `crypto/aes` with `crypto/cipher` for GCM, `crypto/rsa`, `crypto/ecdsa`, `crypto/ecdh`, `crypto/rand`, and `encoding/base32`, `base64`, `hex`, and `json`.

Local development: a `replace github.com/zootella/ergo => ../ergo/go` directive in the consuming project's go.mod, or a go.work file.

## Dependencies

The standard, from concept.md: each edition uses its platform first, and where that falls short, libraries that are universally used and trusted in that ecosystem, and the JavaScript edition runs completely in a browser tab and a web worker, with no Node and no WebAssembly. Not "is it bundled" but "do experienced people already have it in most of their projects."

| | Pure functions | Text in NFC | SHA-1, SHA-256, BLAKE3 | HMAC, PBKDF2, random | AES-GCM, RSA, curves |
|---|---|---|---|---|---|
| JavaScript | platform | platform | platform (Web Crypto) all at once, noble a piece at a time and for BLAKE3 | platform (Web Crypto) | platform (Web Crypto) |
| Go | platform | `golang.org/x/text` | platform, and `lukechampine.com/blake3` | platform | platform |
| Python | platform | platform | platform, and `blake3` | platform | `cryptography` |
| Rust | platform | `unicode-normalization` | RustCrypto, and `blake3` | RustCrypto | RustCrypto |

The pure functions are most of the file: base62 and the other encodings, cut, deindent, the tick text format, Outline, span, quoted encoding, the integer helpers. TOTP and hashcash sit in the middle two columns, since they need only hashing and HMAC, and hash only small inputs, which Web Crypto does well all at once.

## Documentation

One site, not four, and the registries send readers to it. The README on npm, PyPI, crates.io, and GitHub stays short: what ergo is, how to install this edition, and a link.

**The generator is VitePress.** It's markdown first, minimal configuration, from the same family as Vite and Vitest, and it's what Vitest's own documentation is built with. Synced language tabs, where picking Rust on one block switches every block on the page and remembers the choice, come from `vitepress-plugin-tabs`, maintained by a Vite core team member. MkDocs Material is the one alternative that's arguably stronger on documentation features alone, with linked tabs and snippets built in, but it's Python tooling in a JavaScript-led repository. Docusaurus and Starlight both do the job too; neither offers anything the choice above lacks.

**Examples come from source, per language.** A Rust tab needs real Rust, and the test suite holds none, since tests are JavaScript and vectors are data. Each of the four ecosystems already has a native form for executable examples, and that's where ergo's examples live:

- JavaScript: a region inside a demonstration test in the vitest suite.
- Python: a doctest in the module docstring, run by pytest with `--doctest-modules`. The region markers are prose lines in the docstring, and the closing marker needs a blank line before it, because doctest reads expected output until a blank line.
- Rust: a file in `examples/`, which `cargo test` compiles and `cargo run --example` runs. The one-line examples in `///` doc comments stay for docs.rs.
- Go: an `Example` function in `example_test.go`, which `go test` runs against its `// Output:` comment and pkg.go.dev displays.

VitePress pulls them in with its built-in snippet import: a line like `<<< ../go/example_test.go#size` drops the region between `// #region size` and `// #endregion size` markers into the page, matching markers after trimming, so an indented region inside a function is fine. The site builds from source with no copying, and an example that stops compiling fails its own edition's tests. The tabs plugin renders the chosen tab on the server and the rest on the client. Tests of correctness remain one suite in one language; examples are documentation and exist in four languages because they're shown in four.

**Reference pages are a later step.** The Javadoc lineage split into per-language reference generators, like rustdoc on docs.rs, godoc on pkg.go.dev, mkdocstrings, and TypeDoc, and docs-as-code guide sites, like VitePress. No reference generator spans four languages. Linking out to four generated references would give four styles and break the one-site idea, so the plan is a small extractor of our own in the godoc spirit: read each edition's source, take each declaration and the comment on or above it, and emit one markdown page per function with four tabs for VitePress to render. Go's doc comment convention is already the style guide's, a plain prose comment above the declaration with no tags, which is what makes this extractor small. docs.rs and pkg.go.dev will still generate their own reference pages automatically, a free bonus rather than the documentation.

**Hosting.** A GitHub Actions workflow builds the site on every push to main and deploys it to GitHub Pages; VitePress ships a reference workflow for this. Pages has to be turned on once in the repository's settings, with GitHub Actions as the source, or the deploy job fails. For now the site is a placeholder at zootella.github.io/ergo, with `base` set to `/ergo/` in the VitePress config and no CNAME file, so it's reachable before any DNS exists. The destination is zootella.com itself, apex domain and root route, as this repository's custom domain, and moving there is two edits once the DNS below is in place: set `base` back to `/` and create `docs/public/CNAME` holding `zootella.com`, which VitePress copies into the build and GitHub Pages reads as the custom domain. GitHub then redirects the github.io path to the domain. The root zootella.github.io stays empty, since GitHub serves a user's root site only from a repository named exactly that, and nothing needs it. An apex can't be a CNAME, so at the DNS provider it takes four A records to GitHub Pages' addresses, 185.199.108.153 through 185.199.111.153, and four AAAA records to the matching IPv6 addresses, which GitHub's Pages documentation lists. Leave the records unproxied, at least until GitHub has provisioned the certificate and "Enforce HTTPS" is on in the repository's Pages settings; a proxy in front afterward is optional. A `www` CNAME to `zootella.github.io` makes www redirect to the apex. Verifying the domain under the GitHub account's Pages settings prevents anyone else's repository from claiming it. Only one repository can serve a given domain, so zootella.com belongs to this one.

## Versions and releases

Semantic versioning in all four. Start at 0.1.0; the zero major tells users the API may still move. While it's zero, a release that only adds bumps the patch, and one that changes an answer ergo already gave bumps the minor, so a consumer that asks for `^0.1.0` in package.json or `"0.1"` in Cargo.toml takes every addition and never a changed answer. The version lives in package.json, pyproject.toml, and Cargo.toml, and in the two lockfiles that record the project's own version beside its dependencies, uv.lock and Cargo.lock; for Go it lives only in the git tag. All four editions carry the same number for the same release, since the point of ergo is that the four editions are the same thing.

A release is one command at the root: `pnpm release patch`, or `minor`, or an exact version like `0.3.0`, which runs release.js. It checks that main is clean and up to date, pushes it if GitHub hasn't seen it, and waits for GitHub's tests to pass on it, then sets the version in the five files the way each ecosystem writes it, commits that as `ergo 0.1.1`, and pushes the commit together with the tag `v0.1.1`. The tag starts `.github/workflows/publish.yml`, which runs the whole test workflow on the tagged commit, checks that the tag's version matches the three manifests, and publishes to npm, PyPI, and crates.io by trusted publishing through GitHub's OIDC, so there are no long-lived tokens anywhere. It then pushes the Go module's tag, `go/v0.1.1`, on the same commit and asks the Go proxy to fetch it, so Go, like the other three, ships only after the tests pass; a tag pushed with the workflow's own token starts no other workflow. Last, it makes a GitHub release that lists the commits since the previous one and links the version on all four registries. release.js watches the run and ends with the release's address, a couple of minutes after it started. Every step of the workflow first asks whether its part is already done, so a run that fails partway is finished by rerunning it.

Each registry was told once, on its own site, to trust that workflow file in zootella/ergo under the environment named `publish`, so the file and the environment keep those names. npm adds a condition the other two don't: since October 2026, a new trusted publisher has to complete its first successful publish within two days, or it expires and has to be deleted and made again. The first publish binds it to the repository's permanent identity, and after that it doesn't expire. ergo's first npm configuration expired that way, because 0.1.0 went to npm by hand, so the workflow skipped npm and never validated it; the second was validated by publishing 0.1.1 on October 18. Whenever a trusted publisher is made, the next release should run soon after. On npm's side it allows `npm publish` only, not dist-tag, since the workflow never moves a tag. release.js lives at the root because a release belongs to no one edition, and it imports only Node's own modules. The root's package.json is private, depends on nothing, and holds only the script that names the command; the verb is `release` rather than `patch` because `pnpm patch` is a command of pnpm's own, which pnpm runs ahead of any script. It's formatted by the same Prettier configuration as the JavaScript edition, which sits at the root for that reason, and CI checks it.

## Open

- Bytes in the vector files and the driver protocol, and handles for objects, as described under Tests. Both arrive with the first function that needs them.

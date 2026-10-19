# ergo

Ergo is short for ergonomic. It's a small library of helper functions that's easy to reach for: few dependencies, and only ones its ecosystem already trusts, available the normal way in each language's module system, and the same in every language it ships in.

## Where it came from

cold3 has a file called core.js: several thousand lines of helper functions built up over years, with tests inline beside the code. Some are everyday, like cut and deindent. Some are more fun: a base62 encoding tuned for double-clicking and narrow rendering, a reversible readable tick format like 2012aug6.0517.57300, TOTP in a screenful on top of the platform's HMAC, and Outline, a tree of names and bytes with a text form and a binary form.

The first idea was simple. Copy core.js here, make it a proper npm module, and use it from every JavaScript and Node project instead of copying files around. That part is straightforward: port core.js and the pieces of level0.js it leans on into one standalone module, move the tests out of the source file into vitest files alongside so that importing the library never runs a suite and failures report names and diffs, and publish as @zootella/ergo.

## The realization

Then: you're not always in JavaScript. You're in Rust, writing a Tauri app. You're in Python, writing a script for a web server. And you want base62. The same base62, with the same alphabet and the same answers.

So ergo ships in four languages at once: JavaScript, Python, Rust, and Go. Each is a normal module in its own ecosystem, on npm, PyPI, crates.io, and Go modules. The functions have the same names and do the same things. You can work in any one language, or any two, and the parts on each side are compatible: a Rust backend and a JavaScript page agree on every tag, hash, and encoding.

## One library, four natives

Two things are true of ergo at once, and they pull in different directions.

The first is sameness. The four editions are one library. The functions have the same names, do the same things, and give the same answers, so a tag made in Go validates in Python, and text encoded to base62 in Rust decodes on a web page. This is enforced rather than promised: the shared test suite is the specification, and every edition is measured against it.

The second is nativeness. Each edition has to pass as a normal, well-made library in its own ecosystem, with no accent. The test is four mid-career software engineers, each with years of experience in one of the four languages. Each opens their edition, runs the linter they always run, and finds nothing foreign.

The two are reconciled by deciding what each one owns. Sameness owns behavior, the words in the names, and the values in quad-style.md: how code is structured, factored, and commented, errors as values inside the code we control, guards that fail loudly at trust boundaries. Nativeness owns everything the ecosystem has an opinion about:

- Format, which is the language's standard formatter, on save and in CI.
- The spelling of names, which takes the language's case: saySize4, say_size4, say_size4, SaySize4, and Size.kb, Size.KB, size::KB, ergo.KB.
- Package layout and the manifest, which follow each registry's conventions.
- The test runner and the form of executable examples, even though the tests themselves are shared.
- How an edition fails when it can't continue: a thrown Error in JavaScript, a raised exception in Python, a panic in Rust and Go. The suite asks only whether it refused.
- Which dependencies count as free, judged by what that ecosystem already trusts.

The four editions are the same author writing fluently in four languages, not one text run through a translator. The meaning and the character are identical. The idiom is whatever a native would write. This is the core difficulty of ergo and its central triumph: one hand and one foot in each of the four, so a coder finds the same tools in every place, and every place looks correct and normal by that language's own rules, environment, patterns, and conventions. The sections below are this principle applied: to dependencies, to tests, and to the line between style and format.

## What a dependency has to be

Every edition uses its platform first, and where the platform falls short, a library that experienced people in that ecosystem already have in most of their projects. The test isn't whether something is bundled. It's whether it's universally used and trusted, a no-brainer rather than a new target of scrutiny.

The JavaScript edition has the strictest bar. It runs completely in a browser tab and a web worker, so it never leans on Node, and its dependencies are plain JavaScript, never WebAssembly, which is harder to say is audited. Most of core.js lives on the platform alone: Web Crypto, TextEncoder, ReadableStream, all globals. Hashing is where the platform first fell short. Web Crypto has SHA-1 and SHA-256, but only all at once, from a promise, and no BLAKE3, so a page can't hash a large file through it without holding all of the file in memory at once, and that limit is what icarus's Fuji hashes were designed around, with their 4 MiB pieces. So the JavaScript edition takes SHA-1 and SHA-256 all at once from Web Crypto, at its full speed, and noble's hashes only for what Web Crypto can't do: those two a piece at a time, and BLAKE3. noble is plain JavaScript with no dependencies of its own, its SHA-256 independently audited at version 1.0.0, in 2022, and all three tested against the official vectors. A library already in the dependencies is never a reason to use it for more than the platform lacks.

Go's standard library covers everything but BLAKE3, which comes from lukechampine.com/blake3. Python's covers SHA-1, SHA-256, HMAC, and PBKDF2; BLAKE3 comes from the official blake3 package, and the `cryptography` package covers the rest. Rust's standard library has no crypto at all, so the Rust edition brings in the RustCrypto crates, and the blake3 crate, BLAKE3's official implementation.

## One test suite

Four implementations, but one set of tests.

Most tests of functions like these are data. Here is the input, here is the exact output. A text that encodes to this base62. A tick that renders as this string and parses back. A secret and a time that produce this TOTP code. That data doesn't belong to any language, so it lives in JSON files of test vectors. Each edition has a thin reader, a dispatch table from function name to function, that loads the vectors and checks every answer. That keeps each ecosystem's own test command meaningful: vitest, pytest, cargo test, and go test all run the same vectors natively. This is how serious cross-implementation specs work, from Google's Wycheproof crypto vectors to the CommonMark conformance tests.

Not every helper is data in and data out. Some have state and sequence: a Bin that accepts bytes until it's full, an Outline that's built and navigated and mutated. Those tests need a script, and a script in JSON is a half-built interpreter in four languages. So scripted tests are written once, in JavaScript, in the vitest suite. Each edition ships a small driver program that reads JSON lines on stdin and writes JSON lines on stdout: call this function with these arguments, construct this object as a handle, return the result or the error. The vitest suite talks to the JavaScript edition in-process and spawns the other three drivers, and a single test body runs against all four. The driver is the same dispatch table as the vector reader, fed from a pipe instead of a file. BoringSSL's ACVP testing works exactly this way, one harness driving any implementation in any language through a small wrapper binary.

A third kind of test is the fuzz test, and it stays out of the suite on purpose. The suite is deterministic and fast: pinned data, nothing random, tens of milliseconds, so it runs on every save and means the same thing every time. A fuzz test draws random inputs as fast as the workstation allows for a set number of seconds, usually one or four, and checks a property on all of them, like a tick round-tripping through both of its text forms with all four editions agreeing. It runs only when asked, and it's where a hand-rolled calendar gets measured against three platforms across millions of values, which no list of vectors could do.

Fuzz has two layers, and they're about different things. At the bottom, each edition fuzzes directly, the same loop written in that language inside its own test runner, switched on by the same ERGO_FUZZ, running the property at that edition's full speed and printing its count. That layer is about performance and vigor: millions of values in a few seconds, and the one place the four editions' speeds can be compared. Up top, the cross-edition fuzz in the vitest suite draws one random value and sends that same value through all four editions over the pipes, and requires the same answer from each. It runs orders of magnitude slower, tens of thousands of values rather than millions, because every value is a round trip to three driver processes. It's about uniformity, and it's the one thing the tight loops can't do: a tight loop proves an edition agrees with itself, and only the slow layer proves the four agree with each other on the same random data. Both are worth having, and neither replaces the other.

This is what makes four implementations safe to maintain. They can't drift apart quietly, because they're all measured against the same answer key, and the one place that runs everything is the JavaScript suite, which runs it against all four at once. A new function is specified in the suite first, then implemented four times until all four pass.

It also creates a design pressure, and a good one. Vectors are cheaper than scripts in every way, so the question to ask of each new helper is whether its hard core can be a pure function with a vector, leaving the script layer for the thin wrapper around it.

## Round trips

When ergo gives a value a text form, the relationship is one to one. Text to value to text returns the starting text, exactly. Value to text to value returns the starting value, exactly. There is never a text that parses but renders back differently, and never two texts for one value. This holds for everything that crosses between a value and text: the tick forms, base62 and the other encodings, Outline's text form, and whatever comes next. Apply it liberally, and expect it to be universal.

cold3's tick form is the model. Every millisecond since 1970 has one text, like 2012aug6.0517.57300, which is the shortest text that reads back to exactly that millisecond, and the tests walk every unit boundary and fuzz the round trip to prove it. sem.yaml's generated field is written in that form for the same reason.

What the rule costs is leniency, and the trade is deliberate. A reader that accepts variants, like a timestamp with or without its milliseconds, can't round-trip, because one of the variants has to render differently than it was read. So ergo's readers accept the canonical form and nothing else, and the canonical form is the one ergo writes. That strictness is what makes the text parsable without uncertainty, by a program, a person, or a model, and it keeps the code small: one writer, one reader, and a check that they agree.

In the tests, a round trip is a pair of vectors side by side, one each way: the text a value renders to, and the value that text parses to. Beyond the pairs, a fuzz test draws random values for a few seconds and checks the round trip on every one, in every edition at once.

## Style is not format

core.js is written in a house format: tabs, no line wrapping, vertical alignment across neighboring lines, no semicolons, comments jammed against the code. It's deliberate and it's good to read, and it would never pass in a shop where experienced JavaScript engineers keep a team on the same page with a formatter on save. The other three editions will be written to their standard formatters as a matter of course, since that's how one writes Rust or Go. It would be strange for three editions to look the way their ecosystems expect and the fourth to look like one person's editor.

So the two things are separated. Style is how code is structured, named, commented, factored, and how it handles errors and trust. That's quad-style.md, and it governs every edition. Format is whitespace, quotes, semicolons, and line breaks. That's each language's standard formatter, and it governs its edition. The layout rules of cold3's style guide, which quad-style.md grew from, are the one part of it that gives way.

Names sit on the line between the two. The words are style, and every edition shares them. The case is format, and each language sets its own. quad-style.md's rule against all-capital constants is its JavaScript reading of a deeper rule in the same guide, follow the environment's case, so Size.KB in Python and size::KB in Rust keep the rule rather than breaking it.

One deliberate wrinkle. JavaScript convention would also accept Size.KB and Time.HOUR for a constant, and a case could be made that capitals are the more correct spelling there too. ergo's JavaScript stays lowercase camelCase, Size.kb and Time.hour, because quad-style.md avoids shouts: all-capital names read loud and type clumsily. That's a house choice made within what JavaScript allows, not a rule pushed onto the other three, each of which capitalizes exactly where its own convention does. The row Size.kb, Size.KB, size::KB, ergo.KB is the whole principle in miniature: the same tool in four places, each spelled the way that place spells things.

This separation is the mechanism behind the mid-career engineer test above. The values are the same in every edition, and nothing the formatter or linter has an opinion about is left for them to object to.

## Decided so far

- One library, four natives. The shared suite enforces that the editions agree, and each ecosystem's own tools enforce that its edition belongs. Behavior, the words in the names, and the style guide's values are shared. Format, case, layout, packaging, test runner, example form, the mechanism for failing loudly, and the judgment of which dependencies are free are each language's own.
- Round trips are one to one. Text to value to text is the identity, and so is value to text to value, for every text form ergo defines. Readers accept the canonical form only, which is the form ergo writes.
- JavaScript first, published as @zootella/ergo. Then zootella-ergo on PyPI and crates.io, and github.com/zootella/ergo as a Go module.
- The platform first, judged function by function and form by form, and where it falls short, only libraries the ecosystem already trusts: noble in JavaScript for what Web Crypto can't do, blake3 and later cryptography in Python, RustCrypto and blake3 in Rust, lukechampine.com/blake3 in Go. The JavaScript edition runs completely in a browser tab and a web worker, with no Node and no WebAssembly.
- One test suite: JSON vectors read natively by all four editions, plus scripted tests in JavaScript under vitest that drive all four editions through a driver protocol. No hand-written tests in Python, Rust, or Go beyond the vector reader, the driver, and one fuzz loop each, the same loop in each language.
- Style and format are separate. quad-style.md governs structure, names, comments, and factoring in every edition. Format is each language's standard formatter, run on save and checked in CI: Prettier, Ruff, rustfmt, gofmt.
- The protocol's canonical function names are the JavaScript edition's, and each edition spells them in its own case: saySize4, say_size4, say_size4, SaySize4; Size.kb, Size.KB, size::KB, ergo.KB. A constant is a function with no arguments, so Size.kb is vectorable like anything else, and its dotted name is the protocol's.
- When an edition can't continue, it fails the way its language fails: throw an Error in JavaScript, raise in Python, panic in Rust and Go. cold3's toss and TossError stay behind. A throws vector means exactly that, and a scripted test sees it as a rejected call from any edition. How well-structured code handles a truly exceptional circumstance is something the four languages, from different generations of design thinking, each have opinions about, so making exceptions easy is the last and hardest thing ergo will do, not the first. For now, two things fail: a guard like checkTag that caught a caller's bug, loudly, and a reader like textToTick that was handed text it can't read, as a thrown Error or ValueError in the two languages that throw and as a returned Err or error in the two that don't.
- A panic means a bug, so ergo never leaves a caller unable to avoid one. Every kind of value a function refuses has a `has` beside it that asks and never fails, and a `check` that enforces: hasTick and checkTick before tickToIso, hasTag and checkTag before anything that takes a tag. A number from outside, like a system clock or a file's modified time, is asked about first, and once `has` says yes, nothing in ergo fails on it. This is Go's and Rust's own design, errors as values for ordinary outcomes and a panic only for a bug, with Rust's habit of a way to ask first beside every way to panic, like `is_char_boundary` before slicing a string. It's one predicate per kind of value rather than a checked twin of every function, so it stays small as ergo grows. Fuji taught it on its first day: a panic can't be caught the way an exception can, and its log would have aborted on a clock past 9999, because the panic hook that records every panic reads the clock again.
- Integers have one ceiling in every edition, JavaScript's safe range of ±(2^53 − 1). A value within it survives all four runtimes as an integer, and the vectors stay within it. The checkInt that each edition will offer means exactly that: this integer is safe in all four.
- A hasher, anything that takes bytes a piece at a time and then answers a hash, speaks the words the world already uses: `update` and `digest` in JavaScript and Python, as noble and hashlib do, `update` and `finalize` in Rust, and `Write` and `Sum` in Go, the standard library's own `hash.Hash`. That holds for the fuji hashes when they come over, so no hasher in ergo speaks icarus's `add` and `hash`. The seam between the two vocabularies is in cold3 instead: each call site that feeds a Bin and hashes it becomes `update` and `digest` as it moves from core.js to ergo, checked one by one at that moment rather than carried along.
- core.js is where the early milestones come from, and ergo takes its gems and leaves its rough and early parts behind. A function arrives because it earned its place, not because it was there.
- A tick is an integer of milliseconds since 1970, from zero through the last millisecond of 9999, so a year is always four digits. Two text forms carry one exactly: the iso form, 24 characters with milliseconds and Z, and the short form from cold3 that sem.yaml carries. Readers accept only what writers produce. Rust's standard library has no calendar, so the Rust edition carries the few lines of arithmetic itself, pinned by the vectors and the fuzz test against the other three platforms.
- Fuzz tests run only when ERGO_FUZZ names a number of seconds: the cross-edition one from vitest, about uniformity, and one per edition, about performance. The unit suite stays deterministic and fast.
- GitHub repo zootella/ergo, created once the starting form is settled. One repository holds all four editions, each in its own directory, with the shared vectors and the docs site beside them. construction.md has the layout.
- Documentation is one site, not four, built with VitePress, hosted on GitHub Pages at zootella.com itself, apex domain and root route. The README on npm, GitHub, and the other registries stays light and sends the reader there. Read the page about one area of ergo, flip the language switcher from Python to Rust, and every example on the page changes.
- Examples are real code in each language, in each ecosystem's native executable form, verified by that edition's own test command and pulled into the site from source. Tests of correctness stay in one suite; examples exist in four languages because they're shown in four languages.

## Open

- Handles for objects like Bin and Outline in the driver protocol. Bytes arrived with the hashes, and construction.md has their tagged form; a hasher needed no handle, since a vector hands it its pieces as a list.
- Whether a reader in JavaScript should refuse text it can't read with a value rather than a thrown Error. Text from outside is ordinary input, which the walled city of quad-style.md answers with values, as Rust and Go already do; raising stays native in Python, where `int("x")` raises. Tick zero is a real moment, so "the tick or false" is awkward.

## Later

One thing to come back to, and a file of more:

- A reference page per function showing its signature and comment in all four languages. No off-the-shelf tool spans four languages, so this is a small extractor of our own in the godoc spirit: read each edition's source, take the declaration and the comment on or above it, and emit markdown with four tabs. The style guide's comment discipline is what makes that extractor small.
- ideas.md collects ideas beyond the core requirements: the site running its own tests, a demonstration CLI, a playground, the brand, and demonstrators.

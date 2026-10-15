# ruby

Whether to add Ruby as a fifth edition, and the wider question of what a fifth language would be for. Parked here as of October 2026, with the discussion that produced it. The four editions in concept.md are the first set, and nothing here is committed to.

## The question

Ruby is the language the wider community of open source engineers most often calls the best designed, the most rational, and the most beautiful. A library in four languages that leaves it out invites the question why. And beyond the vibes, a real question: would a Ruby column be additional weight, or could it make the other cells in the grid better?

## What Ruby would bring

Technically it fits every bar in construction.md, and most of them easily.

- Dependencies. A model citizen, beside Go. The openssl default gem covers HMAC, PBKDF2, AES-GCM, RSA, and curves, with SecureRandom and Digest beside it, so every column of the dependency table would say platform.
- Registry and name. RubyGems, where the gem `zootella-ergo` is required as `zootella/ergo` and defines `Zootella::Ergo`, which parallels the Python namespace package exactly. Trusted publishing from GitHub Actions is supported.
- Tests. Minitest ships with Ruby, JSON is a default gem, and a failure raises, as in Python. The vector reader and the driver would be the easiest of the foreign editions to write.
- Examples. Ruby has no doctest, so examples live in a test file with region markers, the way the JavaScript ones do.

Then the thing that came out of the discussion. style.md already traces two of its values back to languages in the set: simplicity to Python, where simple is better than complex is a stated core value, and errors as values to Go, which made that bet at the language level. Ruby would lend a third. Its `?` suffix on predicates and `!` suffix on dangerous methods are style.md's prefix promises, has* and is* to ask and check* to enforce, formalized into the language itself. With Ruby in the set, that element of the style has a citation: the other editions spell with a prefix what Ruby spells with a suffix, and a reader who knows Ruby recognizes the rule on sight. That is a real way a fifth language helps the other four, even though the idiom itself doesn't port. Each language in the set lends a precedent, and the precedents are the style guide's bibliography.

## What it would cost, and why not now

- The set is about environments. concept.md picks each language from a place the author works: the browser and Node, a Tauri backend, a server script, a Go service. Rust is the systems slot, Go the server slot, Python the scripting slot, JavaScript the browser slot. Ruby lands in Python's slot. Nothing one reaches for Ruby to do is something Python isn't already covering.
- Design pressure comes from difference, and its lessons are front-loaded. The first port of a function is where you learn what it really does under the idiom. The second catches what the first missed. By the fourth you're transcribing. The three ports already in the set are maximally different from each other. Ruby is a second dynamic scripting language beside Python, the least different candidate there is, so it exerts the least pressure.
- Ruby's beauty lives in parts that don't cross. Blocks, everything an expression, optional parentheses, open classes, DSLs: none of it is portable. What crosses are names, factoring, comments, and error handling, and there Ruby's culture pulls both ways against style.md. The suffixes above are the style guide's own promises. But everyday `raise` is the opposite of result objects, and the many-tiny-methods and metaprogramming habits are the opposite of fewer longer files and plainness. Matz's principle of least surprise and style.md's simplicity are kindred values, and the value is already held.
- Format is the weakest fit. Ruby has no mandatory formatter like gofmt and no dominant one like Prettier. RuboCop is universal as a linter, but every shop configures it differently, and its defaults are an opinionated preset of exactly the kind construction.md rules out. Standard is the no-arguments option, but it is one camp, not the ecosystem. The mid-career engineer test gets blurry here; the nearest answer is RuboCop with only its Lint department on, plus its autocorrect for layout.
- The honest test. Is there a Ruby project that wants base62? If yes, add it. If no, it's an edition nobody runs in anger, which is the one thing the shared suite can't protect against: tests prove the editions agree, not that anyone needs the fifth one.

Nothing has to be decided now. The architecture makes a new edition a known cost, a dispatch table, a driver, a vector reader, a formatter in CI, a tab on the site, and nothing in the protocol ossifies against a fifth language. The bytes and handles design will be language-neutral regardless, so there's nothing to pre-build. And the pressure is in the rows, not the columns: the next functions, bytes, Bin, Outline, base62, and the crypto, will stress the protocol and all four editions far harder than a fifth language would. Each new row tests every column. Each new column re-tests the protocol with diminishing novelty.

Starting with Ruby, implementing the JavaScript parts in Ruby first to evaluate them and spread the Ruby way to the others, would be a detour. The JavaScript edition is the reference by design and carries years of real use from core.js. Ruby first means translating idioms twice before reaching the editions that will run, and what it would mostly teach is how the Ruby edition should look.

## Other languages

Ruby is one of many absences, and the question of why not comes from every direction: enterprise people ask about Java and C#, mobile people about Swift and Kotlin, web people about PHP, Flutter people about Dart. The one-line answer covers them all: these four are where the author writes code. But a few candidates would bring something specific.

**C** is the maximally different one, and the strongest candidate for pressure. No text type, only bytes with an encoding convention. No exceptions, so a guard fails loudly with abort, and a driver can't catch that in-process, which forces a real decision about how a throws vector is observed: a fork per call, or a longjmp handler. Universal, too: every other language can call C. The frictions are real. There is no registry, and the native install story is vendoring a `.c` and `.h`, or a single header. The standard library has no JSON, so the vector reader and the driver need a small parser vendored beside them, like serde_json behind Rust's driver feature, never in the library. And it has no crypto, where the universally trusted answer is OpenSSL or libsodium, or a hand-written SHA-256 that the vectors pin. clang-format is the formatter, with a chosen style checked in, and clang-tidy with the compiler's warnings is the linter.

**Swift** is next by environment coverage, the native Mac and iOS slot, and technically a natural fifth: swift-format ships with the toolchain, Swift Package Manager is the one tool, packages are git URLs like Go modules, and CryptoKit on Apple platforms with swift-crypto elsewhere is the platform answer for crypto.

**A pure functional language**, Haskell, Elixir, or Clojure, would exert the one pressure the set lacks: it would force every helper's hard core to be the pure function style.md asks for, because there would be no other way to write it.

## Where this stands

Not now. The core four are an excellent first set, and a fifth waits for a pull. If a fifth and a sixth ever arrive, the likeliest are C and Ruby: C for the pressure and the universality, Ruby for the lineage and the community.

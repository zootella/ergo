# Hash

Three hashes, each the one the world already knows. SHA-1 is the oldest, the one older manifests name a file by and git names its objects with; it's broken for collisions, so use it to match a hash that already exists, never to protect anything. SHA-256 is the standard everywhere today. BLAKE3 is the newest, built for speed as a tree, and the one to name a file by for years to come.

:::tabs key:lang
== JavaScript
<<< ../javascript/ergo.test.js#hash
== Python
<<< ../python/src/zootella/ergo/__init__.py#hash
== Rust
<<< ../rust/examples/hash.rs#hash
== Go
<<< ../go/example_test.go#hash
:::

Each hash comes in two forms that give the same answer. The function takes all the bytes at once, for data already in memory: `sha1`, `sha256`, and `blake3`, which answer 20, 32, and 32 bytes. The hasher takes them a piece at a time, as a file or a stream delivers them, so a file of any size hashes in the memory of one piece, and one read of the file can feed all three. Each language spells the hasher its own way:

| | JavaScript | Python | Rust | Go |
|---|---|---|---|---|
| all at once | `sha256(d)` | `sha256(d)` | `sha256(d)` | `SHA256(d)` |
| a hasher | `Sha256()` | `SHA256()` | `Sha256::new()` | `NewSHA256()` |
| each piece | `h.update(p)` | `h.update(p)` | `h.update(p)` | `h.Write(p)` |
| the hash | `h.digest()` | `h.digest()` | `h.finalize()` | `h.Sum(nil)` |

SHA-1 and BLAKE3 follow the same pattern: `Sha1()` and `Blake3()` in JavaScript, `SHA1()` and `BLAKE3()` in Python, `Sha1` and `Blake3` in Rust, `NewSHA1()` and `NewBlake3()` in Go. In every language, reading the hash doesn't finish a hasher: feed it more and read the hash again for everything so far. In Rust each hasher is also an `io::Write`, and in Go it's the standard library's own `hash.Hash`, so `io::copy` and `io.Copy` can feed one straight from a file. Bytes are each language's own: a `Uint8Array`, `bytes`, `&[u8]`, and `[]byte`.

The JavaScript edition runs in a page and a web worker as well as Node, and it hashes synchronously, so a page with a large file should hash in a worker, or between the reads of a stream, which give the page a moment between pieces. It uses noble's hashes for all three, since the browser's Web Crypto has SHA-1 and SHA-256 only all at once and no BLAKE3. Python uses hashlib and the official blake3 package, Rust uses RustCrypto's sha1 and sha2 and the official blake3 crate, and Go uses its standard library and lukechampine.com/blake3.

Each example above is real code from its edition, verified by that edition's own tests: a vitest demonstration, a Python doctest, a Rust example that cargo compiles, and a Go example that `go test` runs. The shared vectors pin all three hashes with published answers: FIPS 180's examples for SHA-1 and SHA-256, a million letter a's among them, and BLAKE3's official vectors, which walk its tree from the empty input to a hundred chunks. They pin the piece-at-a-time form too, with inputs split at block and chunk boundaries and between them. And a fuzz test sends random bytes, cut into random pieces, through all four editions and requires one answer from every one.

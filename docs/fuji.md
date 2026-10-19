# Fuji hashes

Two hashes of ergo's own, from icarus, for files of any size, on a page as well as a server. Both are SHA-256 over a summary of a file rather than over the file itself, and both answer a [Data](/data).

The **tip hash** reads almost nothing: the file's first 4 KiB, 4 KiB from its middle, and its last two, at most 16 KiB however large the file. It's done in a few reads, and it finds likely copies of a file across millions of others for the cost of those few reads each. A file of 16 KiB or less is its own tip. The **piece hash** reads everything: it cuts the file into 4 MiB pieces, hashes each one, and hashes the list of those hashes.

:::tabs key:lang
== JavaScript
<<< ../javascript/ergo.test.js#fuji
== Python
<<< ../python/src/zootella/ergo/__init__.py#fuji
== Rust
<<< ../rust/examples/fuji.rs#fuji
== Go
<<< ../go/example_test.go#fuji
:::

`hashFile` computes the tip hash from a file it can read at offsets, a `File` or `Blob` in JavaScript, a seekable file in Python, anything `Read` and `Seek` in Rust, and an `io.ReaderAt` in Go, which an `*os.File` is. `hashStream` computes both hashes in one read of a stream, a `ReadableStream`, a readable file, a `Read`, or an `io.Reader`, noticing the tip's stripes as they pass on the way to the piece hash; the two ways to the tip hash always agree. Each answers a status that holds the hashes, the sizes, and how long it took, and `hashStream` can call a function with that status as each piece is hashed. In JavaScript it also takes an `AbortSignal`.

Each summary starts with a title that names the protocol and the file's size, like `Fuji.Tips.SHA256.4KiB.5.`, so two files of different sizes never share a hash, even with identical tips. `hashProtocol` holds the titles and the sizes, 4 MiB pieces and 4 KiB stripes, and a protocol of its own can be passed instead, which is how the tests run the same code with pieces of a few bytes. An empty file has no fuji hash; both measures need a size of at least one byte.

Why pieces? Because the browser's Web Crypto hashes only all at once, a page can't hash a large file through it without holding all of the file in memory. Hashing 4 MiB pieces and then the list of them lets a page hash a file of any size at Web Crypto's full native speed. 4 MiB hashes in about 10 ms, like an animation frame, and uploads in under a second on a cable modem, and the list of piece hashes stays smaller than one piece for files under about 500 GB. 4 KiB stripes match the cluster size of NTFS and the block size of APFS, so the tip's reads are as fast as a disk allows.

The shared vectors pin both hashes against icarus's own answers: its smoke test of the five bytes of hello, its demonstration at a piece size of four bytes, where changing only the bytes the tip can't see leaves the tip the same and changes the piece hash, and four of its small files. Its larger files, 6 MB and more, are checked in every edition when the fuzz tests run, alongside random tiny files at random tiny piece sizes.

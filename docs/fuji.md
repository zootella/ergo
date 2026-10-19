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

## The hasher

At the center of `hashStream` is `FujiHasher`, which takes a file a piece at a time, for bytes that arrive as events rather than as a stream: the chunks of an upload a page sends on to a bucket, say, which the page hashes as they go, and which a Lambda hashes again on the other side to confirm the file arrived intact. Make one with the file's size, `update` it with each piece in order, each a Data of any size, and `digest` for both hashes, `finalize` in Rust, the same words as the [hashers](/hash). `FujiHasher({size})` in JavaScript, `FujiHasher(size)` in Python, `FujiHasher::new(size)` in Rust, and `NewFujiHasher(size)` in Go, where it's also an `io.Writer`, as it is an `io::Write` in Rust.

`update` never fails. Pieces that don't add up to the size the hasher was told are a fact about the bytes that came in, not a bug, so `digest` refuses them: it throws in JavaScript, raises a `ValueError` in Python, and returns an error from `finalize` in Rust and `Digest` in Go. In JavaScript `update` answers a promise, since each piece goes through Web Crypto, and each one waits for the one before, so the pieces hash in order. Await each one anyway, so a fast source can't pile up pieces that haven't been hashed yet, and leave a piece's bytes alone until its update answers.

This is how ergo does events and streams: a small part you feed, and a thin wrapper that reads the language's own stream into it, which is where a progress callback and cancelling live. With a stream in Node or Lambda, call `hashStream`. With chunks arriving in a page, feed a `FujiHasher`. With a file you can read at offsets, call `hashFile`. Each works where its language does, and they compose.

Each summary starts with a title that names the protocol and the file's size, like `Fuji.Tips.SHA256.4KiB.5.`, so two files of different sizes never share a hash, even with identical tips. `hashProtocol` holds the titles and the sizes, 4 MiB pieces and 4 KiB stripes, and a protocol of its own can be passed instead, which is how the tests run the same code with pieces of a few bytes. An empty file has no fuji hash; both measures need a size of at least one byte.

Why pieces? Because the browser's Web Crypto hashes only all at once, a page can't hash a large file through it without holding all of the file in memory. Hashing 4 MiB pieces and then the list of them lets a page hash a file of any size at Web Crypto's full native speed. 4 MiB hashes in about 10 ms, like an animation frame, and uploads in under a second on a cable modem, and the list of piece hashes stays smaller than one piece for files under about 500 GB. 4 KiB stripes match the cluster size of NTFS and the block size of APFS, so the tip's reads are as fast as a disk allows.

The shared vectors pin both hashes against icarus's own answers: its smoke test of the five bytes of hello, its demonstration at a piece size of four bytes, where changing only the bytes the tip can't see leaves the tip the same and changes the piece hash, and four of its small files. The hasher has vectors of its own: fed a byte at a time, fed two pieces in one update, and fed pieces that don't add up to the size, which every edition refuses. When the fuzz tests run, every edition also checks icarus's larger files, 6 MB and more, alongside random tiny files at random tiny piece sizes and random files of up to 20 MiB with the real protocol, each hashed from a file, from a stream, and fed to a hasher.

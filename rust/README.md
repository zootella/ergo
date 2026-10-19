# zootella-ergo

Simple helper functions, the same in JavaScript, Python, Rust, and Go. This is the Rust edition, on the standard library plus getrandom for the operating system's random source, RustCrypto's sha1 and sha2, the blake3 crate, and unicode-normalization for text in Normalization Form C.

```
cargo add zootella-ergo
```

```rust
use zootella_ergo::{Data, say_size4, sha256, size, tag, tick_to_iso, tick_to_text, time};

let t = tag(); // a new tag, like "AgKxru95C7jFp5iPuK9O7": 21 letters and numerals
let buffer = vec![0u8; 64 * size::KB as usize]; // a buffer of 64 kibibytes, rather than a bare 65536
say_size4(123_456_789); // "117 MB": four digits or less, with a unit
let deadline = now + 2 * time::HOUR; // two hours after a tick, in milliseconds
tick_to_text(deadline); // "2026oct15.2041.7125", a tick's short exact text
tick_to_iso(deadline); // "2026-10-15T20:41:07.125Z", the same tick's iso text
let d = Data::from_text("hello"); // bytes, which read out as base16, base32, base62, base64, or text
sha256(&d).base32(); // the SHA-256 hash, as a Data, out in any form; sha1 and blake3 work the same way
```

Documentation for every function, in all four languages, is at [zootella.com](https://zootella.com). The source and the shared test suite are at [github.com/zootella/ergo](https://github.com/zootella/ergo).

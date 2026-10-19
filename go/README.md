# ergo

Simple helper functions, the same in JavaScript, Python, Rust, and Go. This is the Go edition, on the standard library, plus lukechampine.com/blake3 for BLAKE3 and golang.org/x/text for text in Normalization Form C.

```
go get github.com/zootella/ergo/go
```

```go
import ergo "github.com/zootella/ergo/go"

t := ergo.Tag()                    // a new tag, like "AgKxru95C7jFp5iPuK9O7": 21 letters and numerals
buffer := make([]byte, 64*ergo.KB) // a buffer of 64 kibibytes, rather than a bare 65536
ergo.SaySize4(123456789)           // "117 MB": four digits or less, with a unit
deadline := now + 2*ergo.Hour      // two hours after a tick, in milliseconds
ergo.TickToText(deadline)          // "2026oct15.2041.7125", a tick's short exact text
ergo.TickToISO(deadline)           // "2026-10-15T20:41:07.125Z", the same tick's iso text
d := ergo.DataFromText("hello")    // bytes, which read out as base16, base32, base62, base64, or text
ergo.SHA256(d).Base32()            // the SHA-256 hash, as a Data, out in any form; SHA1 and Blake3 work the same way
```

Documentation for every function, in all four languages, is at [zootella.com](https://zootella.com). The source and the shared test suite are at [github.com/zootella/ergo](https://github.com/zootella/ergo).

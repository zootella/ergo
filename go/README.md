# ergo

Simple, dependency-free helper functions, the same in JavaScript, Python, Rust, and Go. This is the Go edition, on the standard library alone.

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
```

Documentation for every function, in all four languages, is at [zootella.com](https://zootella.com). The source and the shared test suite are at [github.com/zootella/ergo](https://github.com/zootella/ergo).

# @zootella/ergo

Simple helper functions, the same in JavaScript, Python, Rust, and Go. This is the JavaScript edition: one ESM file that runs in a page, a web worker, and Node 22 or newer, on the platform and one dependency, noble's pure JavaScript hashes, for what the platform's Web Crypto can't do.

```
npm install @zootella/ergo
```

```js
import {
  Tag,
  Size,
  saySize4,
  Time,
  tickToText,
  tickToIso,
  Data,
  sha256,
} from '@zootella/ergo'

let t = Tag() // a new tag, like "AgKxru95C7jFp5iPuK9O7": 21 letters and numerals
let buffer = new Uint8Array(64 * Size.kb) // a buffer of 64 kibibytes, rather than a bare 65536
saySize4(123456789) // "117 MB": four digits or less, with a unit
let deadline = Date.now() + 2 * Time.hour // two hours from now, in milliseconds
tickToText(deadline) // "2026oct15.2041.7125", a tick's short exact text
tickToIso(deadline) // "2026-10-15T20:41:07.125Z", the same tick's iso text
let d = Data({ text: 'hello' }) // bytes, which read out as base16, base32, base62, base64, or text
;(await sha256(d)).base32() // the SHA-256 hash, as a Data, out in any form; sha1 and blake3 work the same way
```

Documentation for every function, in all four languages, is at [zootella.com](https://zootella.com). The source and the shared test suite are at [github.com/zootella/ergo](https://github.com/zootella/ergo).

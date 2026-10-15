# zootella-ergo

Simple, dependency-free helper functions, the same in JavaScript, Python, Rust, and Go. This is the Python edition, on the standard library alone.

```
uv add zootella-ergo
```

```python
from zootella.ergo import Size, Time, say_size4, tag, tick_to_iso, tick_to_text

t = tag()  # a new tag, like "AgKxru95C7jFp5iPuK9O7": 21 letters and numerals
buffer = bytearray(64 * Size.KB)  # a buffer of 64 kibibytes, rather than a bare 65536
say_size4(123456789)  # "117 MB": four digits or less, with a unit
deadline = now + 2 * Time.HOUR  # two hours after a tick, in milliseconds
tick_to_text(deadline)  # "2026oct15.2041.7125", a tick's short exact text
tick_to_iso(deadline)  # "2026-10-15T20:41:07.125Z", the same tick's iso text
```

Documentation for every function, in all four languages, is at [zootella.com](https://zootella.com). The source and the shared test suite are at [github.com/zootella/ergo](https://github.com/zootella/ergo).

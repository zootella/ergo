# ergo

Simple, dependency-free helper functions, the same in JavaScript, Python, Rust, and Go.

|            | Install                               |
| ---------- | ------------------------------------- |
| JavaScript | `npm install @zootella/ergo`          |
| Python     | `uv add zootella-ergo`                |
| Rust       | `cargo add zootella-ergo`             |
| Go         | `go get github.com/zootella/ergo/go`  |

The four editions share function names and behavior, and one test suite proves it: JSON vectors in `tests/` that every edition runs natively, and a JavaScript suite that drives all four editions side by side. Each edition is also a normal library in its own ecosystem: written and formatted the way that language is, packaged for its registry, and tested by its own test runner.

Documentation is at [zootella.com](https://zootella.com), built from `docs/`. For what ergo is and how it's built, read `concept.md` and `construction.md`.

## Running everything

Each edition runs its own tests with its own tool. The JavaScript suite additionally starts the Python, Rust, and Go drivers and tests all four editions together, so it needs all four toolchains installed.

```
cd javascript && pnpm install && pnpm test
cd python && uv run pytest
cd rust && cargo test
cd go && go test ./...
```

The unit suite is deterministic and takes milliseconds. Fuzz tests draw random inputs for a set number of seconds and run only when asked. The JavaScript one also runs every value through all four editions and checks that they agree, which is about uniformity; each edition's own runs at that edition's full speed and prints its count, which is about performance.

```
cd javascript && ERGO_FUZZ=4 pnpm fuzz
cd python && ERGO_FUZZ=4 uv run pytest -s -k fuzz
cd rust && ERGO_FUZZ=4 cargo test --release --test fuzz -- --nocapture
cd go && ERGO_FUZZ=4 go test -run Fuzz -v
```

## Releasing

One command releases all four editions at a new version, from a clean main:

```
pnpm release patch    # 0.1.0 to 0.1.1, for a release that adds
pnpm release minor    # 0.1.0 to 0.2.0, for one that changes an existing answer
```

It waits for GitHub's tests to pass on main, sets the version in every manifest, commits and tags it, and pushes. GitHub tests the tag again and publishes to npm, PyPI, crates.io, and the Go proxy, and the command watches until the GitHub release is up. It needs `gh` logged in. `construction.md` has the details.

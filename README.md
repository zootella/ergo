# ergo

Simple helper functions, the same in JavaScript, Python, Rust, and Go.

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

The JavaScript suite drives all four editions unless `ERGO_EDITIONS` names some of them. That's how a new family arrives: its vectors are committed first, green against JavaScript alone, and then each other edition is written against them and added to the run. A run leaves out the editions it doesn't name and counts what it skipped, and a name it doesn't know is an error.

```
cd javascript && ERGO_EDITIONS=javascript pnpm test
cd javascript && ERGO_EDITIONS=javascript,python pnpm test
```

The unit suite is deterministic and takes milliseconds. Fuzz tests draw random inputs for a set number of seconds and run only when asked. The JavaScript one also runs every value through all four editions and checks that they agree, which is about uniformity; each edition's own runs at that edition's full speed and prints its count, which is about performance.

```
cd javascript && ERGO_FUZZ=4 pnpm fuzz
cd python && ERGO_FUZZ=4 uv run pytest -s -k fuzz
cd rust && ERGO_FUZZ=4 cargo test --release --test fuzz -- --nocapture
cd go && ERGO_FUZZ=4 go test -run Fuzz -v
```

## Releasing

Most commits don't need a release. Release when there's finished work you want other projects to be able to install: commit as usual, then run one command from the repository root, on main.

```
git add .
git commit -m "message"
pnpm release patch    # 0.1.0 to 0.1.1, for a release that only adds, like a new function
pnpm release minor    # 0.1.0 to 0.2.0, for one that changes an answer an existing function already gave
```

Choose the word by what changed. While the major version is zero, a project that asks for `^0.1.0` in package.json or `"0.1"` in Cargo.toml takes every patch automatically and a minor only when it opts in, so a changed answer never reaches anyone unasked. `pnpm release 1.0.0` names a version exactly.

The command does the rest. It refuses to start with uncommitted changes, pushes main if GitHub hasn't seen it, and waits for GitHub's tests to pass on it. Then it reads the current version from the five files that hold it, works out the next one, and writes it the way each ecosystem does: `npm version` for package.json, `uv version` for pyproject.toml and uv.lock, and Cargo.toml's version line with `cargo update` for Cargo.lock. It commits that as `ergo 0.1.1`, tags the commit `v0.1.1`, and pushes both. The tag starts the publish workflow on GitHub, which tests the tagged commit again, publishes to npm, PyPI, and crates.io, and tags `go/v0.1.1` for the Go module, whose version lives only in that tag. The command watches the run and ends with the address of the GitHub release, which links the new version on all four registries. No version is ever edited by hand, and there's nothing to pull afterward.

Each release reaches four places, one for each edition's ecosystem:

- **JavaScript**, [@zootella/ergo on npm](https://www.npmjs.com/package/@zootella/ergo), published from `javascript/` with `npm publish`.
- **Python**, [zootella-ergo on PyPI](https://pypi.org/project/zootella-ergo/), built with `uv build` and uploaded with `uv publish`.
- **Rust**, [zootella-ergo on crates.io](https://crates.io/crates/zootella-ergo), published with `cargo publish`, after which [docs.rs](https://docs.rs/zootella-ergo) builds the crate's reference pages on its own.
- **Go**, [github.com/zootella/ergo/go on pkg.go.dev](https://pkg.go.dev/github.com/zootella/ergo/go). Go has no registry to upload to: the `go/v0.1.1` tag in this repository is the release, the workflow asks [proxy.golang.org](https://proxy.golang.org) to fetch it, and pkg.go.dev shows it from there.

Every version also gets a page under this repository's [releases](https://github.com/zootella/ergo/releases), listing the commits since the last one.

It needs `gh` logged in. `construction.md` has the details.

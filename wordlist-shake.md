# wordlist shake

Notes on adding the BIP-39 English wordlist to ergo, for hashes a person can read aloud, and on keeping it out of the programs that never use it. Written in October 2026, before any of it is in ergo. The list is big next to the rest of ergo, so it only belongs here if it shakes out cleanly, and the measurements below say it does, in three of the four editions, as long as each edition follows one rule. They come from small test projects built to look like ergo, not from ergo itself.

## The idea

The application ergo grew out of already uses the list in one place, `prefix39`. It turns a hash into a six-character label like `Deci71`, and stamps each build with one. It takes the first four letters of a BIP-39 word, capitalizes the first, and adds digits. Bytes 0 and 1, read as a big-endian uint16 mod 2048, pick the word, with no bias since 65536 is a multiple of 2048. Bytes 2 through 5, read as a uint32 mod 1000, give the digits, with a bias of about 0.00002%, since 2^32 mod 1000 is 296. A three-letter word gets three digits and a four-letter one gets two, so the label is always six characters.

`dual39` would be its longer sibling: two words and four digits, always, with no padding, like `NewsRave2214`. Since a three-letter word adds no digit, a label runs 10 to 12 characters. It's for spot-checking files, where hundreds of thousands of them are around and nobody is attacking, and `prefix39` isn't quite strong enough. The application uses `prefix39` today, and the first app built on ergo would use `dual39`.

BIP-39 suits this because it was made for people to read, write down, and type back. The first four letters always identify a word, so the four-letter form loses nothing, and 2048 words is exactly 11 bits.

## The list

2048 words, from 3 to 8 letters, 5.4 on average, and 103 of them are three letters.

| form | raw | gzip -9 | brotli |
|---|---|---|---|
| the words, separated by newlines | 13,116 B | 6,256 B | 5,212 B |
| `@scure/bip39/esm/wordlists/english.js` | 13,155 B | 6,303 B | 5,248 B |

The scure file is the list as one template string, split on newlines, so it adds nothing to speak of, and each language sits in a file of its own. For scale, `ergo.js` is 69,787 B raw and 20,053 B brotli, comments included, so the list alone would add about a quarter to the JavaScript edition's compressed size. Each package download, on npm, PyPI, crates.io, and the Go module proxy, grows by about 5 KB compressed.

## The numbers

`dual39` holds 2048² × 10⁴ ≈ 41.9 billion labels, which is 22 + log₂ 10,000 ≈ 35.29 bits.

| form | bits per character | characters for 35.3 bits |
|---|---|---|
| base16 | 4 | 9, at 36 bits |
| base62 | 5.95 | 6, at 35.73 bits |
| two words and four digits | 2.94 | 12 |

The words take twice the length of base62 for the same uniqueness, and a third more than base16. What they buy is a label a person can say, remember, and type back.

How strong is strong enough depends on which of two questions the label answers. When a person checks whether this hash matches that one, the only risk is two different files showing the same label: about 1 in 215,000 for `prefix39`, and 1 in 46 billion for `dual39`, per comparison. When a label tells files apart in a collection, any two matching counts, the birthday problem. For `dual39`, the expected number of matching pairs is about 0.1 among 100,000 files, 1 among 300,000, and 11 among 1,000,000. So `dual39` is plenty for spot checks, and it reaches its limit right around the scale above if it's ever used as a name for files. The full hash is there for that.

Padding a three-letter word with a digit, as `prefix39` does, only adds labels, so it only makes collisions rarer. But it helps less than counting the labels suggests. Labels are equally likely only if every word is followed by the same number of digits, and padding gives the rare words more digits while the common ones keep fewer. What decides collisions is the chance two random labels match, so the table gives the effective count, the number of equally likely labels that would match as often.

| | labels by count | effective labels | 50% chance of a collision after |
|---|---|---|---|
| one word, always two digits | 204,800 | 204,800 | 533 |
| one word, padded, which is `prefix39` | 297,500 | 214,509 | 545 |
| two words, always four digits, which is `dual39` | 41.9 billion | 41.9 billion | 241,000 |
| two words, padded | 88.5 billion | 46.0 billion | 253,000 |

The comment above `prefix39` in the original says 297,500 values and 50% after about 655 hashes, which counts labels as if each were equally likely.

## The shake

The rule in every edition: the list is a plain string, and nothing splits it into words until a function needs one. Splitting it when the module loads, or setting up anything at load time that refers to it, keeps all of it in every program that imports ergo.

| | split on first call, never called | split at load, never called |
|---|---|---|
| JavaScript | dropped, the bundle is 43 B | kept, the bundle is 13,172 B |
| Python | +13 KB of memory, import time unchanged | +125 KB of memory, about 0.7 ms more to import |
| Rust | dropped, in dev and release | can't happen, a static can't run code at load |
| Go | dropped, for macOS, Linux, and Windows | kept, and split at every program start |

When a program does call one, the list comes along: a Rust binary grew by about 17.9 KB and a Go binary by about 17 KB. In Rust and Go the test shape was the real one, `prefix39` and `dual39` sharing one private `wordAt`, built three times: calling neither, calling only `prefix39`, and calling only `dual39`. The list was missing from the first build and present in the other two, in every case.

### JavaScript

Rolldown, the bundler under Vite 8, drops an unused top-level string, since a literal can't do anything. It keeps `words.split('\n')` at the top level, because it can't prove a method call is safe to drop, and the string comes along with the call. ergo's package.json already says `"sideEffects": false`.

```js
let _words = `abandon
ability
...
zoo`
let _wordList
function _wordAt(i) {
  if (!_wordList) _wordList = _words.split('\n') //split on first use, so a bundler sees only a string it can drop
  return _wordList[i]
}
```

### Python

Python has no shaking, so `import zootella.ergo` always loads the list: about 13 KB in the source, 13.7 KB in the `.pyc`, and 13 KB of memory in every process. Splitting at import would also build 2048 string objects, taking 125 KB and most of a millisecond, so the split waits here too.

```python
WORDS = """abandon
ability
...
zoo"""
_word_list = None


def _word_at(i):
    global _word_list
    if _word_list is None:
        _word_list = WORDS.split("\n")
    return _word_list[i]
```

### Rust

Nothing special. A static can't run code when the program starts, so the list stays a string until `get_or_init` runs, and the linker drops what nothing reachable uses. Tested on macOS only, since the Linux and Windows targets weren't installed. rustc passes the same dead-code flag to those linkers by default, `--gc-sections` and `/OPT:REF`, so they should match, but that's unverified. A build with `-C link-dead-code`, which some coverage tools turn on, keeps everything.

```rust
static WORDS: &str = "abandon
ability
...
zoo";
static WORD_LIST: OnceLock<Vec<&str>> = OnceLock::new();

fn word_at(i: usize) -> &'static str {
    WORD_LIST.get_or_init(|| WORDS.lines().collect())[i]
}
```

### Go

Go's linker drops unreachable package-level data, but not anything package init runs. So the list is a `const`, the package-level `var` declarations hold only a `sync.Once` and an empty slice, and the split happens inside the function. The trap is `var wordList = sync.OnceValue(func() []string { ... })` at package level, which looks lazy, but builds its closure during init, and init refers to the list, so every program that imports ergo carries it. Go uses its own linker on every platform, and the builds cross-compiled for Linux and Windows on amd64 matched macOS.

```go
const words = `abandon
ability
...
zoo`

var (
	wordsOnce sync.Once
	wordList  []string
)

func wordAt(i int) string {
	wordsOnce.Do(func() { wordList = strings.Split(words, "\n") })
	return wordList[i]
}
```

## How it was measured

On macOS arm64, with Node 22.21.0 and rolldown 1.2.14, Python 3.15.0, cargo 1.98.0, and Go 1.26.5. Each test project put the list in a library module as a newline-separated string, with an app importing that library. A bundle or binary had the list in it if the word "blouse" appeared, which the application ergo grew out of once used for the same search, since it's unlikely to show up anywhere else. Python's memory is what tracemalloc held after a second import, with the `.pyc` already written.

Not measured: compile time, which a 13 KB string literal shouldn't change much next to an 86 KB lib.rs and a 65 KB ergo.go; and Rust on Linux and Windows.

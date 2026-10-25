# Parallel design

ergo is four editions of one library, so some of its design can't be settled one family at a time. How a call fails, how a collection is looped over, and how bytes arrive over time are each answered by every language in its own idiom, and each language has changed its answer as it grew. Getting them right means a pass across the whole library, in all four editions at once: the same words and the same behavior everywhere, and in each edition the form an expert in that language expects. Each section below is a map for that pass. None of it is planned yet.

## Errors

quad-style.md settles the large shape of failure in ergo, and every edition follows it. A caller's bug fails loudly: a thrown Error in JavaScript, a raised exception in Python, a panic in Rust and Go. An ordinary answer is a value: a Result or an error in Rust and Go, where the language makes it one, while Python raises and JavaScript throws for now. Every kind of value a function refuses has a `has` to ask first. The shared vectors pin which calls refuse, in every edition, and nothing about how.

Below that shape, the details have grown up function by function, and they don't yet agree. This section collects them, for a pass across the whole library rather than a fix in one family.

### Python's exception types

Python's own rule is the one a Python engineer carries: TypeError for a value of the wrong type, ValueError for a value of the right type that's wrong anyway. `int(None)` raises the first and `int("x")` the second. ergo follows it in some places and not others:

- `_whole_tick` and `_whole_span` follow it, raising TypeError for a value that isn't a number and ValueError for one outside the range.
- `_check_data`, and Outline's `set_value` and `add`, raise TypeError for the wrong type.
- The public guards raise ValueError for both: `check_string`, `check_text`, `check_int`, `check_tick`, `check_tag`, and `check_outline_name`. So do the readers handed something that isn't a string, like `text_to_outline(b"...")`.

So `data_to_outline("...")` raises TypeError and `text_to_outline(b"...")` raises ValueError, for the same mistake. Following Python's rule everywhere means the guards raise TypeError when the type is wrong. A caller that catches ValueError around a reader would then miss the wrong-type case, which is arguably right, since that case is a bug, not an answer, but it's a change to make on purpose, in one pass, with a note.

Outline adds two more exception types, which fit Python. `get` raises IndexError for an index past the end, and `n` raises KeyError for a name it can't find, the way a list and a dict do.

### Errors as values in Rust and Go

Rust has three error types:

- **`ParseError(pub String)`**, from the tick readers, which writes "not a text form of a tick"
- **`DataError {form, text}`**, from Data's readers, which writes "not base16" and the like
- **`OutlineError(pub String)`**, from the outline readers and `span_decode`, which writes whatever went wrong

A caller can match on the type but not on the reason, and the public fields fix how each error is held, so adding a line number later, or a kind, means a breaking change. Making the fields private, or the types `#[non_exhaustive]`, keeps that room.

Go's errors are all `fmt.Errorf` strings that start with "ergo:", with no sentinel errors or types, so `errors.Is` and `errors.As` have nothing to find. A Go caller can't tell "not a tick" from "not base16" except by reading the message. The Go idiom is a few sentinels, like `var ErrNotOutline = errors.New(...)`, wrapped with `%w` around the detail, or one error type with a kind.

JavaScript throws plain Errors, so a caller who wants to branch reads the message. concept.md has the open question of whether a JavaScript reader should answer with a value instead. quad-style.md's own result objects, `{success: false, outcome: 'WrongPassword.'}` with a short constant to branch on, are one shape it could take.

### What a message says

Every edition's messages name the refused value, and each writes it in its own escapes: `JSON.stringify` in JavaScript, `repr` in Python, `{:?}` in Rust, and `%q` in Go. The vectors don't pin messages, so they're free to change, edition by edition.

The text outline reader's messages show the whole input, like `uneven indent in "a:\n  b:\n c:\n"`, and never which line. A reviewer measured a message of 62,929 bytes for a 2,001-line file. That's the open question named LineError: report the line number and that one line instead, like `line 3: uneven indent in " c:"`. The case against changing it is that outlines are meant to be small, so the whole input is small too. The case for it is the hand-edited text the reader is lenient for. The person who wrote the outline is the one reading the message, and a line number takes them straight there. Quoting the whole input also writes all of it into a log, which matters when the text came from outside.

### The has promise

quad-style.md promises that `has` asks and never fails, in all four languages. Outline's method named `has` breaks that: `o.has('Key')` throws for a name an outline can't have, on purpose, so a typo is loud. That's the open question named HasThrow. If `has` answers false instead, the promise holds for every function and method spelled `has`. That would also be true, since no outline can hold that name. `n`, `m`, `list`, and `remove` would keep refusing loudly.

### Panics a caller can't avoid

The rule is that a panic means a bug, and ergo never leaves a caller unable to avoid one. Two places in Rust and Go test it:

- **CapPanic.** Outline's writer panics when a name, a value, or contents would pass the span's cap of about 256 MB. No `has` asks first, and the cap isn't exported. The text reader can hand back an outline that size, so a program that reads text from outside and writes it as bytes can panic. That's the one path where input from outside reaches the panic.
- **Bin's `add`** panics when the bytes won't fit, and that's fine, since the capacity is the caller's own prediction, and `size` and `capacity` ask first.

## Iterators

ergo has no iterators yet. Nothing in it answers something to loop over. Where a family holds several things, it hands them over by index or as a list: an outline's contents come one at a time through `get(i)` up to `length()`, or as a list of the ones with a name through `list(k)`. That's enough today. This section is for the day ergo goes looking for what in the whole library could be iterable, and how to make it so in four languages at once.

### The user story

A program holds an outline and wants every outline in its contents. Today it writes the same loop in every edition:

```js
for (let i = 0; i < o.length(); i++) {
  let c = o.get(i)
}
```

An engineer in each language reaches first for that language's own loop, and none of them works:

- `for (let c of o)` in JavaScript
- `for c in o` in Python
- `for c in &o` in Rust
- `for c := range o.All()` in Go

The program also might want every outline in the tree, depth first, to find each one with a name wherever it sits. ergo already walks a tree that way inside, with `_walk` and its enter and leave, and doesn't offer it outside. When an engineer in each language read Outline with fresh eyes, three of the four asked for this.

### How each language iterates

Each of the four grew its idea of iteration in stages, and the modern form in each is newer than most code a reader has seen:

- **JavaScript** got the iterator protocol in 2015: an object with a `[Symbol.iterator]` method answering an iterator, whose `next()` answers `{value, done}`, and `function*` generators to write one. That's what `for...of`, spread, and `Array.from` take, along with destructuring. Iterator helpers came much later, as a standard library on every iterator: `map`, `filter`, `take`, `drop`, `flatMap`, `reduce`, and `toArray`, with `Iterator.from` to adopt any iterable. An async iterator, `[Symbol.asyncIterator]` with `for await`, is how a stream is read a piece at a time, which ties this section to the one on streams.
- **Python** separates an iterable, which has `__iter__`, from an iterator, which has `__next__` too. A generator with `yield` is the plain way to write one, and `itertools` is the toolbox. A container that also has `__len__` and `__getitem__` speaks the sequence protocol, and gets `len()`, indexing, and iteration together, which is what a Python engineer expects of something that holds things.
- **Rust** has the `Iterator` trait with its one required method, `next`, and dozens of adapters on top. A collection answers three ways, through `IntoIterator` for itself, for a reference, and for a mutable reference: `into_iter` takes the items, `iter` borrows them, and `iter_mut` borrows them to change. The borrow checker rules out changing a collection while iterating it. Handing out a slice, `&[Outline]`, gives all of that at once: `iter`, `len`, `is_empty`, and a `get` that answers an Option.
- **Go** ranged only over its built-in kinds, slices, maps, strings, and channels, until 1.22 added integers. 1.23 added iterators: a function of type `iter.Seq`, `func(yield func(V) bool)`, that a `range` loop drives, with `iter.Seq2` for pairs and `iter.Pull` to turn one into a `next` to call. The standard library names its methods for them `All`, `Values`, and `Backward`, as in `slices.All` and `maps.Keys`. Before that, Go code returned a copy of a slice, or a cursor with a `Next` or `Scan` method in a loop, like `bufio.Scanner` and `sql.Rows`.

### What there is to decide

- **Live or a copy.** JavaScript's own iterators over a Map are live, and Python raises if a dict changes while it's iterated. A copy is safe and costs an allocation. Rust decides it for its borrow checker, by forbidding the change.
- **The words.** Every edition should say the same word, and `contents` is the obvious one. But the native form of it differs: a method answering an iterator in JavaScript, `__iter__` and `__len__` in Python, a `contents` slice in Rust, and an `All` method answering an `iter.Seq` in Go.
- **One level or the whole tree.** Contents are one level. The whole tree depth first is the walker made public, and it would need to say each outline's depth too, and maybe its leave.
- **What order.** An outline's identity is its sorted form, and the order it holds is the order it was built or read in. Iterating should follow the held order, the way `get` does, and anyone who wants sorted order calls `sort` first.
- **Laziness and cost.** A generator answers as it goes, while a list or a slice is all at once. Outlines are small, so either is fine, but streams aren't.

### Across the library

Outline's contents and its tree are the clear cases. Other candidates for the same sweep:

- **Text:** the lines of a multiline string, which `deindent` already splits, and the pieces between tags, which `cut` finds one at a time in a loop today.
- **The fuji hashes:** the pieces and stripes a file is measured into, which `hashMeasure` answers as lists.
- **Data:** its bytes need nothing, since each platform's own byte array already iterates.
- **Streams:** they're iterators that wait, so whatever the section on streams decides lands here too.

## Streams

ergo doesn't speak streams yet, with one exception. It reads them to hash: `hashStream` reads a stream into a `FujiHasher`, `hashFile` reads a file at a few offsets, and the hashers take bytes a piece at a time through `update` and answer through `digest`. Nothing in ergo makes a stream or turns one into another. Data's forms, text, base16, base32, base62, and base64, take a whole buffer and answer a whole string, or the other way around, and Outline reads and writes whole packets. This section is about where that could change, and where it shouldn't.

### The encodings

Data's forms are built for short things: a hash in base32 for a database column, a few bytes in base62 for a query string or a location bar, base16 for looking at bytes. For those, a whole buffer at a time is the right shape, and the readers' strictness depends on it: each one writes the bytes back out and requires exactly the text it read, which needs the whole text in hand.

Speaking an endless run of base62 to the other side of a connection would be unusual. A program with bytes to send usually sends the bytes. A text encoding goes on the wire only where the channel carries nothing but text. Those channels are common, though, and base64 is the encoding that crosses them in volume:

- email attachments in MIME, base64 in lines of 76 characters (RFC 2045)
- certificates and keys in PEM, lines of 64 (RFC 7468)
- bytes inside JSON, in a terminal, or through a clipboard

So the platforms stream base64, each in its own idiom:

- **Go** has `encoding/base64`'s `NewEncoder`, an `io.WriteCloser` whose `Close` writes the last partial block, and `NewDecoder`, an `io.Reader`. `encoding/base32` and `encoding/hex` have the same pair.
- **Rust**'s base64 crate has `write::EncoderWriter`, whose `finish` encodes the leftover bytes and the padding (dropping it finishes too, but swallows any error), and `read::DecoderReader`.
- **Python**'s `base64.encode(input, output)` encodes from one file object to another, and `codecs` defines the general model: an incremental encoder and decoder fed piece by piece, told `final=True` with the last one.
- **The web platform** got `Uint8Array.fromBase64` and `toBase64` as Baseline 2025. `setFromBase64` decodes into an existing array and answers `{read, written}`, the way `TextEncoder.encodeInto` does, and its `lastChunkHandling: "stop-before-partial"` leaves a partial group unread, to come back with the next piece. That's decoding a stream in pieces, built in.

Each encoding has a group size, which is what a stream has to respect: base16 turns 1 byte into 2 characters, base64 3 into 4, base32 5 into 8. ergo's base62 is a stream of bits already, and its alphabet is even named `alphabet62Stream`. Each character takes the next 6 bits, or 4 for the three codes at the top, read from a two-byte window, so an encoder fed pieces carries less than a byte between them. Its end is the subtle part. The last character can hold bits past the last byte, and the reader refuses one whose extra bits aren't zero, so a decoder can't settle a character until it knows the stream is over. Text has the same kind of edge. A piece can end partway through a character's UTF-8 bytes, which is why `TextDecoder` has a `stream` option. And Normalization Form C can't settle a character while a combining mark might still follow, which is why Unicode's UAX #15 defines a Stream-Safe Text Format that bounds how many marks can pile up.

### What streaming would bring

ergo already has the shape for bytes that arrive over time, from the hashes: an object you make with what it needs, feed with `update`, and finish with `digest`, with each language's own streams wrapped thinly around it (quad-style.md's Factoring section, and `FujiHasher` under `hashStream`). A streaming encoder would take the same shape: `update` with each piece, answering the text it can write so far, and `digest` for the rest, the way `codecs`' incremental encoders work. That keeps the hard core a plain object with no I/O, the kind every edition can implement and a vector can pin, piece boundaries included, the way the hasher vectors already pin splits at block and chunk boundaries.

The concerns arrive with it:

- **Boundaries.** A piece can end in the middle of a group, or of a character.
- **Flush.** The last partial group, its padding, and base62's last character are written only once the end is known.
- **Strictness at the end.** Some refusals can only be made at the end, after output has already gone out. A strict reader that checks by writing everything back can't hold the whole stream to do it, so it needs a check that works a group at a time.
- **Errors partway.** What a decoder does with bytes it already answered, once it finds a bad character.
- **Backpressure.** A fast producer and a slow consumer, and who waits.
- **Cancelling**, and **line wrapping** for MIME and PEM.

Backpressure belongs to the wrappers, and each language has its own:

- **JavaScript** has two stream families. The WHATWG one, `ReadableStream`, `WritableStream`, and `TransformStream` joined by `pipeThrough` and `pipeTo`, runs in a page, a worker, and Node. Its queues fill to a high-water mark and report `desiredSize`, and a writer waits on its `ready` promise. Node's own streams signal the same thing when `write` answers false, until `drain` fires, and `Readable.fromWeb` and `toWeb` cross between the two. `TextEncoderStream`, `TextDecoderStream`, and `CompressionStream` are the platform's own transforms to follow.
- **Python** paces a blocking file object by blocking. `asyncio`'s `StreamWriter` asks a writer to `await drain()`, and a generator of pieces is pulled at the consumer's speed.
- **Rust**'s `Read` and `Write` pull and push, blocking as the backpressure, with `flush` and a wrapper's own `finish`. The async side, `AsyncRead`, `AsyncWrite`, and the `Stream` trait in tokio and futures, waits by answering `Pending`.
- **Go**'s `io.Reader` and `io.Writer` block too. `io.Pipe` joins a writer to a reader with no buffer between, `Close` flushes, and a `context` cancels.

An object you feed does no I/O, so it never has to decide any of this. Each wrapper takes its platform's mechanism as it comes: a `TransformStream` in JavaScript gets backpressure from the pipe it sits in, and an `io.WriteCloser` in Go blocks like any other. Writing one design into four stream models is the exercise ergo already does for everything else. It would be a close tour of how each language does streams today, worth taking for its own sake.

### Outline

Outline isn't meant to stream from the inside. It's a small, self-describing packet, and a program that would need to look inside an outline before the rest of it arrives has made an outline too big. The way to stream with outlines is to send many of them, one after another.

The binary form already allows that, with no framing added. An outline is self-delimiting: a reader learns where one ends from its own three spans, the name's, the value's, and the contents', so the next outline starts right after. Bencode works the same way, which is how a torrent's dictionaries need no outer length. Protocol Buffers messages don't, and that's why a stream of them prefixes each with its length, the way protobuf's `writeDelimitedTo` does, and the way netstrings and BitTorrent's peer messages carry a length up front.

What ergo would add is a reader for the run. `dataToOutline` reads exactly one outline and refuses bytes after it. An outline stream reader would be one more object you feed: `update` it with pieces as they arrive, and it answers each outline as soon as it's whole, keeping the partial one. It's also the right place for the size limit a program reading from the wire needs. The root's spans announce its sizes in its first few bytes, so the reader can refuse an outline bigger than the caller allows before it holds any of it. That's earlier and cheaper than capping the bytes after they arrive, which is what the span essay advises today.

Writing a run of outlines needs nothing new: one `data()` after another. The text form doesn't run together the same way. Its reader refuses a second root, and blank lines and comments are skipped inside an outline, so a run of text outlines would need a separator of its own, which nothing has needed yet.

# icarus migration

This file is for later: the day a project that calls icarus's helpers switches to calling ergo's instead. It isn't about how ergo is built.

Two migrations share the word. The first is happening now. ergo is bringing in icarus's helpers, the JavaScript in the application ergo grew out of, family by family, and as each family comes over, some of it changes, since what fits a JavaScript application on the web doesn't always fit a library in four languages: an answer here, a refusal there, a helper left behind. CLAUDE.md's flow describes that work. The second comes later, once a family is in ergo: a project that has been using icarus, the application ergo grew out of among them, replaces its calls to icarus with calls to ergo. Each change the first migration makes is a landmine for the second, so each is recorded here as it's made, family by family, for checking a project's call sites as it moves, while it leans on its own tests.

ergo's comments and documentation describe ergo as it is, and leave the differences to this file. Each claim was checked against icarus's own code, core.js and level0.js, in October 2026, most of them by running it beside ergo's. icarus is JavaScript, so this file uses the JavaScript edition's names.

## Everywhere

- **Failing.** icarus fails through `toss`, which throws a `TossError` carrying a short kind, like `'data'` or `'bounds'`, and the variables it watched. ergo has no custom throw helper and no custom error class: it throws a plain `Error` whose message names the values involved. Code that catches `TossError`, or reads a kind or a watched variable from one, changes.
- **One module.** icarus splits its helpers between core.js and level0.js. ergo is the one import `@zootella/ergo`, including the helpers that came from level0.js, like `checkInt` and `sayTick`.

## int

`checkInt` is the one to look at in every call site.

- icarus's `checkInt(i, m)` takes a minimum, zero unless given, so `checkInt(-1)` throws there and `checkInt(n, 1)` refuses zero. ergo's `checkInt(i)` has no minimum. It passes any integer every edition holds exactly, negatives included, and ignores a second argument without a word. Where a call meant a count, write `if (!(hasInt(n) && n >= 0)) throw new Error(...)`, with the call's minimum in place of the zero.
- `minInt(i, m)` becomes `hasInt(i) && i >= m`.
- ergo's `checkInt` returns `i`, so it can wrap a value in place. icarus's returned nothing.

## text

- `hasTextOrBlank` and `checkTextOrBlank` become `hasString` and `checkString`: any string, blank included.
- `hasText` and `checkText` keep their meaning, a string with something besides whitespace, but whitespace is now Unicode's White_Space list rather than JavaScript's `trim`. The two differ at two characters. A byte order mark alone, `'\ufeff'`, is text in ergo and wasn't in icarus. The next line control alone, `'\u0085'`, was text in icarus and isn't in ergo.
- `trim(s)` is new, and differs from `s.trim()` at the same two characters: it keeps a byte order mark, and takes off U+0085.
- **A byte order mark survives trim.** Code that reads a file's bytes, like a CSV a person exported from Excel and uploaded, and trims its first line got the first column's name clean from `s.trim()`, which took the invisible mark off by accident. ergo's `trim` keeps it, so moving that code to ergo, or to ergo's `hasText`, means taking off a leading byte order mark on purpose.
- **cut can cut at whitespace.** `cut`, `cutLast`, and `cut2` take any tag of at least one character, so `cut(s, ' ')` and `cut(s, '\n')` work in ergo. icarus refused a space, a tab, or a newline as a tag, since its `checkText` called them blank. A blank tag throws in both.
- `cutAfterLast` is the same, and its message names the tag and the text.
- `deindent` is the same, except the whitespace it takes off the end of each line is Unicode's list, so a line ending in U+FEFF keeps it, and one ending in U+0085 loses it.
- Left behind: `replaceAll` and `replaceOne`, for the platform's own. `replaceOne(s, a, b)` is `s.replace(a, b)` with a string `a`, which is what it called. But `replaceAll(s, a, b)` is `s.split(a).join(b)`, not the platform's `s.replaceAll(a, b)`, which reads `$&`, `$1`, and `$$` in the replacement as patterns, where icarus put the replacement in as written. And icarus's two refused a blank or whitespace tag or replacement, so `replaceAll(s, 'a', '')` threw there, while the platform's take them.
- Left behind: `safefill`, icarus's tag for text that's about to be hashed, and `checkSafeFill`, its rule, since they guard the application's own hashed messages and nothing in ergo needs them. A project that uses them keeps its own.

## Size

- `Size` has the same names and the same values.
- `saySize4` says exactly one byte as `1 byte`, where icarus said `1 bytes`. It refuses a negative or a fraction, where icarus said `-5 bytes`, and `1 bytes` for 1.5. Ask `hasInt(n) && n >= 0` about a number from outside first.
- Left behind: `sayNumber4`, `sayPlural`, and `sayHugeInteger`.

## Time

- `Time` has the same constants and the same values.
- `Time.months`, with its `zeroToJan`, `oneToJan`, `janToZero`, and `janToOne`, is left behind. ergo has `monthNames`, the full names in title case, so icarus's `Time.months.zeroToJan[m]` is `monthNames[m].slice(0, 3)`, and its lowercase lookups go through `toLowerCase`.
- `inSeconds(t)` is left behind. It's `t / Time.second`.

## tick

- `tickToText` and `textToTick` write and read the same text, and refuse the same ticks. icarus refused a tick before 1970 with `checkInt`, since it's negative, and one past 9999 with its round trip; ergo refuses both with `hasTick`, which a caller can now ask first.
- **sayTick is a different function with the same name.** icarus's, in level0.js, takes only a tick. It shifts the tick to the page's local time, says `(not yet)` for zero or a missing tick, and writes one fixed shape, like `Mon05:17a57.300s`. ergo's `sayTick(t, template)` writes UTC in whatever shape its template describes, and throws without one. The template `Dddhh12:mmass.ttts` writes icarus's shape exactly. Shift the tick by the local offset first, and check for zero or a missing tick before calling.

## Tag

- `Tag`, `hasTag`, `checkTag`, and `tagLength` are the same.
- `checkTagOrBlank(s)` is left behind. It's `if (s !== '') checkTag(s)`.

## Data and Bin

- **A Data can be empty.** icarus meant a Data never to be. It refused blank text, blank base16, base32, base62, and base64, `{random: 0}`, a clip of nothing, and `Bin(0)`. But an empty array or buffer, or a Bin's `data()` before anything was added, made an empty one anyway, and on it `size`, `text`, `base16`, `base32`, `base64`, and hashing all answered as usual. In ergo every one of those makes or holds an empty Data. So wherever a project leaned on a Data having at least one byte, it asks for itself, with `d.size() > 0`. The guarantee had holes in icarus, so those checks are worth writing anyway.
- **Methods need their Data.** A Data in ergo is a class behind the plain function `Data`. So `let { base16 } = d`, `promise.then(d.text)`, and `{ ...d }`, which worked in icarus, where each method carried its Data along, don't work here. Call the method on the Data.
- **Text with a byte order mark.** icarus refused bytes that begin with one as text, since its decoder dropped the mark and the round trip noticed. ergo reads the mark as the character U+FEFF, so the text round-trips.
- **Large random requests.** icarus's `Data({random: n})` threw past 65,536 bytes, Web Crypto's limit for one call. ergo fills a larger request in steps.
- **Two forms at once.** Given both an array and a buffer, icarus took the buffer and ergo takes the array. Neither was meant to be given more than one.
- **base62 of no bytes** writes blank in ergo. icarus threw, since its round trip read the blank back through a `Bin(0)`.
- Left behind:
  - `d.view()`, which is `new DataView(d.array().buffer, d.array().byteOffset, d.size())`.
  - `d.hash()`, which is `await sha256(d)`.
  - Bin's `clipView` and `clipCopy`, which are `b.data().clipView(start, end)` and `b.data().clipCopy(start, end)`.
  - Bin's `hash()`, which is `await sha256(b.data())`, once the code has checked `b.size() == b.capacity()`, the check icarus's made.
- `intToBase62` and `base62ToInt` haven't come over yet.

## Hash

- `hashData(d)` is `await sha256(d)`, with the same bytes.
- `hashText` gives the same answers, and answers for blank text too, which icarus refused.
- Code that fed a Bin and then hashed it can feed a hasher instead: `Sha256()`, then `update` with each piece and `digest` for the hash, with no capacity to predict. No hasher in ergo speaks icarus's `add` and `hash`.
- Left behind: `hash_size` and `hash_length`, which are 32 and 52; `checkHash` and `checkHashOrBlank`; `hashObject`; and `random32`.

## Fuji hashes

- `hashFile`, `hashStream`, and `hashProtocol` take the same arguments and give the same answers. ergo's vectors pin icarus's own answers.
- The status objects time themselves with `Date.now()`, not icarus's `Now()`, so a simulated clock doesn't move them.
- `hashStream` calls `onProgress` once for each read that completes one or more pieces, and once at the end. icarus called it once for every piece, the last fragment included, and then at the end, so a read that completes two pieces made two calls there and makes one here. And icarus also checked the abort signal before hashing each piece, where ergo checks before each read and before the final hashes.

## Outline and quoted encoding

- `Outline`, `textToOutline`, `dataToOutline`, `dataToQuoted`, and `quotedToData` keep their names and their shapes.
- **Blank lines in a text outline** are the lines `hasText` calls blank, by Unicode's whitespace, where icarus used JavaScript's `trim`. So a line that is only a byte order mark is no longer skipped, and fails for having no colon, and a line that is only U+0085 is now skipped.
- **The readers check what they're handed.** `textToOutline` and `quotedToData` refuse anything that isn't a string, where icarus's `quotedToData(5)` quietly answered null. `dataToOutline` and `dataToQuoted` refuse anything that isn't a Data, with an Error that says so.
- **A value is always a Data.** icarus's `o.value()` answered null for an outline with no value. ergo's answers a Data of no bytes, so `if (o.value())` is always true here, and code that asked it asks `o.value().size() > 0` instead. `o.value(null)` still clears a value, and setting an empty Data is the same as clearing one.
- **Quoted encoding of nothing.** `quotedToData('')` answers a Data of no bytes, where icarus answered null. `dataToQuoted` takes only a Data, so `dataToQuoted(null)`, which icarus answered with blank, throws.
- **The writers leave the outline alone.** icarus's `o.data()` sorted the outline in place before writing it, so afterward the outline was no longer in the order a project built it, and `o.text()` wrote whatever order it found. ergo's two writers each sort a copy and write that, so both write the sorted form, and the outline keeps its order. Code that read `o.get(i)` after `o.data()`, counting on the sort, calls `o.sort()` first. And text icarus wrote from an unsorted outline comes out in a different order from ergo.
- **The binary reader refuses contents out of order.** icarus's `dataToOutline` read the contents of each level in any order. ergo's throws unless each level is in the order its writer sorts to, so the same information is always the same bytes. Two siblings the same are fine. icarus's `o.data()` always wrote sorted bytes, so bytes it wrote read the same here, and only bytes made some other way can fail.
- **0 and false aren't the blank name.** icarus's name setter and lookups took any falsy value as no name, so `o.list(0)` listed the unnamed contents. ergo takes only undefined, null, and blank, and throws for 0 and false.
- New: `hasOutlineName` and `checkOutlineName` ask about and enforce a name from outside. And `spanEncode` and `spanDecode`, private in icarus, are public: `spanEncode` answers a Data, and `spanDecode` reads from one.
- **get takes an index.** `o.get(i)` throws for anything but a whole number from 0 to one less than `o.length()`. icarus's answered undefined for `o.get(0.5)`, and the first outline for `o.get('0')`.
- **A comment holds any character.** A `#` comment runs to the end of its line, whatever it holds. icarus refused a comment holding a carriage return, a line separator, or a paragraph separator, since the dot in its pattern doesn't match them.
- **Long names read from binary.** `dataToOutline` reads a name of any length. icarus's spread the name's bytes into one call, which threw a RangeError for a name past about a hundred thousand bytes in Node.
- **A line of only a comment is skipped.** In a text outline, a line that holds only spaces and tabs and then a `#` is skipped the way a blank line is, at any indent, so a note above a block or a line commented out reads fine. icarus refused it for having no colon.
- **sort checks the tree first.** `o.sort()` refuses an outline that holds itself, or holds the same outline in two places, the way the writers do. icarus's sort ran out of stack on a cycle, with a RangeError, and sorted a shared subtree twice without a word.
- **Methods need their outline.** An outline in ergo is a class behind the plain function `Outline`, like Data. So `let { text } = o`, `promise.then(o.data)`, and `{ ...o }`, which worked in icarus, where each method carried its outline along, don't work here. Call the method on the outline.
- **add takes only an outline ergo made.** `o.add(x)` adds `x` as an outline only if ergo's `Outline` made it, since `sort` reaches into an outline's private contents. icarus took anything whose `type` was `'Outline'`, so a project moving over adds ergo's outlines to ergo's, rather than mixing in icarus's. A `null` throws an Error that says what `add` takes, where icarus threw a TypeError.
- **The readers have limits.** `dataToOutline` and `textToOutline` refuse an outline past 4 MiB, a text outline past 4 MiB as UTF-8, one nested more than 256 levels below the root, or one with a name past 2 KiB, unless the caller passes larger limits as a second argument, like `dataToOutline(d, {size: 64 * Size.mb})`. icarus's readers had none. The defaults are in `outlineLimits`.

# tag

What a tag is worth against a UUID, where its randomness comes from in each edition, and what an experienced engineer would say about it in a meeting. Written in October 2026, when Tag came back into ergo after the exceptions question was settled, so the answers are on file before anyone asks.

## The design

A tag is 21 characters from an alphabet of 62: the numerals and the letters in both cases. That's nanoid's default length and nanoid's default alphabet with the hyphen and underscore removed, so a double-click in a terminal, a browser, or Slack selects the whole tag. Every character is drawn uniformly from the operating system's cryptographic random source. A tag is a random identifier like a UUID v4, not a sortable one like a UUID v7, and that's a choice, below.

## The numbers

| Identifier | Random bits | Characters |
|---|---|---|
| UUID v4 | 122 | 36 |
| nanoid default, 64 symbols | 126 | 21 |
| ergo tag, 62 symbols | 125 | 21 |

Twenty-one characters from 62 symbols carry 125 bits, three more than a UUID v4 in a string less than two thirds the length. The birthday bound at 125 bits is about four quintillion tags for an even chance of a single collision. nanoid's collision calculator puts it another way: at a thousand tags an hour, about 107 billion years for a one percent chance. Dropping the two symbols cost nothing anyone will notice.

## Where the randomness comes from

Each edition draws from the same source that its ecosystem's own UUID library draws from, and the test is that an experienced engineer would expect exactly this.

| Edition | ergo's source | What the standard UUID library there uses |
|---|---|---|
| JavaScript | `crypto.getRandomValues`, Web Crypto, a global in browsers and Node 22 | the same, in both `crypto.randomUUID` and the uuid package |
| Python | `secrets.token_bytes`, which is `os.urandom` | `uuid.uuid4` reads `os.urandom` |
| Rust | the getrandom crate, since the standard library's random source is still unstable | the uuid crate's v4 draws through getrandom |
| Go | `crypto/rand.Read`, which since Go 1.24 never fails | google/uuid reads `crypto/rand` |

## How the bytes become a tag

The algorithm is nanoid's, the same in all four editions. Draw 35 random bytes at once. Mask each down to six bits, 0 through 63. Accept 0 through 61 as a character of the alphabet and skip 62 and 63. Stop at 21 characters, and draw again if the 35 bytes didn't yield enough, which happens about once in ten trillion tags.

Skipping the two values past the alphabet's end is rejection sampling, and it's the standard way to map uniform bytes onto an alphabet whose size isn't a power of two without bias. It's what Python's secrets, Go's crypto/rand.Int, Rust's rand, Java's SecureRandom, and libsodium do. The alternative that never skips, a byte modulo 62, is the one reviewers flag: 256 isn't a multiple of 62, so the first few letters of the alphabet come out measurably more often. The uniformity of ergo's version can be read off the implementation in three lines: the mask covers 0 through 63, every value below 62 maps to one character, and every value at or above 62 maps to none. No test is needed to see it, and the two percent of bits thrown away cost nothing from a source that produces gigabytes a second.

One rule of the smoke test: a tag is mostly a call to the random source, so an edition that calls more often than it needs to shows up at once as the slow one. The first Python draft chose each character with its own call and made tags at a fourteenth of JavaScript's rate; one draw per tag made it seven times faster.

## The meeting

A junior engineer proposes tags instead of UUIDs: shorter, double-clickable, the same in every language of the stack, and no meaningful loss of uniqueness. A senior engineer says "look at their implementation." Here's what they could say, in order of how much it deserves to land.

**Random primary keys fragment the index; use UUID v7.** The one real objection. A v7 or ULID carries a timestamp first, so rows insert in order and primary-key B-trees stay tight under heavy writes. A tag is the v4 kind of identifier, random throughout, and that's a choice made on purpose, not a flaw to fix. If a team wants sortable identifiers, that's a different kind of tag, with the tick in front, and ergo could offer one later. The answer in the meeting is "ours is the v4 kind, deliberately."

**Our collation is case-insensitive.** MySQL and SQL Server fold case by default, so to those databases a 62-symbol alphabet is 36 symbols. That still leaves 108 bits, which is still beyond any real system's reach, so nothing breaks. But tags belong in a case-sensitive or binary column, and the site should say so. Postgres and SQLite are case-sensitive already.

**The uuid type is 16 bytes; a tag is 21 bytes of text.** True. A native UUID column is smaller and compares as binary. A tag compares as text and can be read in a Slack message and double-clicked. That's the trade being proposed, and it's a fair one to make knowingly.

**That loop can spin forever.** If the random source fed exactly 62 or 63 forever, the draw would never finish. No standard library bounds its loop, because the operating system's generator is the root of trust for the whole process: if it's adversarial, it controls every byte, and a hang is the mildest harm it can do. OpenSSL's range sampler does give up after a hundred iterations, as a defense against a broken source rather than a malicious one. ergo matches the standard libraries and leaves the loop unbounded, with the reasoning here; a cap would be one comparison and a branch no working generator can ever reach, in four places.

**Show me it's uniform.** Three lines of the implementation show it, above. A frequency count over the millions of tags the fuzz loop already makes would show it statistically, within two percent, and would catch a mistyped mask that no other test sees. It isn't in place; it's the cheapest thing to add if the question ever comes up in earnest.

**The weak ones.** Confusable glyphs like 0 and O matter for identifiers people read aloud or retype; tags are copied and double-clicked. A tag can begin with a digit, which matters only in a CSS selector. A tag of all numerals is possible at odds of one in fifty quadrillion. "Just use randomUUID" is the format question, and the format is the point.

## Decided

- A tag is the random kind of identifier, like UUID v4. A sortable kind is a different function, if it's ever wanted.
- The algorithm is nanoid's, identical in all four editions, with one draw of 35 bytes per tag and rejection of the two values past the alphabet.
- The draw loop is unbounded, like every standard library's. The reasoning is above, so the decision doesn't have to be made again.
- The site's Tag page should carry the facts a senior engineer would ask for: 125 bits against a UUID's 122, the four sources, the case-sensitive column, and random rather than sortable.

## Later

- A sortable tag, the tick followed by random characters, for teams whose primary keys need insert order.
- A symbol frequency check in the fuzz loops, if proof beyond reading the code is ever wanted.

# parse, don't validate

Notes on a pattern ergo's has and check pairs are reaching for, written in October 2026 to return to later. Nothing here is in ergo yet. It records the idea, where it comes from, how far each of ergo's four languages can take it, and what adopting it would cost.

## The wish

A function like `tickToIso` needs a tick: an integer from zero through the last millisecond of 9999. A function like `cut` needs a tag that isn't the empty string. ergo checks these at runtime, with a has that asks and a check that enforces, at the top of every function that cares. That works, but the check runs again at every gate the value passes through, and nothing in the program remembers that it already passed.

The wish is a type that carries the check with it. Declare a variable as text that doesn't vanish or change when trimmed, and every function that takes one knows, without asking again, that it holds. Every new strongly typed language raises the hope, and most disappoint it: they check that a value is a number or a string, its shape, but not what's in it.

## What exists

The idea has a name, _refinement types_: a type plus a predicate, like a string whose trim is itself and isn't empty. Languages reach for it at three levels.

### The compiler proves the predicate

**Ada** is the most production-ready. Since Ada 2012 a subtype can carry a predicate, and every assignment and conversion checks it at runtime:

```ada
subtype Trimmed_Text is String
   with Dynamic_Predicate => Trimmed_Text'Length > 0
      and then Trim (Trimmed_Text, Both) = Trimmed_Text;
```

SPARK, Ada's proven subset, can go further and prove at compile time that a check never fails, so the runtime test can come out.

**Liquid Haskell** adds refinements to Haskell, like `{v:String | len v > 0}`, and checks them with an SMT solver as the program compiles. **F\***, from Microsoft Research and Inria, is built around refinements. **Flux** is a research project bringing them to Rust. **Idris**, **Agda**, **Lean**, and **Rocq** have dependent types, where any predicate at all can be a type, and the programmer supplies the proof.

### A library attaches the predicate to a type

Scala's **refined**, which began as a Haskell library, writes the wish almost word for word:

```scala
type Text = String Refined (NonEmpty And Trimmed)
```

`Trimmed` means no leading or trailing whitespace. A value gets the type only by passing the check, at runtime for a value from outside and at compile time for a literal. Scala 3's **iron** does the same, more lightly.

### The newtype, which works everywhere

Alexis King's essay "Parse, don't validate," from 2019, makes the case for the pattern every mainstream language can use. _Validating_ checks a value and throws the evidence away, answering a boolean and leaving the value as loosely typed as it arrived. _Parsing_ checks a value and returns something of a more precise type, which carries the evidence with it. Do the parsing once, at the boundary where values arrive, and everything inside works with types that can't hold a bad value.

The mechanism is a _newtype_, or _smart constructor_. Wrap the plain value in a type of its own, and make the checking function the only way to get one. The check is still runtime code, but it runs in one place, and every function that takes the type knows the value already passed. The compiler can't prove what trim means, but it can prove that the check happened.

## ergo's pairs, seen this way

A has is validation: it asks, answers a boolean, and leaves the value as it was. A check is validation too, passing the value through unchanged, still the plain type. The pattern would add the third piece, a parse that answers the precise type.

ergo already does this in one place. `Data({ base16: s })` takes loosely typed text, checks it exactly, and answers a Data, a type that holds bytes and means those choices were made. Every function that takes a Data trusts it. The readers parse too, `textToTick` from text to a tick, but in JavaScript a tick is still just a number afterward, so the evidence is lost on the way out.

In Rust and Go, the pattern would also retire a rule. ergo says a panic means a bug, so every kind of value a function refuses has a has beside it, which lets a caller ask first. If `tickToIso` took a `Tick` rather than an integer, it couldn't panic at all, since no `Tick` can hold a bad value. The has and check would move into the type's constructor, which refuses by returning an `Option` or an error, the ordinary way.

## In ergo's four languages

### Rust

The strongest of the four. A private field means no code outside the module can make one except through the constructor:

```rust
pub struct Tick(i64); // the field is private, so outside this module the only way to get a Tick is Tick::new

impl Tick {
    pub fn new(t: i64) -> Option<Tick> {
        if has_tick(t) { Some(Tick(t)) } else { None }
    }
    pub fn get(&self) -> i64 {
        self.0
    }
}

pub fn tick_to_iso(t: Tick) -> String // can't panic, since no Tick holds a bad value
```

`TryFrom<i64>` is the idiomatic spelling of the constructor, and `AsRef<str>` keeps a text newtype easy to read from.

### Go

Nearly as strong. An unexported field keeps other packages from setting it:

```go
type Tick struct{ ms int64 } // other packages can't set ms

func NewTick(ms int64) (Tick, bool) {
	if !HasTick(ms) {
		return Tick{}, false
	}
	return Tick{ms}, true
}
```

There's one hole. Every Go type has a zero value, and any package can declare `var t ergo.Tick` without calling `NewTick`. For a tick that happens to be fine, since zero is the first millisecond of 1970. For a text type it isn't, since the zero value is the empty string. Such a type has to treat its zero value as never checked, or be designed so the zero value is valid.

### Python

Static only. `typing.NewType` makes a type for the checker that disappears at runtime, and `TypeIs` turns a has into one the checker understands. TypeIs arrived in Python 3.13, and before that it comes from `typing_extensions`:

```python
Text = NewType("Text", str)

def has_text(s: object) -> TypeIs[Text]: ...  # after `if has_text(s):`, a type checker knows s is Text
```

mypy and pyright enforce it, and Python itself never does. A class with the check in its constructor gives a runtime guarantee, at the cost of no longer being a plain `str`.

### JavaScript

Here the language itself offers nothing static, and at runtime only an object can carry proof. A string is a primitive, so there's no way to mark one as checked: no property to set, no WeakSet to put it in, and no telling a checked `'abc'` from an unchecked one. So the runtime version is a wrapper object, which is what Data already is for bytes. Since ES2022, the brand on such an object can't be forged:

```js
class _Text {
  #text
  constructor(s) {
    this.#text = checkText(s) //the one place the check runs
  }
  text() {
    return this.#text
  }
  static has(o) {
    return typeof o == 'object' && o !== null && #text in o //true only for an object this class made
  }
}
```

`#text in o` is an _ergonomic brand check_: nothing outside the class can make an object pass it, unlike a `type: 'Data'` tag, which anyone can write. The costs are real. The value stops being a string, so `+`, `===`, Map keys, and JSON all need `.text()`. Each one is an object, about 82 bytes, like a Data. And two copies of ergo in one application each have their own class, so a Text from one fails the other's check, which is why Data identifies itself with a tag instead.

Strings being immutable has a quiet upside, though. A check on a string never goes stale, so asking `hasText` again is always correct, only repeated work. For a cheap check, asking again is usually the right answer, and the proof matters most for expensive checks, or for objects that can change after they're checked.

### TypeScript, the zero-cost layer for JavaScript

TypeScript has ergo's pairs as language features. A has is a _type predicate_ and a check is an _assertion function_, and a _branded type_ is a newtype that costs nothing at runtime:

```ts
declare const textBrand: unique symbol
export type Text = string & { readonly [textBrand]: true } // a plain string at runtime; the brand exists only for the compiler

export function hasText(s: unknown): s is Text // after if (hasText(s)), the compiler knows s is Text
export function checkText(s: unknown): asserts s is Text // after checkText(s), it knows too
export function tickToIso(t: Tick): string // a caller can't hand it an unchecked number
```

The value stays a string or a number, so nothing about the JavaScript changes. This fits a plan construction.md already has: a hand-written `ergo.d.ts` beside ergo.js. ergo.js would stay plain JavaScript, TypeScript users would get the brands, and plain JavaScript users in VS Code would get most of it in their editor hints.

## What it buys: the preamble, gone

In plain JavaScript, every scope that cares opens with the same check, because nothing remembers that the value passed one already:

```js
function greet(name) {
  checkText(name)
  return `hello ${name}`
}
function save(name) {
  checkText(name)
  database.put(name)
}
function handle(input) {
  checkText(input) //the same text, checked three times on one path
  greet(input)
  save(input)
}
```

With a branded type, the check happens once, where the value arrives, and every function past that point says what it needs in its signature instead of its first line:

```ts
declare const textBrand: unique symbol
type Text = string & { readonly [textBrand]: true } // a plain string at runtime; the brand exists only for the compiler

function hasText(s: unknown): s is Text {
  return typeof s == 'string' && /[^\p{White_Space}]/u.test(s) // the type and the value, in one question
}
function checkText(s: unknown): asserts s is Text {
  if (!hasText(s)) throw new Error(`not text: ${s}`)
}

function greet(name: Text) {
  return `hello ${name}` // no preamble: the parameter's type says the check already passed
}
function save(name: Text) {
  database.put(name)
}
function handle(input: unknown) {
  checkText(input) // the one check, at the boundary; from here on the compiler knows input is Text
  greet(input)
  save(input)
}

greet('bob') // a compile error: a plain string isn't a Text, so nobody can skip the check by accident
```

Four things make it work:

- **`unknown` at the boundary forces the check there.** TypeScript won't let a value of type `unknown` be used as a string until something narrows it, so forgetting the check at the door is a compile error, not a missing line.
- **One check covers the type and the value.** `checkText(s: unknown): asserts s is Text` tests `typeof` and the contents together, and afterward `s` has gone from `unknown` to `Text`: both halves of the old preamble in one call.
- **The has works in an `if`,** which fits the ordinary answers quad-style.md wants at an edge:

  ```ts
  if (!hasText(input)) return { success: false, outcome: 'NameRequired.' }
  greet(input) // input is Text here
  ```

- **It costs nothing at runtime.** A `Text` is still a plain string, with no wrapper, no `.text()`, and `+`, `===`, and JSON working as they always do.

The limit is that the guarantee is only as strong as the compiler's view. A plain JavaScript caller, an `as Text` cast, `any`, and the result of `JSON.parse` all slip past it. So the real trust boundaries still check at runtime: request bodies, user input, and responses from other services. They just check once, and the types carry the answer inward. It's quad-style.md's walled city, with the compiler keeping the promises the gates made.

That also says where ergo itself stands. ergo's public functions are a boundary, since a plain JavaScript caller can hand them anything, so they keep their runtime guards even with an `ergo.d.ts`. The saving lands in the code built on top of ergo: an application written in TypeScript against branded types would check at its own edges, then pass `Text` and `Tick` inward with no preambles at all.

## Approved once, all the way down

The runtime check happens once, at the boundary, and then the value is approved for as deep as the call stack goes. Each function below declares its parameter as `Text` and does no runtime work, and at every call site the compiler confirms the argument is a `Text`, which happens only as the code compiles and costs nothing as it runs:

```ts
function handle(input: unknown) {
  checkText(input) // the only runtime check on this whole path
  render(input)
}
function render(name: Text) {
  layout(name, 3) // no check: the compiler already proved name is Text
}
function layout(name: Text, depth: number) {
  if (depth > 0) layout(name, depth - 1) // ten calls deep or a hundred, still no check
}
```

The approval travels through anything that keeps the type: variables, return types, object fields, `Text[]`, `Map<Text, …>`, and `Promise<Text>`. A function can return a `Text`, so a value approved in one module stays approved in another.

Immutability is what makes this safe. A JavaScript string can't change after it's made, so a `Text` that passed its check can never stop passing it, and reassigning a `let name: Text` needs another `Text`.

Where the approval ends is principled. Anything that makes a new string gives back a plain `string`:

```ts
let a = name.trim() // string: the trim could have emptied it
let b = name.slice(1) // string: one character in, nothing may be left
let c = name + '!' // string: the compiler can't know the result still qualifies
```

Each of those is a new value, so it needs its own check if the check matters. Where a derivation is known to keep the property, it's written once as a trusted helper, like `function exclaim(s: Text): Text { return (s + '!') as Text }`, so the cast lives in one reviewed place rather than scattered through the code. The approval also ends wherever the type does: `JSON.parse`, storage, `postMessage`, `any`, or a library that answers a plain `string`. Each of those is the next boundary, and it checks once again.

Objects are the caution. An object can change after it's checked, and TypeScript's `readonly` holds only at compile time, so a branded object should be immutable in fact, frozen or built fresh, or its approval can go stale while its type says it hasn't.

## Brands are a pattern, not a feature

TypeScript has no feature called brands. A brand is a pattern built from ordinary parts that have been in the language for years:

| Part                                                                           | Since                |
| ------------------------------------------------------------------------------ | -------------------- |
| Intersection types, `string & { ... }`                                         | TypeScript 1.6, 2015 |
| Type predicates, `s is Text`                                                   | TypeScript 1.6, 2015 |
| `unique symbol`, so nobody can fake the brand by typing the same property name | TypeScript 2.7, 2018 |
| Assertion functions, `asserts s is Text`                                       | TypeScript 3.7, 2019 |

Before `unique symbol`, the brand was a string property, like `string & { __brand: 'Text' }`, and the TypeScript compiler's own source brands its file paths that way, with `type Path = string & { __pathBrand: any }`.

Without the pattern, TypeScript doesn't give this. It's structural: two types with the same shape are the same type, so a plain alias protects nothing:

```ts
type Text = string // just another name for string
function greet(name: Text) {}
greet('') // compiles fine: any string is a Text
```

So a typical project writes `name: string`, the compiler confirms it's a string and nothing more, and every layer that cares about the value checks again at runtime, or doesn't. The intersection is what breaks the sameness: `string & { readonly [textBrand]: true }` has a property no plain string has, so only what went through `hasText` or `checkText` can be one. The TypeScript team has had a request for real nominal types open since 2014, issue #202, and the ecosystem carries the pattern meanwhile: Zod's `.brand()` turns a schema's parse into a branded type, type-fest has it as `Tagged`, and Effect has a `Brand` module. It's well known to people who go looking for it and not the default, mostly for the friction at the edges: every boundary has to parse, libraries answer plain `string`, and one careless `as Text` quietly undoes it.

## The appeal: a few fundamental types

This is a reason to look at TypeScript that has nothing to do with modeling an application's world in classes, a `User` object with a type for every field and a layer of ceremony around each. It's for a handful of fundamental kinds of value, the low level ones every program trades in and keeps checking: text that doesn't change or collapse when trimmed, a tick, a tag, a count. Those are exactly the kinds ergo already has a has and a check for, so the brands would be few, small, and made only by ergo's own guards.

## What adopting it would cost, and what to decide

- **The four editions would differ in strength.** Rust and Go would be enforced by the compiler, TypeScript and Python by their type checkers, and plain JavaScript and plain Python not at all. ergo already lets each language decide how a failure looks. This would let each decide how strong a guarantee is, too.
- **Every caller parses at its boundary.** That's the point of the pattern, and also its weight. Code that has a number from a clock calls `Tick.new` or `NewTick` once and handles the refusal there, rather than calling `hasTick` before each use.
- **The plain functions could stay.** Typed versions could arrive beside the existing ones, so callers move over when they want, or replace them in a minor release while ergo's major version is zero.
- **The vectors wouldn't notice.** They carry plain JSON values, and each dispatch table already converts them for its edition, so the table becomes the parse point.
- **The first candidates** are the kinds ergo already guards: `Tick`, `Tag`, `Text`, and an outline name when Outline arrives. Data is the precedent.

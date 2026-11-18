# typescript, the good parts

Notes from November 2026 on one question: can a JavaScript project that stays away from TypeScript, for its ceremony and brittleness, still let TypeScript in for one job? That job is a few granular types, made as brands, that remember a guard already passed. The answer turned out to be no. Brands get partway: one check where a value is made, no re-checks past it, and a forgotten check caught before the code runs. But a type never runs, so past the checker the protection is gone, where the guard at the top of a function protects every call from anywhere. So ergo keeps its has and check pairs, and code built on it keeps its guards. These notes record the question, the research behind it, the design that was tried, an experiment that shows where it stops, what does deliver the wish, and the decision. parse-dont-validate.md is the neighbor: it follows refinement types and the newtype pattern through all four of ergo's languages. This file stays in JavaScript and asks how far TypeScript can be let in without taking over.

The title borrows from Douglas Crockford's _JavaScript: The Good Parts_, from 2008, which taught a generation to use a language by choosing a subset of it. No TypeScript book has that title. But TypeScript has now been king long enough to ask Crockford's question of it: is this king good, and which parts of it are?

## The question

Could a full stack JavaScript web app, designed and scaffolded from scratch today to work well with coding agents, use TypeScript to define a few granular types as brands? And could it get them without the rest of TypeScript's weight, verbosity, brittleness, dogma, and ceremony?

The types in question were four:

- **Text**: a string that trimming wouldn't change or empty. "hello" is Text, and "", " ", and " hello" aren't. That's stricter than ergo's hasText today, which takes " hello" as text, since it asks only for something besides whitespace.
- **Int**: an integer every edition of ergo holds exactly, a whole number within plus or minus 2^53 - 1, which is what hasInt asks.
- **Min0**: an Int that's zero or more, like a count.
- **Min1**: an Int that's one or more, like a quantity someone asked for.

Min0 and Min1 are icarus's `minInt(i, m)` given names. ergo split that function into hasInt and an inline `n >= 0` (icarus-migration.md has the change), and the brands would bring the minimum back as two named kinds of value.

The motivation is the guard pattern in quad-style.md, as it lives in the application ergo grew out of. A Nuxt endpoint receives a post from a page and runs a first sanity check on what arrived. That check doesn't prove the request makes sense for this user and this state; more specific checks come later. But it catches mistakes, and it's the first line of defense against abuse. So the tops of functions at the boundaries are lined with guards: checkText appears more than fifty times in the application, and its check functions together about 130 times. The wish was to mark a parameter as Text instead. The value would get checked once, where it arrives, and every function past that point would know without asking again.

Two things were explicitly not wanted. One is types for the application's own ideas, like a User or an Email. Modeling an application in types is never impossible, but it gets verbose and brittle right away, and every change to the idea becomes a change to the type. The other is end-to-end type safety, where one schema's types are inferred from the database through the server to the page. The application wants flexibility instead, so the schema and the front end can each change a great deal without being locked together and changed in lockstep.

## Why teams stayed away

The concern goes like this. A TypeScript project accumulates ceremony and brittleness that the safety doesn't pay for. A change anywhere ripples full stack, from a component that renders a user, through the client and the server, down to the database schema. So refactoring gets put off, bad design persists, code rots faster, and eventually the project is thrown away for a simpler one. The claimed benefit is the reverse: TypeScript stops one part of a system from handing another something it doesn't expect, and makes refactoring safer, since the compiler finds every place a change touches.

Much of the concern is widely shared. Ryan Carniato, who wrote SolidJS, put the ceremony this way in 2020: "You can spend hours perfecting an elegant solution to your types without shipping any new workable code." A data-backed retrospective on 126,000 lines of TypeScript at 15Five found that "the work it takes to add types somewhat negates the improved development speed from refactoring and intellisense." On a 2023 Hacker News thread asking whether TypeScript is worth it, one engineer wrote that they "regularly have to change good solid code to please TS." Rich Hickey named the coupling in a 2017 talk about static types in general: "Flowing type information is a major source of coupling in programs." He argued that checking "should be à la carte."

The full stack ripple is where opinion splits most cleanly, because a whole movement builds it on purpose. "End-to-end type safety," in tRPC, Prisma, Drizzle, Zod, and create-t3-app, infers one schema's types from the database through the server into each component. Its proponents present the ripple as the demonstration: delete a column, watch every affected line turn red, and in one tutorial's words, "That is the power of TypeScript and end-to-end type safety." tRPC's readme says "Move fast and break nothing." The T3 starter says that any decision that weakens its type safety "should be made in a different project." The cost appears in writing from larger systems with several teams. A long-time tRPC user found that "this tight coupling can become problematic as your application grows," with small changes to a procedure's signature forcing teams on different release schedules to coordinate. A writer on React architecture warns that passing an API's types straight into components ties the UI to the server, so a backend change means adjusting "like 90% of the frontend code." Even the tools hedge: tRPC's documentation scopes it to TypeScript monorepos.

On refactoring, the proponents' strongest claim is the opposite of the concern. The compiler hands you a list of every place a change touches, which Michael Feathers called "leaning on the compiler" in _Working Effectively with Legacy Code_. One engineer on the same thread: "I've yet to do a large refactor in a good TS codebase that caused a single regression." Matt Pocock, who teaches TypeScript, says that without types, changing a function's signature "could take hours, and with no guarantee that you'd fixed everything before you pushed to production."

Both sides are describing the same mechanism. If a page reads a column and the column goes away, the page is broken with or without types; TypeScript turns a failure at runtime into an error at compile time, and that's the safer refactoring. The brittleness comes from everything else that gets welded together when types are inferred across every layer. Changes that would break nothing at runtime become compile errors at forty places: a field becoming optional, null where undefined was, an extra property. A change one team could have made alone becomes a coordinated release, and the type layer becomes code of its own to maintain. TypeScript's own experts prescribe the cure:

- **Mirror types.** Dan Vanderkam's _Effective TypeScript_ has an item called "Mirror Types to Sever Dependencies," and advises annotating return types so that "implementation errors" don't surface "in user code."
- **Explicit exports.** Deno's JSR registry requires explicit types on everything a package exports, because inferred types aren't stable over time and slow down everyone who imports them.
- **Explicit boundaries.** typescript-eslint has a rule, explicit-module-boundary-types, to make a module's inputs and outputs explicit.
- **Translation at the edge.** The React architecture writer above maps the API's types to the app's own types where data arrives.

The practice that holds up is explicit types at the boundaries, not inference flowing through them. That's what icarus and ergo already do, by hand, with guards.

On the last part of the concern, that code rots faster and projects get thrown away, no evidence turned up either way. Both sides have only anecdotes.

## What happened, 2012 to 2026

- **2012.** Microsoft ships TypeScript, with Anders Hejlsberg, who earlier created Turbo Pascal and designed C#, as its lead architect.
- **2018.** Brendan Eich calls it "brilliant" for being a strict superset of JavaScript, and adds, "People are used to type systems that don't pay their way." Douglas Crockford: "It doesn't solve problems that I have."
- **2021.** DHH describes Rails 7's way of building web apps with no bundling or transpiling at all: "No need for any JavaScript toolchain to turn your source code into anything else."
- **2022.** Microsoft proposes type syntax for JavaScript itself, annotations that engines would ignore like comments. The proposal is still at Stage 1 in November 2026, and its readme warns that it "has not been updated regularly."
- **2023, the peak of the backlash.**
  - **May.** DHH: "TypeScript sucked out much of the joy I had writing JavaScript."
  - **Also in May.** Svelte moves its own source from .ts files to JSDoc. That wasn't a departure. Rich Harris: "If you're rabidly anti-TypeScript and think that us doing this vindicates your position, I'm about to disappoint you… Our commitment to TypeScript is stronger than ever." They kept the type checker and dropped the build step, and only for a library; an app, he said, has a build step anyway.
  - **September.** Turbo 8 drops TypeScript: it "pollutes the code with type gymnastics," and "things that should be easy become hard, and things that are hard become `any`." The pull request drew 771 thumbs down to 144 up.
- **2024 and 2025, the platform takes in TypeScript's syntax.**
  - **Node.** It strips types natively: behind a flag in 22.6 in 2024, on by default in 23.6 in January 2025 and in the 22.18 LTS that July, and since marked stable.
  - **TypeScript 5.8.** It adds `--erasableSyntaxOnly`, which forbids every TypeScript feature that does something at runtime, like enums, namespaces, and parameter properties. Code that passes it is JavaScript plus annotations that can simply be deleted, which is all Node's stripping does.
- **2025, AI tips the adoption numbers.** In August, TypeScript becomes the most used language on GitHub, measured by monthly contributors. GitHub credits AI agents, which do better with typed code, and frameworks that start new projects in TypeScript.
- **2026, the compiler is rewritten in Go.** TypeScript 6.0 in March is the last release on the old JavaScript codebase. TypeScript 7.0 ships July 8, with full builds 8 to 12 times faster and strict mode on by default. It has no stable programmatic API yet, so tools built on the compiler's API, like typescript-eslint, still need TypeScript 6. TypeScript 7.1's beta, in October, brings an API.
- **Since 2023.** No departure the size of Turbo's or Svelte's. 37signals still writes plain JavaScript.

The surveys hold a paradox. In State of JS 2025, 40% of respondents write only TypeScript, 6% write only JavaScript, and the median respondent's code is 88% TypeScript. Yet static typing is still the pain point named most often, two years running. The survey's conclusion says "TypeScript has won." Both are true at once: people use it, and it hurts. Stack Overflow's 2025 survey, which reaches past the JavaScript community, has JavaScript at 66% and TypeScript at 44%.

## The evidence

The evidence is thin, and the effects it finds are small.

- **Gao, Bird, and Barr, 2017.** They took 400 fixed public bugs from JavaScript projects on GitHub and asked whether TypeScript or Flow would have caught each one. Both would have caught 15%. Of the bugs types couldn't catch, 78% were specification errors: the code did what it said, and what it said was wrong. The study counts only public bugs, so it undercounts, and it measures nothing about what the types cost to write and keep.
- **Airbnb, 2019.** A postmortem review found 38% of the bugs in it "preventable with TypeScript." The method was never published, and postmortems cover incidents, not all bugs.
- **Bogner and Merkel, 2022.** They compared 604 GitHub applications, about half of them TypeScript. The TypeScript apps had fewer code smells and lower complexity, but "bug proneness and bug resolution time… were not significantly lower," and the bug-fix ratio was actually higher. The authors suspect TypeScript teams may simply label bugs more carefully, and warn that "TS developers should not fall into a false sense of security."
- **Tang and colleagues, 2026.** In 633 bugs from TypeScript projects, the largest category was tooling and configuration, at 27.8%, ahead of type errors at 12.4%. Static typing, they wrote, "shifted fragility toward build systems and toolchains." The authors note that comparing directly with the older JavaScript studies isn't possible.
- **The famous language study.** Ray and colleagues' 2014 study of languages and code quality didn't survive its replication. Only 4 of its 11 findings held, "and even for those the effect size is exceedingly small." Of the 41 projects labeled TypeScript, only 16 contained any TypeScript.

No peer-reviewed study measures refactoring effort, long-term maintainability, or how often projects are rewritten, in TypeScript against JavaScript. The concern above is exactly the question nobody has measured.

## The schools of restraint

Without a book, restrained TypeScript lives in schools, each defined by what it leaves out.

- **Restraint inside full TypeScript.** Vanderkam's _Effective TypeScript_, second edition, 2024, is the closest in spirit to Crockford. Its items: limit `any`; annotate signatures, not locals; "Prefer Imprecise Types to Inaccurate Types"; avoid needless generics, since "if a type parameter only appears in one location, strongly reconsider if you actually need it"; "Consider Codegen as an Alternative to Complex Types"; and "Prefer ECMAScript Features to TypeScript Features."
- **Erasable syntax only.** Leave out everything that does something at runtime. Pocock says the flag "disables a bunch of features that I don't think should ever have been part of TypeScript." Google's TypeScript style guide bans namespaces and const enums.
- **JSDoc.** Keep the checker and leave out both the syntax and the build step. The code stays plain .js, with types written in comments and checked by `// @ts-check`. webpack, Svelte, Yjs, and Preact work this way, for reasons like Yjs's maintainer's: "I don't like code transpilation." One widely discussed post is titled "JSDoc is TypeScript."
- **Types only on the public surface.** A plain `index.js` with a hand-written `index.d.ts` beside it, whose types are tested with `tsd`, as Sindre Sorhus's style guide lays out. construction.md already plans this for ergo, with an `ergo.d.ts`.
- **Brands for a few primitives.** Zod's `.brand()`, type-fest's `Tagged`, and Effect's `Brand` each make a nominal type out of a plain one. Zod's documentation calls its brand "a static-only construct," and Effect separates `Brand.nominal`, which checks nothing, from `Brand.refined`, which validates at runtime.
- **Outside TypeScript altogether.** Hickey's checking "à la carte."

The design tried below drew from three of them: erasable syntax only, a hand-written .d.ts beside plain JavaScript, and brands for a few primitives.

## Why 37signals writes plain JavaScript

37signals, which makes Basecamp and HEY, is the most prominent team that still writes plain JavaScript in 2026. Its reasons are mostly material, with taste on top of them, and taste alone only in its libraries.

Its apps barely have JavaScript to type. Rails 8's defaults, Propshaft and import maps, serve the source files straight to the browser, and browsers can't run TypeScript, so TypeScript would bring back the bundler they removed on purpose. The server renders HTML, and the JavaScript is small controllers that decorate it. Counted from the source 37signals has published:

| App       | JavaScript  | Stimulus controllers    |
| --------- | ----------- | ----------------------- |
| Fizzy     | 4,352 lines | 70, about 50 lines each |
| Campfire  | 3,855 lines | 36                      |
| Writebook | 1,069 lines | 22                      |

None of the three has a `.ts` file, a `tsconfig`, or a single JSDoc annotation. In an app shaped like that, the real contract is the HTML the server just rendered, meaning the targets and data attributes that controllers find by name, and TypeScript can't see across that boundary anyway. DHH also concedes scale: "What I object to is the idea that what's right for a code base of 10 million lines of code" should apply everywhere.

Then there's taste. "I am not ambidextrous. I do not enjoy static typing." "I don't write code with tools, I write them with text editors." And they value View Source, which is why they don't minify either.

The taste stands alone in their libraries. Turbo and Lexxy, a 16,700-line editor, already build with Rollup, so the no-build reason doesn't apply to them, and they're still plain JavaScript. Stimulus, curiously, is still TypeScript, and nobody has said why.

The critics' strongest point is about libraries versus apps. When Turbo dropped TypeScript, its users lost the types it shipped: 7.3 declared its `.d.ts` files, and 8.0 declares none. The community stepped in through DefinitelyTyped, whose types trail Turbo's releases. The most upvoted comment on the pull request said that removing types "altogether (even without JSDocs and/or .d.ts files) is a step back," and Pocock pointed out that Turbo could have shipped a hand-written `.d.ts` instead. That applies to ergo, a library with outside users. The critics' remedy, plain JavaScript with a hand-written `.d.ts` beside it, is already ergo's plan.

## Do agents do better with TypeScript

GitHub's explanation for TypeScript's rise leans on AI, so the question matters for a project built with agents. The measured answer: nobody has shown that models do better on TypeScript. The few tests that hold the task fixed lean slightly toward JavaScript, or find no difference.

- **No per-language results exist for current frontier models.** The system cards for Claude Opus 5.5 and Claude Fable 5.1 report SWE-bench Multilingual only as totals, 93.9% and 89.1%, with JavaScript and TypeScript folded into one group. No vendor publishes a JavaScript-versus-TypeScript split for any current frontier model.
- **Yusuke Endoh, March 2026.** He had Claude Code, on Opus 4.6, build a small git clone 20 times in each language. JavaScript and TypeScript both passed every test. TypeScript took 1.6 times as long, cost 1.6 times as much, and came out 25% longer. The task was prototype-sized, and he allows that static typing "may become advantageous at larger scales." He's a Ruby committer, and says so.
- **Multi-LCB, June 2026.** The same 1,055 programming-contest problems in 12 languages, on open-weight models only. Of 24 configurations, JavaScript scored higher in 12, the two tied in 7, and TypeScript scored higher in 5. These numbers came from a summarizer's reading of the paper's table, and want a spot-check before anyone leans on them.
- **Repository benchmarks.** SWE-PolyBench and Multi-SWE-bench flip between the two languages depending on the agent setup. Their JavaScript and TypeScript tasks come from different repositories, so they can't tell the language apart from the repository.
- **The "94%" figure says much less than it's cited for.** GitHub's Octoverse cites a study finding that 94% of compile errors in model-written code were type-check failures. That 2025 paper tested only TypeScript, on six small open-weight models, with no JavaScript comparison. It found that when model-written TypeScript fails to compile, the cause is usually a type error rather than a syntax error, and nothing more. A GitHub developer advocate calls the evidence "correlative."
- **MultiPL-E, 2022.** It found "no overall effect of type annotations" on how often a model wrote correct code. Running the model's TypeScript without the checker made no significant difference, so the checker caught nothing the tests didn't.
- **Agents and escape hatches.** In a 2026 study of TypeScript pull requests, agents added `any` about nine times as often as people did, 2.16 per pull request against 0.24, and used more non-null assertions and type assertions. Those are exactly the annotations that look authoritative while they skip the check.
- **Tokens.** Measurements put TypeScript at about 24% to 31% more tokens than the same JavaScript.
- **Length and attention, unmeasured.** The hypothesis that TypeScript's extra length makes logic and boundary mistakes harder to see, for a person or a model, has never been tested. The nearest research, Chroma's "context rot," shows models doing worse as their input grows, across 18 models, but it didn't test code.

Anthropic's own guidance for Claude Code is to give Claude "something it can run that comes back pass or fail." That means tests, a build's exit code, and linters, with a type check as one example. It doesn't recommend TypeScript over JavaScript. Armin Ronacher, who builds with agents daily, says an agent writing typed TypeScript "performs well. I don't know if it outperforms raw JavaScript." The measured part comes to this: what helps a model is a check it can run. ergo's shared vectors are the strongest form of that check, since they test behavior, which types can't.

## What a brand is, and what it isn't

Getting the mechanism right comes before deciding anything, and the intuitive picture is wrong in instructive ways.

TypeScript never runs. Its design goals rule out runtime type information, and the compiler writes nothing about types into the program it leaves behind: no metadata, no marker, nothing in the inspector. In a debugger, every value looks exactly as it would in plain JavaScript, because by then it is plain JavaScript. That holds for every type, whether a branded Text, an `interface User`, or a `number[]`. The one opt-in exception is `emitDecoratorMetadata`, which Angular and NestJS use to write some types into the running program, and it's exactly the kind of feature that erasable syntax forbids.

A brand is a plain type joined to a property that no real value has:

```ts
declare const textBrand: unique symbol //exists only for the checker; there's no symbol at runtime
export type Text = string & { readonly [textBrand]: true }
```

No real string has a `[textBrand]` property, so no plain string is a Text, and the only way to get one is through a function whose signature says it answers one. The symbol isn't exported, so no code elsewhere can name it to fake a brand, apart from a cast. Brands stack, too. A Min1 can go anywhere a Min0 or an Int is asked for, and not the other way:

```ts
export type Int = number & { readonly [intBrand]: true }
export type Min0 = Int & { readonly [min0Brand]: true } //a Min0 is an Int
export type Min1 = Min0 & { readonly [min1Brand]: true } //a Min1 is a Min0, and an Int
```

Four corrections follow from that:

- **Nothing is marked inside the value.** At runtime a Text is the same plain string it always was. A string is a primitive, so there's nowhere to put a flag: no property, and it can't go in a WeakSet. The approval exists only in the checker's picture of the code, before the code runs. A deeper function doesn't look at its value and see that it's approved; it contains no check at all. It's safe because the checker proved, ahead of time, that every call to it passes something that went through the check.
- **A typed parameter runs nothing.** `function save(name: Text)` causes no work at runtime, and the colon isn't shorthand for a call to checkText. The program still calls checkText, once, at the boundary. The type makes it a compile error to call save with anything that didn't come through that call, so the gain isn't that a check runs automatically. The gain is that no one can forget the one check.
- **A type can't throw.** No type can refuse a blank the moment someone tries to put one in, because types don't run. What a brand can do is make the one function that throws impossible to skip: the only door into Text is checkText, or hasText in an `if`, and that door is the one that throws. A language feature that really does check on every assignment exists in Ada, whose `Dynamic_Predicate` parse-dont-validate.md describes, and TypeScript is deliberately not that.
- **It saves lines and mistakes, not CPU.** Checking a string again was never expensive, and since a string can't change once it's checked, checking again is always correct, just repeated. The saving is that the preambles disappear and a forgotten one becomes an error.

So there are three ways to guard a value, and they differ in what runs:

1. **A preamble everywhere,** as icarus does today: the same string gets checked at each function it passes through.
2. **A brand:** checkText runs once, at the boundary, and nothing runs past it. That's less work than today, and no memory at all, since the value stays the same string, with `+`, `===`, Map keys, and JSON working as they always have.
3. **A wrapper object,** like Data: the check runs once, when the object is made, and the object carries proof at runtime. That costs memory, about 82 bytes for each one, and the value stops being a string. parse-dont-validate.md has those costs.

Erasable syntax is aligned with all this, not opposed to it. The checks that matter, like whether text is blank or a number is whole, are runtime checks, and they stay ordinary JavaScript that someone wrote. Erasable syntax only means TypeScript adds claims about them and never code of its own. The opposite, TypeScript's non-erasable features like enums, doesn't run checks either; those features just generate ordinary code. No TypeScript setting would run hasText. What TypeScript can't do is answer "is this string blank?", which is why an earlier look at TypeScript, from the angle of checking values, came up empty. What it can do is answer "has this string already been checked?", but only while the checker reads the code, never while it runs, and that difference is what the rest of these notes come down to.

## The design that was tried

The design kept the runtime checks and the claims about them in separate files, with a few rules for the app that used them.

**The runtime half was the guards, in plain JavaScript.** ergo.js would have stayed exactly as it is, with a has that asks and a check that enforces for each kind of value. Each check would have kept its own runtime guard, since a plain JavaScript caller can hand it anything. ergo's public functions are a boundary.

**The claims half was a hand-written `ergo.d.ts` beside ergo.js,** the one construction.md plans. It declared the four brands and said what each has and check promises:

```ts
export function hasText(s: unknown): s is Text //after it says true, the checker knows s is Text
export function checkText(s: unknown): Text //answers the same value with its brand, or throws
```

TypeScript has names for these. A has written this way is a _type guard_, which tutorials call `isText`. A check that returns its value with the brand is a _smart constructor_, which tutorials call `toText` or `makeText`. A third form, the _assertion function_, `assertText(s: unknown): asserts s is Text`, narrows a variable in place, but can't return a value too. So ergo's passthrough check fit the smart constructor, and the pattern turned out to have a slot for exactly the pair ergo had already invented.

**The app wrote .ts files with erasable syntax only.** Node runs them directly, erasing the types with no build step, and in the browser, the bundler a framework already has, Vite in Nuxt, strips them the same way. The app wrote types in exactly two places:

- **On a parameter that requires a brand,** like `title: Text` or `count: Min1`.
- **Where data arrives from outside,** as `unknown` or `Record<string, unknown>`. Then the checker refuses to let any of it reach a branded parameter until a check has looked at it.

Everything else stayed untyped, the way it reads in plain JavaScript. Here's the door, and a function past it:

```ts
function door(text: string) {
  let body: Record<string, unknown> = JSON.parse(text) //every field is unknown until checked
  let title = checkText(body.title) //the one check for the title
  let count = checkMin1(body.count) //and the one for the count
  saveNote(title, count)
}
function saveNote(title: Text, count: Min1) {
  store(title, `${label(title)} ${amount(count)}`) //no preamble: the types say the checks passed
}
```

And here is what Node actually runs, from its own stripper, with each annotation turned into spaces and everything else left where it was:

```js
function door(text        ) {
  let body                          = JSON.parse(text) //every field is unknown until checked
  let title = checkText(body.title) //the one check for the title
  let count = checkMin1(body.count) //and the one for the count
  saveNote(title, count)
}
function saveNote(title      , count      ) {
  store(title, `${label(title)} ${amount(count)}`) //no preamble: the types say the checks passed
}
```

That second block is the whole story in miniature. `saveNote` as it runs has no check and no type. It takes whatever it's handed.

**The configuration was strict, with one exception.** Strict mode stayed on, except `noImplicitAny`, which was turned off so a parameter left untyped is allowed and plain functions could stay plain. `strictNullChecks` had to stay on, because without it null and undefined fit every type, a Text included. `erasableSyntaxOnly` kept TypeScript to claims, and `verbatimModuleSyntax` made each type-only import say `type`, which is what lets Node's stripper know to remove it.

**The checker was a separate step, so it had to be run.** Node doesn't check types; a program with type errors still runs. So `tsc --noEmit` would have had to run in CI, before every start, and in the agent's loop after every change, or the guarantee is fiction.

**A lint rule had to close the hole that `any` leaves.** `any` is TypeScript's escape hatch, and it fits every type, a brand included. `JSON.parse` returns `any`, so do many request body readers, and so does every parameter left untyped. So `saveNote(sloppy.title, sloppy.count)` compiles with no check at all, and the hole sits exactly at the trust boundary. typescript-eslint has a rule for this, no-unsafe-argument, but in the experiment it reached much further than brands: it refused an `any` handed to `Math.abs` or to an array's `push`, so it flagged plain untyped functions that had nothing to do with brands, and it takes no options. A rule of ergo's own did the narrower job in about forty lines, refusing an `any` only where a parameter asks for a brand, and only in a call's arguments, not in assignments or returns.

**The agent would have needed the policy in writing:** types only to require a brand; `unknown` for data from outside; `as` only inside the check functions; everything else untyped; the checker and the lint after every change.

**Someone would have watched the weight.** TypeScript makes its own weight visible:

- **The checker's counters.** `tsc --extendedDiagnostics` prints the number of types the checker built and how many times it filled in a generic, plus check time and memory. A jump after a change is the signal to look closely at what the change did.
- **A trace.** `tsc --generateTrace` writes a trace to open in Perfetto, a flame chart of what the checker spent its time on.
- **A declaration snapshot.** `tsc --declaration --emitDeclarationOnly` writes out every exported type, with everything inference worked out spelled out as text, so inference leaking across a boundary shows up in a diff. Microsoft's API Extractor formalizes this as a committed report of a package's public types.
- **An `any` count.** The `type-coverage` tool reports what share of identifiers are something other than `any`.
- **The share that's types.** Strip a file with Node's `stripTypeScriptTypes` and compare the two sizes.

## The experiment

A throwaway script, built outside the repository in November 2026, sends the same values down two paths. The branded path is a door that checks a post's title and count, and five functions past it that take Text and Min1 and check nothing. The plain path is the same five functions with ergo's preamble at the top of each, the way ergo and the code built on it are written. guards.js holds the four has and check pairs in plain JavaScript, with a counter of how many checks run; guards.d.ts holds the four brands and their signatures; and brand-rule.js holds the lint rule. Node runs it, TypeScript 6 checks it, and ESLint runs the rule.

What brands got:

- **One check instead of six.** Through the five functions, the branded path ran two checks, one for the title and one for the count, both at the door. The plain path ran six.
- **The door refused bad input at runtime,** a title of spaces and a count of zero, because the door is real code, the same as a preamble.
- **The checker refused a forgotten check before anything ran,** and its messages were plain:

  ```
  Argument of type 'string' is not assignable to parameter of type 'Text'.
    Type 'string' is not assignable to type '{ readonly [textBrand]: true; }'.
  Argument of type 'number' is not assignable to parameter of type 'Min1'.
    Type 'number' is not assignable to type '{ readonly [intBrand]: true; }'.
  ```

  The first refused a plain string like "Bread", which nothing had checked. The second refused `three + 1`, since a computed number is a plain number again and needs its check written where it's made, as `checkMin1(three + 1)`. The plain path took `3 + 1` as it was, and checked it on arrival.

Where brands stopped:

- **Run anyway, the mistakes the checker would refuse went through.** Node checks nothing, so a blank title and a count of 3.5 went into the branded `saveNote` and were stored. The plain `saveNote` refused both, at runtime, from its preamble.
- **An `any` slipped past the checker.** The fields of an untyped `JSON.parse`, a title of one space and a count of -1, went through the branded path while the checker passed. Only the lint rule of ergo's own saw them. The plain path refused them.
- **A cast slipped past everything.** `saveNote('\t' as Text, 0 as Min1)` passed the checker and the lint alike, with no error and no warning, and stored a tab. The plain path refused it.

At the end, the branded shelf held "Groceries", "Bread", "Eggs", "", " ", and a tab. The plain shelf held "Groceries" and "Eggs".

The weight was mostly the platform's. The checker read 200 files: about 200 lines of the app's TypeScript and its declarations, against about 59,000 lines of the standard library's definitions and 56,000 of Node's. It checked in 0.66 seconds on TypeScript 6, and TypeScript 7 checked the same folder in 0.17 seconds. Tests wouldn't slow down at all, since Vitest, like Node, strips types without checking them, and production bundles would be the same JavaScript, since the types are gone before bundling.

## What was wanted, and what has it

Stated from its own side, rather than from what brands offer, the wish was this. Write `let a: Min5`, or a parameter `a: Min5`, where Min5 has been defined as an integer of five or more, and have that label do the work. Every value going into a Min5 gets checked once, at that moment, automatically. Anything typed Min5 is then known good, and since values don't change, it never needs checking again. A plain `let a` holds anything, as JavaScript always has.

Brands get the second half of that: once a value is a Min5, nothing checks it again. They can't get the first half, and the gap comes down to one thing, a check that runs implicitly at the moment a value converts into the type. TypeScript never runs anything because of an annotation, since its design goals rule out runtime type information, and that won't change. So where the wished-for language would quietly insert the check, TypeScript stops and asks for it to be written, and past the checker, nothing is there at all. JavaScript's own proposal for native types, the TC39 type annotations, is types that engines ignore, so the platform is heading toward the same design. The wish itself is common: asked how they'd want native types in JavaScript, State of JS 2025's respondents chose runtime types 3,524 times, beside 5,380 for annotations.

What does deliver it:

- **Ada, exactly.** `subtype Min5 is Integer range 5 .. Integer'Last;` declares the kind, and for a rule like Text's, a subtype with a `Dynamic_Predicate` does the same. Assigning to a Min5 variable or passing a value to a Min5 parameter converts it to the subtype, every such conversion runs the check, and a failed check raises an error. A compiler can leave out a check it can prove unnecessary, like handing one Min5 to another. It's the wish nearly word for word, from a language built for systems where a wrong value is a disaster.
- **Python, closely.** With beartype, `Min5 = Annotated[int, Is[lambda n: n >= 5]]` defines the kind, and one line, `beartype_this_package()`, makes every annotated function in a package check its arguments at runtime, with no decorator and no call at the call site. Pydantic's `@validate_call` does the same function by function, with `Annotated[int, Field(ge=5)]`. The label really does run the check. Where they fall short of the wish is "once": they check at every labeled function, which is the preamble, generated from the signature, and cheap. So ergo's Python edition is the one place the wish is within reach today.
- **JavaScript, not in the mainstream.** The nearest tools generate the preamble rather than remove it. Zod can wrap a function so its arguments are checked on every call, and typia turns types into checking code at build time, through a compiler plugin. A custom build step could, in principle, insert a check exactly where the checker can't prove a brand and nowhere else, which would be the wish. But it would be a tool of ergo's own, not erasable, and a new target of scrutiny, all things concept.md's standard for dependencies rules out.

## What brands gave, and what they cost

What they gave was real:

- **One check per value instead of one per function.** Each check moves to where a value is made or arrives, and the functions past it lose their preambles.
- **A forgotten check becomes a build error.** In plain JavaScript, a forgotten preamble is silently absent, and nothing notices. On forgetting, brands beat the preamble.
- **Nothing at runtime.** No memory, no wrapper, no work past the one check. The values stay strings and numbers, so `+`, `===`, Map keys, and JSON work as they always have.
- **Small and erasable.** Four brands, eight signatures, one hand-written file, nothing generated, and Node runs the code directly.
- **A check an agent can run** that comes back pass or fail, which is the one thing the research on agents supports.

What they cost was more:

- **The label never runs.** A brand's protection exists only in the checker's pass over the source. Past it, through a skipped check, a cast, an `any`, or a plain JavaScript caller, the function takes whatever it's handed, where a preamble would refuse it. The experiment's two shelves show the difference.
- **The check moves; it doesn't disappear.** Every value has to be checked where it's made, and arithmetic makes new values constantly: `count + 1` is a plain number again. The one line at the top of a function, the very thing the wish was to save, is traded for a check at every place a value is made. For values from a door, which is already checked, that's a saving. For computed values, it's about even, or worse.
- **The guarantee needs machinery to hold.** The checker has to gate CI, every start, and the agent's loop. The `any` hole needs a lint rule of ergo's own, since typescript-eslint's reaches into every plain function. Casts need a policy, since no tool in the experiment caught one. And typescript-eslint supports TypeScript only below 6.1, so for now a project runs two versions of one compiler: TypeScript 6 for the lint, and TypeScript 7 as the fast checker.
- **Everything pulls toward more.** TypeScript 7 turns strict mode on by default, and the ecosystem's documentation and examples annotate everything. So does what agents have learned from: agents add `any` nine times as often as people. The policy would have to be enforced by tools, or it drifts back toward ordinary TypeScript one helpful annotation at a time.
- **Frameworks bring their own inference.** As of this writing, Nuxt infers the return types of its server API routes into `$fetch` and `useFetch` by default, which is end-to-end inference arriving uninvited, and would have to be turned back into `unknown` at each fetch.

## The decision

ergo keeps hasInt and checkInt, hasText and checkText, and the rest of its guard pairs, in plain JavaScript. Code built on ergo keeps its preambles at the tops of the functions that need them.

The reason fits in a sentence. A preamble protects every call, at runtime, from any caller, whatever process surrounds the code, while a brand protects only the calls a checker has seen, and only if nothing runs without it. Brands would have traded the one line at the top of a function for a check at every place a value is made, a lint rule of ergo's own, a policy against casts, and a build process strict enough to hold all of it together. In exchange, they'd have given one real gain: a forgotten check caught before it runs. The research gave no reason to pay that price. Types catch a minority of bugs in every study that measured, nothing shows that agents do better with TypeScript, and ergo's vectors test behavior, which no type can.

That's the answer quad-style.md already gave, now with reasons: guards at the trust boundaries, written as plain lines, enforced at runtime.

## Left open

- **A stricter Text.** The rule these notes used, a string that trimming wouldn't change or empty, is clearer than the one ergo's hasText has today, which takes any string with something besides whitespace: one rule, no gray area. It doesn't need TypeScript; it's a guard like any other. Adopting it would mean changing hasText's meaning in all four editions, with new vectors, or adding a guard pair of its own. And a door would then trim text from people before checking it, since a form field like " Bob " would be refused.
- **JSDoc wouldn't change this.** Types written in JSDoc comments are TypeScript's types, read by the same checker and erased the same way, so they have the same limit: a comment can't run a check either.
- **ergo.d.ts is a separate question.** construction.md's plan for a hand-written ergo.d.ts, which would give editors completion and give TypeScript users ergo's signatures instead of `any`, is about the editors of the people who use ergo, not about checking values, and this study doesn't decide it.

## Sources

The history and the public voices:

- [Eich and Crockford on JavaScript's future, InfoQ, 2018](https://www.infoq.com/news/2018/07/eich-crockford-js-future/)
- [DHH, "Modern web apps without JavaScript bundling or transpiling," 2021](https://world.hey.com/dhh/modern-web-apps-without-javascript-bundling-or-transpiling-a20f2755)
- [Microsoft, "A Proposal For Type Syntax in JavaScript," 2022](https://devblogs.microsoft.com/typescript/a-proposal-for-type-syntax-in-javascript/)
- [TC39 type annotations proposal](https://github.com/tc39/proposal-type-annotations)
- [DHH, "Programming types and mindsets," 2023](https://world.hey.com/dhh/programming-types-and-mindsets-5b8490bc)
- [DHH, "Turbo 8 is dropping TypeScript," 2023](https://world.hey.com/dhh/turbo-8-is-dropping-typescript-70165c01) and [the pull request](https://github.com/hotwired/turbo/pull/971)
- [Rich Harris on Svelte's move to JSDoc, Hacker News, 2023](https://news.ycombinator.com/item?id=35892250)
- [Matt Pocock, "Are big projects ditching TypeScript?"](https://www.totaltypescript.com/are-big-projects-ditching-typescript)
- [DHH on Lex Fridman #474, transcript, 2025](https://lexfridman.com/dhh-david-heinemeier-hansson-transcript/)
- [Node.js, TypeScript type stripping](https://nodejs.org/api/typescript.html)
- [TypeScript 5.8 and `--erasableSyntaxOnly`](https://devblogs.microsoft.com/typescript/announcing-typescript-5-8/)
- [Anders Hejlsberg, "A 10x Faster TypeScript," 2025](https://devblogs.microsoft.com/typescript/typescript-native-port/)
- [TypeScript 7.0](https://devblogs.microsoft.com/typescript/announcing-typescript-7-0/)
- [GitHub Octoverse 2025](https://github.blog/news-insights/octoverse/octoverse-a-new-developer-joins-github-every-second-as-ai-leads-typescript-to-1/)
- [State of JS 2025, usage](https://2025.stateofjs.com/en-US/usage/) and [conclusion](https://2025.stateofjs.com/en-US/conclusion/)
- [Stack Overflow Developer Survey 2025](https://survey.stackoverflow.co/2025/technology)

The concern, argued both ways:

- [Ryan Carniato, "The Trouble with TypeScript," 2020](https://dev.to/ryansolid/the-trouble-with-typescript-4fpp)
- [15Five, "100,000 lines of TypeScript," 2021](https://dev.to/15five/100-000-lines-of-typescript-a-data-backed-retrospective-45dd)
- ["Ask HN: Is TypeScript worth it?", 2023](https://news.ycombinator.com/item?id=34359504)
- [Slack, "TypeScript at Slack," 2017](https://slack.engineering/typescript-at-slack/)
- [Rich Hickey, "Effective Programs," transcript, 2017](https://github.com/matthiasn/talk-transcripts/blob/master/Hickey_Rich/EffectivePrograms.md)
- [tRPC](https://github.com/trpc/trpc), [create-t3-app](https://create.t3.gg/en/introduction), and [Sabin Adams, "End-to-End Type Safety"](https://dev.to/sabinthedev/end-to-end-type-safety-what-why-and-how-179n)
- [Dennis Lin on tRPC's coupling, 2024](https://blog-dennislin.pages.dev/posts/2024-09-27-trpc-adverse-cases-alternatives)
- [Johannes Kettmann on domain entities and DTOs, 2024](https://dev.to/jkettmann/path-to-a-cleaner-react-architecture-domain-entities-dtos-3ja0)
- [Dan Vanderkam on inferable types](https://effectivetypescript.com/2020/04/28/avoid-inferable/) and [_Effective TypeScript_](https://effectivetypescript.com/)
- [JSR on slow types](https://jsr.io/docs/about-slow-types)
- [typescript-eslint, explicit-module-boundary-types](https://typescript-eslint.io/rules/explicit-module-boundary-types/)

The evidence:

- [Gao, Bird, and Barr, "To Type or Not to Type," 2017](https://janvitek.org/events/NEU/7575/pubs/no-type.pdf)
- [Airbnb's migration, InfoQ, 2020](https://www.infoq.com/news/2020/08/airbnb-typescript-migration)
- [Bogner and Merkel, 2022](https://arxiv.org/abs/2203.11115)
- [Berger and colleagues, the replication of Ray and colleagues, 2019](https://arxiv.org/abs/1901.10220)
- [Tang, Alimadadi, and Sumner, "From Logic to Toolchains," 2026](https://arxiv.org/html/2601.21186v1)
- [Dan Luu's review of the empirical literature](https://danluu.com/empirical-pl/)

The schools of restraint:

- [Matt Pocock on erasable syntax only](https://www.totaltypescript.com/erasable-syntax-only)
- [Google's TypeScript style guide](https://google.github.io/styleguide/tsguide.html)
- ["JSDoc is TypeScript," 2025](https://culi.bearblog.dev/jsdoc-is-typescript/)
- [Yjs on JSDoc](https://discuss.yjs.dev/t/why-are-yjs-types-writen-with-jsdocs-and-not-typescript/2668)
- [Sindre Sorhus, TypeScript definition style guide](https://github.com/sindresorhus/typescript-definition-style-guide)
- [Zod's branded types](https://zod.dev/api?id=branded-types), [type-fest's `Tagged`](https://github.com/sindresorhus/type-fest/blob/main/source/tagged.d.ts), and [Effect's branded types](https://effect.website/docs/code-style/branded-types/)

37signals:

- [REWORK on the TypeScript drama, 2023](https://37signals.com/podcast/typescript-drama-and-open-source-opinions/)
- [Jorge Manrubia, "A vanilla Rails stack is plenty," 2024](https://dev.37signals.com/a-vanilla-rails-stack-is-plenty/)
- [Ryan Kahn's re-migration of Turbo to TypeScript, 2025](https://dev.to/shiftyp/type-casting-doubt-how-a-typescript-re-migration-revealed-the-truth-about-turbo-2ajf)

Types that check themselves:

- [Ada 2012 Reference Manual, 3.2.4, Subtype Predicates](https://ada-rapporteur-group.github.io/ARM/Ada_2012/RM-3-2-4.html)
- [AdaCore, design by contract in Ada](https://learn.adacore.com/courses/intro-to-ada/chapters/contracts.html)
- [beartype's import hooks, `beartype_this_package()`](https://beartype.readthedocs.io/en/stable/api_claw/)
- [Pydantic's validation decorator, `@validate_call`](https://pydantic.dev/docs/validation/2.11/concepts/validation_decorator/)
- [typia](https://typia.io/)

Agents and types:

- [Yusuke Endoh, "Which Programming Language Is Best for Claude Code?", 2026](https://dev.to/mame/which-programming-language-is-best-for-claude-code-508a)
- [Multi-LCB, 2026](https://arxiv.org/html/2606.20517v1)
- [SWE-PolyBench](https://arxiv.org/html/2504.08703v1) and [Multi-SWE-bench](https://arxiv.org/html/2504.02605v1)
- [Mündler and colleagues, "Type-Constrained Code Generation with Language Models," 2025](https://arxiv.org/abs/2504.09246)
- [MultiPL-E, 2022](https://arxiv.org/pdf/2208.08227)
- [Lee, Hassan, and Hindle on type constructs in AI-generated code, 2026](https://arxiv.org/abs/2602.17955)
- [Chroma, "Context Rot," 2025](https://www.trychroma.com/research/context-rot)
- [Hejlsberg on TypeScript in the AI era, GitHub, 2025](https://github.blog/developer-skills/programming-languages-and-frameworks/typescripts-rise-in-the-ai-era-insights-from-lead-architect-anders-hejlsberg)
- [Claude Code best practices](https://code.claude.com/docs/en/best-practices)

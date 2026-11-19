# format and lint

How ergo's four editions are formatted and linted: which tools, every setting and the reason for it, how the work goes day to day, and how quad-style.md's goals land in the settings. quad-style.md is the style guide, and it governs structure, names, comments, and factoring in every edition. Format is whitespace, quotes, semicolons, and line breaks, and it belongs to a tool; concept.md has the reasoning behind that split, and construction.md has the rest of how ergo is built. This document is where the two meet. A setting here is how a rule in the style guide gets kept without anyone keeping it by hand, and a rule no tool can keep stays in the style guide alone.

## The four editions

| | Formatter | Linter | Configuration | Checked in CI by |
|---|---|---|---|---|
| JavaScript | dprint | oxlint | `dprint.json` and `.oxlintrc.json` at the root | `pnpm test`, which runs `dprint check` and `oxlint --deny-warnings` before the suite |
| Python | Ruff | Ruff | its defaults | `ruff format --check .` and `ruff check .` |
| Rust | rustfmt | Clippy | their defaults | `cargo fmt --check` and `cargo clippy --all-targets --features driver -- -D warnings` |
| Go | gofmt | `go vet` | none, since gofmt has none | `gofmt -l .` prints nothing, and `go vet ./...` |

Python, Rust, and Go each have one standard formatter, and ergo runs each as it ships. A Python engineer expects Ruff, which replaced Black, flake8, and isort as the one tool, with Black's output, four-space indentation, and 88 columns. A Rust engineer expects rustfmt and Clippy, which ship with the toolchain, with four spaces and 100 columns. A Go engineer expects gofmt, which is mandatory and unconfigurable, with tabs and no line length; `go vet` ships with the toolchain. Of the three, gofmt is the friendliest to the layout of the original, the JavaScript ergo grew from, since it never wraps a line and it aligns trailing comments on neighboring lines itself.

JavaScript has no such answer, so it took a spike to choose one.

## Why dprint and oxlint

JavaScript's formatters and linters are many, and several are good. In November 2026 a spike ran the four families a team starting today could plausibly pick, each over icarus, the upstream, on its own branch: ESLint and Prettier, the incumbent; Biome, one Rust tool that does both; oxlint and oxfmt, from the Oxc project beside Vite and Vitest; and dprint, the configurable formatter under Deno's, paired with oxlint since it only formats. Each went through the same steps: defaults, the same seven decisions about layout, the linter configured alike, its safe fixes, the dev loop, and escape hatches.

Every formatter cleared the same floor, four mechanical checks on every run: both test suites still pass, every comment's text survives, a second run changes nothing, and every file parses. So the choice was about fit and cost, not safety:

| | oxlint and oxfmt | Biome | dprint and oxlint | ESLint and Prettier |
|---|---|---|---|---|
| Look | Prettier's | Prettier's | its own, Deno's | Prettier's |
| A body left alone on the line below its `if` | 20, braced by the linter | 45, which its braces rule can't flag alone | none: the formatter braces any body that leaves its line | 45, braced by the linter |
| A comment after an opening brace | moves to the next line | moves | stays | moves |
| The space after `//`, and runs of blank lines | no | no | yes, both | no |
| Packages installed | 5 | 2 | 4, plus plugins fetched outside the lockfile | 75 |
| Before 1.0 | oxfmt | nothing | dprint and its plugins | nothing |
| The whole test run on icarus | 2.5 s | 2.44 s | 2.56 s | 3.34 s |
| Lint defaults | quiet | loud, enforcing `const` and `===` | oxlint's, quiet | errors, with `no-undef` on |

Biome and the incumbent fell behind. Biome's formatter leaves the most misleading `if`s while its braces rule has no setting to catch them alone, and its recommended lint set enforces exactly what quad-style.md reserves. The incumbent is the reference everyone knows and has the most mature linter, but it brings 75 packages from the 2010s micro-package world and the slowest loop. Between the two finalists, oxlint and oxfmt won on looking mainstream and staying simple, and dprint won on fit. dprint was chosen for the one thing no printer in Prettier's family does: a comment on the line that opens a function or block stays on that line. With it come braces added exactly when a body has to leave its line, runs of blank lines kept, and the space after `//`. Its costs were weighed and accepted: a style of its own rather than Prettier's, more settings, default import sorting to switch off, plugins fetched outside the lockfile, and versions before 1.0. oxlint came along from the Oxc column: fast, quiet by default, and useful at once, since it found a real bug in icarus on its first run, an undefined `e` in an error path.

The working rule through all of it: give up the letter of how the code looked before, freely; prefer the tool's default; and depart from a default only to defend the spirit of the style guide, with a setting an experienced JavaScript engineer would recognize from real projects. A setting justified only by "that's how it looked" didn't survive. Two ways to fail were in view the whole time. One is taking every default of the newest tool and losing what the original found on its own, which is tight, expressive, and clean. The other is stacking up settings until a formatter technically runs but the code looks unlike anything an experienced engineer has seen.

The settings are chosen for icarus as well, and kept the same in both, apart from what each project's own files need, like its paths and the files it generates. So code moves between the two projects without reformatting, and a setting stays even where ergo's own code doesn't need it today, like `no-unused-vars` left off.

## Two lenses

Every setting below is read two ways.

**Mainstream or specialized.** Does it keep the tool's default, or change it, and if it changes it, is the change one an experienced developer would recognize from real projects? Departures stay few, and each has a reason.

**Normalizes or maintains.** Does it make code come out one way however it was written, or does it keep what the author wrote? Normalizing is the default good here: a project now has two editors, the person and the model, and neither should produce a second form of the same thing. Maintaining is kept only where the author's choice carries meaning, like the order of imports or a deliberate run of blank lines.

## quad-style.md, rule by rule

How each rule of the style guide that a tool can see is kept, and which rules no tool sees.

| quad-style.md | Kept by | How |
|---|---|---|
| One form for everything, so neither editor, the person or the model, decides layout | dprint, Ruff, rustfmt, gofmt | each normalizes everything it owns, and CI fails on anything unformatted |
| Tabs | dprint `useTabs`, gofmt | JavaScript and Go indent with tabs. Python and Rust keep the spaces their formatters ship with, since PEP 8 and the Rust Style Guide both say spaces, and in Python a tab pasted into four-space code is a `TabError` |
| Width is for code, and a comment at the end of a line never breaks the code in front of it | dprint `lineWidth: 120`, rustfmt, gofmt | none of the three counts a trailing comment toward the width. Ruff does, which is why quad-style.md moves a comment that would pass 88 columns onto the line above |
| No semicolons in JavaScript | dprint `semicolons: asi` | a guarding `;` goes in front of a line that starts with `(`, `[`, or a backtick, and nowhere else |
| Single quotes, and backticks to interpolate | dprint `quoteStyle: preferSingle` | double quotes only where a string holds a single quote; backtick strings are never touched |
| A short guard on one line, `if (!row) return false` | dprint's `singleBodyPosition: maintain` and `useBraces: whenNotSingleLine`, oxlint `curly: multi-line` | a body on its `if` line stays there without braces, and a body on its own line gets braces from the formatter, so the misleading shape, a lone body on the line below, never appears; the lint rule states the same thing where readers look for rules |
| Comments at the end of the line they're about | dprint | trailing comments stay where they are, and a comment after an opening brace, `function f() { // note`, stays on that line |
| A space before the comment marker and one after | dprint `commentLine.forceSpaceAfterSlashes` in JavaScript, and Ruff in Python | `x // note` in JavaScript, exactly one space on each side, and `x  # note` in Python, with the two spaces Ruff sets before the marker; in Rust and Go, `x // note` is the convention, neither formatter changes a comment's text, and gofmt pads trailing comments on neighboring lines so they line up |
| What comments say, and their capitals | no tool | the style guide's alone |
| Tight braces for data, `{text: s}`, distinct from a block | dprint `spaceSurroundingProperties: false` and the two named-import and export settings | objects, destructuring, imports, and exports all read alike; a block is always on several lines, so spaced braces never mean data |
| A lone arrow parameter bare, `x => f(x)` | dprint `arrowFunction.useParentheses: preferNone` | |
| A trailing comma on every broken list | dprint's default `trailingCommas: onlyMultiLine` | adding an item is a one-line diff, and an edit can't leave the line above without its comma |
| `let` and `==` by default, `const` and `===` reserved as signals | oxlint, by leaving `prefer-const` and `eqeqeq` off | see Deliberately off, below |
| Ternaries flat and simple, never nested | oxlint `no-nested-ternary` | ESLint's version of the rule, which forbids any nesting |
| Runs of blank lines to set a section apart | dprint `maxBlankLines: 24` | |
| Imports in the order they're written | dprint's four sort settings, all `maintain` | sorting reorders module loading, which can change behavior |
| Alignment only where whitespace carries information | each formatter's escape for one block, or a comment, whose text no formatter touches | see Escape hatches and Recipes, below |
| Names, comment density, essays, `ttd` notes, result objects, guards, try and catch at the gates, factoring | no tool | the style guide's alone |

What the formatters give up in JavaScript: alignment padding collapses; a one-line function body, `function test(f) { _tests.push(f) }`, becomes three lines; two statements sharing a line split onto two; a hand-laid hanging layout, like arguments lined up under the first, is rejoined; and a long import goes to one name per line, losing any grouping by line.

## dprint.json

```json
{
	"lineWidth": 120,
	"useTabs": true,
	"indentWidth": 2,
	"typescript": {
		"maxBlankLines": 24,
		"operatorPosition": "sameLine",
		"semicolons": "asi",
		"quoteStyle": "preferSingle",
		"quoteProps": "asNeeded",
		"arrowFunction.useParentheses": "preferNone",
		"spaceSurroundingProperties": false,
		"importDeclaration.spaceSurroundingNamedImports": false,
		"exportDeclaration.spaceSurroundingNamedExports": false,
		"module.sortImportDeclarations": "maintain",
		"module.sortExportDeclarations": "maintain",
		"importDeclaration.sortNamedImports": "maintain",
		"exportDeclaration.sortNamedExports": "maintain"
	},
	"markup": {},
	"malva": {
		"quotes": "preferSingle"
	},
	"excludes": [
		"**/node_modules",
		"**/*-lock.json",
		"**/*.md",
		"**/*.json",
		"**/*.yaml",
		"**/*.yml",
		"**/*.svg"
	],
	"plugins": [
		"npm:@dprint/typescript@0.101.0",
		"npm:dprint-plugin-markup@0.28.0",
		"npm:dprint-plugin-malva@0.16.0"
	]
}
```

| Setting | Default | Ours | What it does | Lens |
|---|---|---|---|---|
| `lineWidth` | 120 | 120 | breaks code longer than this; comments never wrap and never count | the default, written out so a reviewer used to 80 or 100 sees the choice |
| `useTabs` | false | true | indents with tabs | specialized but respected, Biome's default and Go's standard; dprint doesn't read .editorconfig, so it's set here |
| `indentWidth` | 2 | 2 | how wide a tab counts toward the width | the default, written out because with tabs nothing else says it |
| `maxBlankLines` | 1 | 24 | keeps runs of up to 24 blank lines, trimming longer ones to 24 | specialized, and only dprint has it; maintains |
| `operatorPosition` | nextLine | sameLine | puts `&&`, `\|\|`, and `+` at the end of a broken line | Prettier's placement and the original's; normalizes |
| `semicolons` | prefer | asi | leaves semicolons off, except a guarding `;` before a line starting with `(`, `[`, or a backtick | common, the Standard style; normalizes |
| `quoteStyle` | alwaysDouble | preferSingle | single quotes, double where that avoids escaping; backtick strings untouched | common; normalizes |
| `quoteProps` | preserve | asNeeded | removes needless quotes on object keys | Prettier's default; normalizes |
| `arrowFunction.useParentheses` | maintain | preferNone | `f => f(s)` for a lone parameter | a recognized setting; normalizes |
| `spaceSurroundingProperties` | true | false | `{key: 'value'}` in objects and destructuring | specialized: tight braces read as data, distinct from the spaced or multi-line braces of a block; normalizes |
| `importDeclaration.spaceSurroundingNamedImports`, `exportDeclaration.spaceSurroundingNamedExports` | true | false | `import {x}` and `export {x}` | the same rule, so every brace holding names looks alike; normalizes |
| `module.sortImportDeclarations`, `module.sortExportDeclarations` | caseInsensitive | maintain | keeps import and export statements in written order | required: sorting reorders module loading, which can change behavior |
| `importDeclaration.sortNamedImports`, `exportDeclaration.sortNamedExports` | caseInsensitive | maintain | keeps names inside a list in written order | maintains: the order carries meaning |
| `markup` | | on defaults | formats Vue and HTML; ergo has neither, and the plugin is here so this file stays the one icarus uses, which has both | |
| `malva` | | single quotes | formats CSS, the site's one stylesheet | |
| `excludes` | node_modules, lockfiles | adds the scope | formats hand-written code only | see Scope below |
| `plugins` | | pinned | the JavaScript, markup, and CSS plugins, at exact versions | |

### Defaults that do real work

These stay at dprint's defaults, and the configuration depends on them.

| Default | What it does | Lens |
|---|---|---|
| `ifStatement.useBraces: whenNotSingleLine`, and the same for `for` and `while` loops | braces a body on its own line, even a short one, and takes the braces off a body written on its `if` line, so the misleading braceless body alone on the next line never appears | normalizes |
| `ifStatement.singleBodyPosition: maintain` | keeps `if (x) y` on one line when it was written that way and fits | maintains |
| `commentLine.forceSpaceAfterSlashes: true` | `// note`, never `//note` | normalizes |
| `trailingCommas: onlyMultiLine` | a trailing comma exactly when a list is broken across lines | normalizes |
| `preferSingleLine: false`, for arguments, arrays, objects, parameters, conditions, and chains | a list or expression written across lines stays across lines even when it would fit | maintains |
| `importDeclaration.preferSingleLine: true` | an import that fits on one line is joined onto one line | normalizes |
| `bracePosition: sameLineUnlessHanging`, `nextControlFlowPosition: sameLine` | `{` on the opening line, `} else {` together | normalizes |
| `blockPadding: remove` | no blank lines at the start or end of a block | normalizes |

Not a setting but just as important: a comment right after an opening brace, `function f() { // note`, stays on that line. It's why dprint was chosen over Prettier's family, which moves it to the first line inside.

### Normalizes and maintains, at a glance

Normalized, one output however it was written: indentation, width, semicolons, quotes, key quotes, arrow parentheses, brace spacing, operator position, trailing commas, comment spacing, braces on a body that leaves its line, brace and else placement, and block padding; and by the nature of a printer, one statement per line and expanded block bodies.

Maintained, kept as written: import and export order and the order of names inside them; runs of up to 24 blank lines; a braceless `if` on one line; a comment after an opening brace; parentheses the author wrote, even redundant ones; a braced body on its own lines, even one that would now fit beside its `if`; and a list, condition, or chain the author wrote across lines.

## Scope

The rule: format what people and agents write by hand, and leave out what machines write and what isn't code. In ergo that's the JavaScript edition in `javascript/`, `release.js` at the root, the site's `.vitepress/config.js` and its theme, and the site's stylesheet. Left out: Markdown, which is prose; JSON, where the shared vectors in `tests/` are laid out one case to a line by hand; YAML and TOML; SVG images, like the site's disc, which the markup plugin would otherwise reformat; and everything generated, like lockfiles and build output, which dprint skips by following `.gitignore`.

dprint fetches its plugins from npm itself, pinned to the exact versions in dprint.json and cached on the machine. They sit outside pnpm's lockfile, so dprint.json is where their versions live. dprint and oxlint themselves are pinned exactly in `javascript/package.json`. pnpm reports that it ignored dprint's build script; dprint runs without it.

## .oxlintrc.json

```json
{
	"env": {"browser": true, "node": true},
	"rules": {
		"curly": ["warn", "multi-line"],
		"no-undef": "warn",
		"no-var": "warn",
		"no-nested-ternary": "warn",
		"no-unused-vars": "off"
	}
}
```

| Item | Default or ours | Why |
|---|---|---|
| The correctness category's default rules, 111 of them | oxlint's default | code that's outright wrong or useless, across ESLint's core rules and the unicorn, oxc, and TypeScript plugins; 15 of them need type information and stay inert on plain JavaScript |
| `env: browser, node` | ours | ergo.js runs in both, and the driver, the suite, and release.js run in Node |
| `curly: multi-line` | ours | allows a braceless body only on its `if` line; dprint already makes this hold, and the rule makes it visible where readers look for rules |
| `no-undef` | ours; oxlint leaves it off, ESLint turns it on | flags a name used without being defined or imported; in plain JavaScript it's the nearest thing to a type checker for names |
| `no-var` | ours | keeps `var` out |
| `no-nested-ternary` | ours, ESLint's version, which forbids any nesting | the style guide says never nested |
| `no-unused-vars: off` | ours | oxlint can't treat unused imports apart from other unused variables, and icarus, which shares these settings, keeps imports ready on purpose, so a use can appear at any moment and work. Dead code is judged case by case, and tree shaking already drops unused code from bundles |

### Deliberately off

Two choices stand out to any reader trained on ESLint and Prettier, and they're deliberate. `eqeqeq` and `prefer-const` are off. The style guide reserves `===` and `const` as signals: `==` is the default because types are already certain where values meet, and `===` marks the rare place where type must be checked; `let` is the default, and `const` marks what must not change. Blanket `===` and `const` spend those signals down to nothing and give a false sense of safety, since `const` doesn't freeze an object and `===` doesn't fix confused types upstream. The cost is that the convention rests on the style guide and review rather than a tool, while agents write `const` and `===` by reflex.

Also off, as whole categories: pedantic, style, restriction, and nursery, which are menus rather than standards, and some of their rules contradict this code outright, like `no-inline-comments`, which would ban trailing comments. The four rules turned on above are taken from those categories by name, `curly` and `no-nested-ternary` from style, `no-var` from restriction, and `no-undef` from nursery, each because it says nothing on clean code and catches something real.

### A silent linter

The aim is a linter that says nothing on clean code. Standing warnings get ignored by people and agents alike, cost tokens on every run, and bury the one that matters. So every rule is either on and satisfied, or off, and `pnpm test` runs oxlint with `--deny-warnings`, so any new warning fails the run and CI. ergo is at zero.

## Escape hatches

Each formatter has a recognized escape for one block, and using one says: this whitespace is information. Use them rarely, so each reads as the signal it is.

| Edition | Formatter | Linter |
|---|---|---|
| JavaScript | `// dprint-ignore` protects the one statement directly below it; `// dprint-ignore-file`, at the top, the whole file; `excludes` in dprint.json, files by pattern | `// oxlint-disable-next-line rule-name`; `// oxlint-disable-line rule-name` at the end of a line; `/* oxlint-disable rule-name */` until `/* oxlint-enable rule-name */`; `ignorePatterns` in .oxlintrc.json |
| Python | `# fmt: off` and `# fmt: on` around a block, `# fmt: skip` on one statement | `# noqa: RULE` on the line, with the reason beside it |
| Rust | `#[rustfmt::skip]` on an item | `#[allow(clippy::lint_name)]` on an item |
| Go | none, and none needed: gofmt aligns neighboring trailing comments itself | none in `go vet` |

Name the rule in a disable comment rather than disabling everything, which keeps the exemption narrow and tells the reader why it's there. dprint's JavaScript plugin has no on and off range: a comment like `// dprint-ignore-start` simply protects the next statement. An exemption has to sit directly above the flagged line as formatted, and formatting can move that line.

ergo uses two. `textPieces` in ergo.test.js sits under `// dprint-ignore`, so its rows of code points stay in rows, each with its comment. And the Python driver's top gate, `except Exception`, carries `# noqa: BLE001`, with the reason the blind catch is right there.

## Recipes

- **Keep import groups.** No formatter keeps several names to a line grouped by meaning. Either put a comment line above each group inside the list, which dprint keeps as a heading with the names below it one per line, or write one import statement per group, each short enough to stay on one line with its own trailing comment. Importing from one module several times is ordinary JavaScript, and the module still loads once.
- **Set a section apart.** Leave blank lines, up to 24.
- **Write a short guard.** `if (!row) return false` stays on one line; if a body grows too long for the line, dprint braces it.
- **Rewrite a nested ternary** as an `if` ladder ending in the same value, which the formatter leaves alone and the linter accepts.
- **Keep a long template literal whole.** When a template literal runs past the width, dprint breaks it inside an interpolation, `${` at the end of one line and `} a second` on another, unless every interpolation past the width is a plain name or a member like `p.longer`. So compute the value first, `let rate = Math.round(cycles / fuzzSeconds)`, and write `${rate} a second`; or join two template literals with `+`, which dprint breaks at the `+`.
- **Indent a `deindent` template with tabs**, like the code around it. A formatter never touches what's inside a template literal, and deindent turns each tab at the start of a line into two spaces before it takes off the indent, so the text comes out the same. It also keeps the examples the site pulls from ergo.test.js lined up, since VitePress takes leading whitespace off a snippet a character at a time.
- **Lay out a table of values** under `// dprint-ignore`, rather than breaking the table into pieces to steer the formatter, like spreading one array into another for each row, which oxlint's `no-useless-spread` flags.
- **Point a diagram in a comment at code** by padding the comment, never the code. A formatter leaves a comment's text alone and may move the code, so the row of carets under base62's alphabet lines up in all four editions with no escape: the alphabet sits on its declaration line, and the comment below it is padded out to the alphabet's first character.
- **Bring code over from another formatter.** Prettier, at its default of 80 columns, breaks many lines that fit in 120, and dprint keeps a list, a condition, or a body broken once it's broken. So before the first real run, format once with `singleBodyPosition: sameLine`, and `preferSingleLine: true` for arguments, arrays, parameters, objects, and conditions, which rejoins whatever fits, and then run the real configuration. Stop there: preferring single lines everywhere splits declarations like `let a, b // note` across lines.
- **Put a long comment on a CSS line above it.** malva counts a trailing comment toward the width, and breaks a short declaration like `margin: 24px 0;` across two lines to make room. Ruff does the same in Python, where quad-style.md moves the comment above.

## The dev loop

The commands, in each edition's directory:

| Edition | Change files | Check only |
|---|---|---|
| JavaScript | `pnpm fix`: oxlint's safe fixes, then dprint, in that order, since a fix can leave layout for the formatter to tidy | `pnpm lint`, the full lint report; `pnpm test`, the format check, the lint check, then the suite that drives all four editions |
| Python | `uv run ruff format .` | `uv run ruff check .` |
| Rust | `cargo fmt` | `cargo clippy --all-targets --features driver -- -D warnings` |
| Go | `gofmt -w .` | `go vet ./...` |

The JavaScript scripts run their tools from the repository's root, which is how they reach release.js and the site, and because oxlint won't take a path that climbs out with `..`.

When to run them is judgment, the same as for tests. Format and lint at the end of a piece of work that changed code, in the editions it touched. Run the tests when behavior changed, and the whole suite before a commit. No hook runs any of this as code is written, and nothing needs to: `pnpm test` checks the format and the lint before the suite, CI checks everything on every push, and a file that drifts gets caught the next time the formatter runs. Editors stay plain, with no format on save. `.editorconfig` sets each language's indentation, which every editor reads.

## Blame

The commit that moved ergo from Prettier to dprint touched nearly every line of JavaScript, so `.git-blame-ignore-revs` lists it, and blame looks past it to the commit that last changed each line's meaning. GitHub reads that file on its own. Locally, `git config blame.ignoreRevsFile .git-blame-ignore-revs` tells git to. A later commit that changes only layout, like a dprint upgrade, goes in the same file.

## Caveats

- **`maxBlankLines` above about 30 hit a plugin bug** on icarus: "Formatting not stable", after which dprint left the file unformatted and reported an error. It fails loudly, never quietly, and 24 stays well clear.
- **The four sort settings must stay at `maintain`.** dprint's default sorts imports and exports, which reorders module loading.
- **dprint and its plugins are before 1.0**, so they're pinned to exact versions and upgraded deliberately: bump the version, run `pnpm fix`, check that only layout changed, commit the reformat on its own, and list it in `.git-blame-ignore-revs`. A check that only layout changed parses each file before and after, compares the trees with positions set aside, and compares every comment's text and place; letter-to-icarus2.md carries a script that does it.
- **The output is dprint's own, not Prettier's.** The difference is subtle: comments stay on brace lines, written parentheses stay, and braces get added; it's proven at scale as the engine under Deno's formatter.
- **Two comments sharing a line get split.** dprint puts a comment that follows another comment on the same line onto a line of its own, so ASCII art built from a column of `/* */` comments with words beside it comes apart, as the drawing beside hasText in ergo.js did, which is left as it came out.
- **Where a body sits decides its braces.** dprint braces a body on its own line, even one short enough to fit beside its `if`, and keeps those braces when the body later shrinks. It takes the braces off a body written on its `if` line, so `if (x) { y() }` becomes `if (x) y()`. To put a braced body back on one line, write it there without braces, and dprint keeps it.

## Beyond JavaScript

dprint is built as one formatter host for many languages, with plugins for JSON, Markdown, TOML, CSS, and more, and an `exec` plugin that runs a language's own formatter under the same `dprint fmt`. It would seem to suit a library in four languages, and ergo doesn't use it for the other three. dprint has no plugin for Rust or Go, so it could only run rustfmt and gofmt through `exec`, which wraps the real tool and changes nothing in the output. Its Ruff plugin is Ruff's own formatter built to WebAssembly, so the output would be the same, with Ruff's version pinned in dprint.json as well as uv.lock, two places to keep in step, and Ruff's linter would still run on its own. And each of those three ecosystems has one standard formatter that its engineers expect to run, which is the test this whole design answers to: an experienced engineer in each language opens their edition, runs the tools they always run, and finds nothing foreign. Linting stays per language too, since oxlint covers JavaScript only.

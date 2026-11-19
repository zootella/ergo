# letter to icarus 2

This is for icarus 2: the next home of icarus's JavaScript, the parts that will never come into ergo, in a new project of its own that imports ergo and takes over icarus by copying it in. icarus as it lives today, in the application ergo grew out of, never gets a formatter or a linter. icarus 2 does, the same ones as ergo, with the same settings.

Those settings were worked out in November 2026, in a spike run over a copy of icarus, and ergo adopted them first. ergo's own documents describe ergo, so whatever the spike learned that's about icarus or about an application, and has no place in ergo, is collected here, so that a project setting up icarus 2 needs nothing but ergo.

## What to take from ergo

- **dprint.json and .oxlintrc.json**, as they are. They're the shared copy, kept identical to what icarus 2 should run. The dprint file already carries the markup plugin for Vue and HTML and the malva plugin for CSS, though ergo has none of the first and one stylesheet of the second, so icarus 2 adds no plugins.
- **The scripts in javascript/package.json**: `fix`, which runs oxlint's safe fixes and then dprint; `lint`, the full report; and `test`, which checks the format and the lint before the suite. ergo's run from the repository's root because oxlint won't take a path that climbs out with `..`; icarus 2 points them at wherever its code lives.
- **dprint and oxlint, pinned exactly**, at the versions in ergo's package.json and dprint.json.
- **format-and-lint.md**, for what every setting does and why, the escape hatches, the recipes, and the caveats.
- **quad-style.md**, for the style, whose JavaScript rules say how the style lands under dprint.

## What icarus 2 changes

- **Generated files go out of scope**, in dprint.json's `excludes` and .oxlintrc.json's `ignorePatterns` both. In icarus that's `wrapper.js`, which a build step writes, and lockfiles, which ergo's configuration already leaves out.
- **Names a framework provides go in .oxlintrc.json's `globals`**, as `"readonly"`, so `no-undef` doesn't flag them. Where icarus touched Nuxt in the spike, those were `$fetch`, `setResponseStatus`, `useRuntimeConfig`, and `useNuxtApp`. List whatever icarus 2's framework provides, and nothing more.
- **The scope rule stays the same**: format what people and agents write by hand, and leave out what machines write and what isn't code. So JavaScript, Vue, and CSS are formatted. YAML is left out, since it's written by machines or rarely touched, and a formatter would turn its double-quoted strings to single. JSON is left out, since pnpm writes package.json. Markdown is left out, since it's notes and prose. Generated code is never formatted.

## icarus's own spots

- **The barrel.** icarus's index.js gathers the library into export lists grouped by meaning, several names to a line. dprint would put one name per line and lose the grouping, so the spike put `// dprint-ignore-file` at its top, and all 311 lines stayed exactly as written.
- **The workbench switches.** icarus has `if (false) {` blocks used as switches, five in the spike's copy, which oxlint's `no-constant-condition` flags. Each got `// oxlint-disable-next-line no-constant-condition` above it, so the rule stays on everywhere else to catch a condition that's constant by accident. One trap: an exemption has to sit directly above the flagged line as formatted, and a switch written on the same line as a test's opening ended up inside the expanded body, a line below its exemption, which then had to move inside too.
- **The import cycle.** core.js and level0.js import each other. `import/no-cycle` would flag it, so it stays off until someone looks at whether and how to unwind the cycle.
- **The drawing.** The goodwin drawing beside hasText in level0.js has words on the same lines as its comment columns, and dprint splits any two comments that share a line, so the drawing comes apart, as it did in ergo.
- **What dprint keeps.** icarus's 168 function descriptions written right after the opening brace, `function f(a) {//note`, stay on the brace line, which is the main reason dprint was chosen. Its 352 short braceless `if`s stay on one line. Runs of blank lines up to 24 stay. And dprint adds the space after `//` to every comment.
- **What dprint changes.** About 95 one-line function bodies expand to three lines, about 270 pairs of statements sharing a line split, the alignment padding collapses, and the tabs stay tabs.

## Vue and HTML

- **Formatting Vue changes a lot.** In the spike, dprint's markup plugin changed the application's 97 Vue files on about the same scale as Prettier's family, where most of the change is in templates, which get indented under `<template>`. Unlike Prettier, it closes elements cleanly by default, without Prettier's well-known `>Sign In</Button` wrap.
- **Prose in a template gets rewrapped at the width.** A page of legal paragraphs written one per line comes out hard-wrapped at 120 columns, and no setting stops it. The answers are a `<!-- markup-fmt-ignore -->` comment above the element, which is the markup plugin's own spelling, since `<!-- dprint-ignore -->` does nothing in a template; leaving content-heavy files out with `excludes`; or keeping long prose out of templates.
- **Linting Vue** finds nothing on oxlint's defaults, and oxlint's Vue plugin adds nothing. `no-undef` is the exception; see the next section.
- **CSS** formats cleanly with malva. The spike ran it over the application's stylesheet.

## Auto-imports and no-undef

A linter reads one file at a time, so a name used without an import or a declaration looks undefined to `no-undef`, and an auto-imported name looks exactly like that. A formatter never notices; this is about linting only. It matters because in plain JavaScript without TypeScript, `no-undef` is the nearest thing to a type checker for names: in the spike it found a real bug in icarus on its first run, and it would have caught the missing imports a test-moving script left behind before any test ran.

Where icarus lives today, the application auto-imports Vue's and Nuxt's names, like `ref`, `computed`, `useRoute`, and its stores; and by deliberate wiring, common icarus functions, re-exported from a composables file that Nuxt auto-imports into every component, and pinned globally for the server's files by a plugin. Compiler macros like `defineProps` and `definePageMeta` aren't imports at all. Across the application's 97 Vue files, `no-undef` flagged 377 names, every one supplied without an import, while icarus itself, which imports explicitly, had four, and one of those was the real bug.

The three ways to handle it:

1. **Apply `no-undef` only where imports are explicit**, through an override in .oxlintrc.json: on for icarus and plain JavaScript, off for components and the framework's server files. It's the smallest change, and it puts the sharpest rule where the logic lives. It gives up the check on the framework's side, where a misspelled or missing name surfaces only when the page runs.
2. **Hand the linter the list of auto-imported names**, from the declarations the framework writes into its build folder, so `no-undef` works everywhere, at the cost of a generated file kept current and a step before linting.
3. **Turn auto-imports off and import explicitly everywhere.** Every file then says where its names come from, which suits agents and search, and the code moves to another framework without leaning on one's machinery. The costs are a convenience, a framework's documentation leaning the other way, and import lines at the top of each file, which agents write for free. Compiler macros would still need declaring to the linter.

Option 1 now and option 3 later is a natural path. A project in TypeScript answers this differently: it turns `no-undef` off and lets the type checker read the framework's generated declarations.

## Moving the tests first

The spike moved icarus's unit tests out of the source files, where they sat inline under icarus's own small test runner, into Vitest files beside each source file, like `level0.test.js` beside `level0.js`, so every formatter and linter run could be checked against them. icarus 2 does the same move again, and what the spike learned:

- **A script moved them, so no test was retyped.** It parsed each file with acorn, found each top-level test by its exact character range, and moved the text unchanged, rewriting only the head, `test(() => {` to `test('', () => {`. A line in each new file, `const {ok, throws: bad} = assert`, kept the short names, so not one assertion changed.
- **It checked its own work**: every non-blank line of the originals reappears exactly once across the new files, and the counts of tests and assertions match, file by file.
- **Static checks aren't enough; run the tests.** The first real run left `export const` names out of the import headers. That passed every check, since a missing import changes no lines and no counts, and failed 14 tests the moment they ran.
- **Tests leave production bundles on purpose**, so any build that copies source files skips `*.test.js`.
- **What to tidy after:** tests that call private helpers needed those helpers exported, underscore names included, which goes against the underscore convention; library code with no caller but the tests is now exported too; and essays that pointed at examples "below" now point at another file.
- **One config at the root**, whose `include` names the folders that hold tests, since Vitest otherwise searches build output too. Tests that need a shared database build it in `beforeAll` or `setupFiles`, never `globalSetup`, which runs in a process the test workers can't reach.

## Adopting, in order

1. **Move the tests first**, as above, so every step after can be checked against them.
2. **Copy the configuration** from ergo, and make the changes above.
3. **Reformat in one commit.** icarus was never through Prettier, so format-and-lint.md's recipe for rejoining lines Prettier broke doesn't apply; its one-line guards and lines are already where dprint keeps them. Prove the commit changed only layout, with the script in the appendix below: every file still parses, every comment survives with its words and its place, and the parsed code is the same with positions set aside. Then list the commit in `.git-blame-ignore-revs` so blame looks past it.
4. **Work the lint findings to zero**, then run oxlint with `--deny-warnings` from then on. On the spike's copy of icarus, with these settings, there were 21: a nested ternary to rewrite as an `if` ladder; an undefined name in an error path; two deliberate undefined names in demonstration tests, and a typo, `tag()` for `Tag()`, in a skipped one; four `Promise.all` calls around a single promise; a few leftover test expressions and never-assigned variables; a `then` property on the test database adapter; and one `new Array`.
5. **Make the sweeps no tool makes.** Unused `catch (e)` parameters become `catch {`, about 32 of them. Comments come into quad-style.md's sentence form the way ergo's do, as code is worked on.

## Appendix: check-format.js

The spike's check that a reformat changed only layout, improved as ergo adopted dprint, and tested on ergo's reformat. It needs acorn as a development dependency and Node 22. Save it as check-format.js, format the code, and run `node check-format.js icarus` before committing the reformat, with the folder that holds the code. A dprint upgrade later is checked the same way.

```js
/*
Check that a reformat changed only layout.

Run it after formatting, before committing: node check-format.js folder [base]. It compares every JavaScript file under the folder with its version at base, a git commit, HEAD unless you name another, and proves three things. Every file still parses. Every comment survives with its words and its place: its text, with each line's surrounding whitespace set aside, so a formatter may reindent a comment but not lose, add, or reword one; and whether it ends a line of code or sits on a line of its own. And the parsed code is the same: it parses both versions with acorn, sets aside positions and raw source text, and compares the trees.

Some differences only look like code, and the tree comparison treats each as the same: a quoted object key and a bare one, 'origin': and origin:; a BigInt literal, which JSON can't hold as a number; the empty statement a formatter adds as a guarding ; when it leaves semicolons off; a body in braces and the same body bare, if (x) { y() } and if (x) y(), since dprint adds and removes braces by where a body sits; and an array literal spread into another, [...[a, b], c], which is [a, b, c]. Quote style, semicolons, parentheses, trailing commas, and whitespace never reach the tree at all. So anything a program could notice shows up as a difference, and nothing else does.

Two more checks run separately: the tests, and a second run of the formatter, which should change nothing. A lint fix that changes code on purpose, like one that adds a variable, makes this report the file that differs, rather than pass.
*/

import {execFileSync} from 'node:child_process'
import {readdirSync, readFileSync} from 'node:fs'
import {join} from 'node:path'
import * as acorn from 'acorn'

const folder = process.argv[2]
const base = process.argv[3] || 'HEAD'
const ignoredKeys = new Set(['start', 'end', 'loc', 'range', 'raw']) // Positions and raw text, which formatting is allowed to change.

function parse(source, comments) {
	return acorn.parse(source, {ecmaVersion: 'latest', sourceType: 'module', onComment: comments})
}
// Each comment as its text and its place, so a comment that moved from the end of a line onto its own line shows up.
function commentKeys(source, comments) {
	return comments.map(c => {
		let lineStart = source.lastIndexOf('\n', c.start - 1) + 1
		let trailing = source.slice(lineStart, c.start).trim().length > 0
		let text = c.value.split('\n').map(line => line.trim()).join('\n').trim()
		return (trailing ? 'ends a line: ' : 'own line: ') + text
	})
}
function bare(body) {
	return body?.type == 'BlockStatement' && body.body.length == 1 ? body.body[0] : body
}
function strip(tree) {
	return JSON.stringify(tree, (key, value) => {
		if (ignoredKeys.has(key)) return undefined
		if (typeof value == 'bigint') return value.toString() + 'n'
		if (Array.isArray(value)) return value.filter(x => x?.type != 'EmptyStatement')
		if (!value || typeof value != 'object') return value
		if (
			(value.type == 'Property' || value.type == 'MethodDefinition' || value.type == 'PropertyDefinition') &&
			!value.computed
		) {
			let k = value.key
			return {...value, key: {type: 'Key', name: k.type == 'Identifier' ? k.name : String(k.value)}}
		}
		if (value.type == 'IfStatement') {
			return {...value, consequent: bare(value.consequent), alternate: bare(value.alternate)}
		}
		if (/^(For|ForIn|ForOf|While|DoWhile)Statement$/.test(value.type)) return {...value, body: bare(value.body)}
		if (value.type == 'ArrayExpression') {
			let elements = value.elements.flatMap(e =>
				e?.type == 'SpreadElement' && e.argument.type == 'ArrayExpression' ? e.argument.elements : [e]
			)
			return {...value, elements}
		}
		return value
	})
}

let files = readdirSync(folder, {recursive: true}).filter(f =>
	f.endsWith('.js') && !f.split(/[\\/]/).includes('node_modules')
).sort()
let problems = 0
for (let file of files) {
	let before
	try {
		before = execFileSync('git', ['show', `${base}:./${file}`], {
			cwd: folder,
			encoding: 'utf8',
			stdio: ['ignore', 'pipe', 'ignore'],
			maxBuffer: 1 << 26,
		})
	} catch {
		console.log(`${file}: new since ${base}, skipped`)
		continue
	}
	let after = readFileSync(join(folder, file), 'utf8')
	if (before == after) continue

	let commentsBefore = [], commentsAfter = []
	let treeBefore = parse(before, commentsBefore)
	let treeAfter
	try {
		treeAfter = parse(after, commentsAfter)
	} catch (e) {
		console.log(`${file}: PROBLEM, no longer parses: ${e.message}`)
		problems++
		continue
	}

	let a = commentKeys(before, commentsBefore), b = commentKeys(after, commentsAfter)
	let lost = a.filter(k => {
		let i = b.indexOf(k)
		if (i == -1) return true
		b.splice(i, 1)
		return false
	})
	if (lost.length || b.length) {
		problems++
		console.log(
			`${file}: PROBLEM, comments changed: ${lost.length} lost, moved, or altered, and ${b.length} new, moved, or altered`,
		)
		for (let k of lost.slice(0, 3)) console.log(`  was ${k.slice(0, 100)}`)
		for (let k of b.slice(0, 3)) console.log(`  now ${k.slice(0, 100)}`)
	}
	if (strip(treeBefore) != strip(treeAfter)) {
		problems++
		console.log(`${file}: PROBLEM, the code differs from ${base} beyond layout`)
	}
}
console.log(
	problems ?
		`\n${problems} problems` :
		`checked ${files.length} files against ${base}: everything parses, every comment survives in its place, and the code is unchanged beyond layout`,
)
process.exit(problems ? 1 : 0)
```

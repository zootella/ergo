# ideas

Ideas beyond the core requirements of ergo, which are four editions, one test suite, and one documentation site. Each is a short description of something that would be good to have. None is committed to; concept.md and construction.md hold what is.

## The site runs the tests

The documentation site runs the JavaScript edition's test suite in the visitor's browser and shows that it passes, front and center. An easter egg for the curious and a proof of concept for everyone: this library works, right here, inside the page you're reading. The vectors are JSON and the JavaScript edition runs in a browser, on the platform and plain JavaScript, so the whole suite can run on the page with nothing installed.

## A demonstration CLI

A command-line tool that demonstrates ergo's functions and runs the test suite in all four languages from one command. Where a visitor to the site sees the JavaScript tests pass, someone who clones the repository sees all four editions pass against the same vectors, side by side.

## A playground

Maybe. A page on the site with an editor on one side and output on the other, where a visitor types and sees results, trying ergo for themselves. A guided tour above the editor walks through the functions one at a time, each step dropping an example into the editor to run and change. The JavaScript edition is the one that runs in a browser, so the playground runs JavaScript; the other three are shown beside it, not run.

## Brand

The brand color of zootella ergo is CSS Burlywood, #DEB887. The site is styled in a surrounding palette of wonderfully warm tones. Type is IBM Plex for text and IBM Plex Mono for code.

Code examples sit in fenced boxes with four tabs, one per language. A visitor clicks between them and sees syntax-highlighted JavaScript, and a click later, Rust. The highlighting needs a color scheme that is pleasing and readable, that belongs to the brand color and its warm theme, and that works well in all four languages. VitePress highlights with Shiki, which uses the same grammars as VS Code for all four languages and accepts any VS Code theme or a custom one, so the work is a theme in the palette rather than a choice of library.

## A fuzz environment

The fuzz tests in the vitest suite run when ERGO_FUZZ asks, and print how many cycles they managed. A fuller version is an interactive environment and demonstration suite: pick a function that benefits from this kind of analysis, choose the ranges and the seconds, watch the cycles climb and the editions agree, and see the distribution of inputs that were tried. On the site, it would run the JavaScript edition in the browser; in the repository, all four. It's the fuzz counterpart of the site running the tests, above.

## Demonstrators

Generators and visualizers that show why a design choice was made, beside the tests that prove it works. The first: render base62 output in a common font like small Inter, and show how much narrower the text gets when i and j sit in the alphabet's most common positions.

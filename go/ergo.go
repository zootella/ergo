// Package ergo is short for ergonomic: simple helper functions, the same in JavaScript, Python, Rust, and Go.
package ergo

import (
	"bytes"
	"cmp"
	"crypto/rand"
	"crypto/sha1"
	"crypto/sha256"
	"encoding/base32"
	"encoding/base64"
	"encoding/hex"
	"fmt"
	"hash"
	"io"
	"math"
	"regexp"
	"slices"
	"strconv"
	"strings"
	"time"
	"unicode"
	"unicode/utf8"

	"golang.org/x/text/unicode/norm"
	"lukechampine.com/blake3"
)

// An integer in ergo is one every edition holds exactly: a whole number from -(2^53 - 1) through 2^53 - 1, JavaScript's safe range. Past it, a double can't tell one integer from the next, so a number that crossed from Rust or Go into a page could quietly change. HasInt asks whether a value is one, and asking never panics, so a number from outside can be asked about first; CheckInt is the guard that enforces the same answer.

const intCeiling = 1<<53 - 1 // the largest integer a double holds along with every integer below it

// HasInt is true if i is an integer every edition holds exactly, a whole number from -(2^53 - 1) through 2^53 - 1. It never panics, so a number from outside can be asked about first.
func HasInt(i int64) bool {
	return -intCeiling <= i && i <= intCeiling
}

// CheckInt is a guard: it passes an integer through, or panics because the caller handed it something that isn't one, which is a bug in the caller, not an ordinary answer to handle. Ask HasInt first about a number from outside.
func CheckInt(i int64) int64 {
	if !HasInt(i) {
		panic(fmt.Sprintf("not an integer: %d", i))
	}
	return i
}

// A string is a type, and text is a string that holds something. HasString asks only about the type: is this a string at all, blank and whitespace included. In Rust and Go every string parameter already is one, so there it always answers true, and it's in all four editions to keep them alike. HasText asks about what's inside: a string with at least one character that isn't whitespace, so "a" and " a " are text, while "", " ", and a tab and a newline aren't.
//
// Whitespace means one thing everywhere in ergo: the 25 characters on Unicode's White_Space list, and Trim takes them off both ends of a string. Each platform's own trim keeps a list of its own. JavaScript's differs from Unicode's at exactly two characters of the 1,112,064: it trims the byte order mark, U+FEFF, which Unicode doesn't call whitespace, and keeps the next line control, U+0085, which Unicode does. Python's strip also trims the four separator controls, U+001C through U+001F, and Rust's trim and Go's TrimSpace follow Unicode's list already, so here Trim is Go's own. So ergo uses Unicode's list in every edition, and Trim, HasText, and Deindent agree everywhere about what whitespace is.
//
// The two characters are worth knowing by name. Next line is a line break from IBM's mainframes, whose EBCDIC text ends each line with it, and Unicode gave it a place so that text could come over and go back. Nobody types it, and a browser never makes it. Its usual accident is the byte 0x85, the ellipsis in Windows-1252, read as Latin-1 instead, and browsers read anything labeled Latin-1 as Windows-1252. So it turns up only in old data, from a mainframe or from server code that decodes with a true Latin-1. The byte order mark is common. It sits invisibly at the very start of a file to say how the file is encoded, and Excel's CSV export and older Windows tools write one. The web platform drops it whenever it decodes text, in fetch, TextDecoder, and a File's text, but Go's os.ReadFile and a string made from those bytes keep it, and so does a Data, so that bytes and text round-trip exactly. A server that reads an uploaded spreadsheet's bytes and splits its first line finds the mark in front of the first column's name, and Trim leaves it there, as Go's own TrimSpace does. So code that reads files from people should take off a leading byte order mark on purpose.

// HasString reports whether s is a string, which in Go it always is, since the signature already says so. It's here so the four editions match: in JavaScript and Python, where a value can be anything, hasString checks the type at runtime.
func HasString(s string) bool {
	return true
}

// CheckString passes a string through. It never panics, since the signature already guarantees a string; it's here so the four editions match, where in JavaScript and Python checkString is a guard.
func CheckString(s string) string {
	return s
}

// /*************************************************************************/
// /*                                                                       */
// /*                                  (`-.                                 */
// /*                                   \  `                                */
// /*      /)         ,   '--.           \    `                             */
// /*     //     , '          \/          \   `   `                         */
// /*    //    ,'              ./         /\    \>- `   ,----------.        */
// /*   ( \  ,'    .-.-._        /      ,' /\    \   . `            `.      */
// /*    \ \'     /.--. .)       ./   ,'  /  \     .      `           `.    */
// /*     \     -{/    \ .)        / /   / ,' \       `     `-----.     \   */
// /*     <\      )     ).:)       ./   /,' ,' \        `.  /\)    `.    \  */
// /*      >^,  //     /..:)       /   //--'    \         `(         )    ) */
// /*       | ,'/     /. .:)      /   (/         \          \       /    /  */
// /*       ( |(_    (...::)     (                \       .-.\     /   ,'   */
// /*       (O| /     \:.::)                      /\    ,'   \)   /  ,'     */
// /*        \|/      /`.:::)                   ,/  \  /         (  /       */
// /*                /  /`,.:)                ,'/    )/           \ \       */
// /*              ,' ,'.'  `:>-._._________,<;'    (/            (,'       */
// /*            ,'  /  |     `^-^--^--^-^-^-'                              */
// /*  .--------'   /   |                                                   */
// /* (       .----'    |   *************************************************/
// /*  \ <`.  \         |   */
// /*   \ \ `. \        |   */  Make sure s is a string that has some text,
// /*    \ \  `.`.      |   */  meaning it's not blank, and not just space
// /*     \ \   `.`.    |   */
// /*      \ \    `.`.  |   */
// /*       \ \     `.`.|   */
// /*        \ \      `.`.  */
// /*         \ \     ,^-'  */
// /*          \ \    |     */
// /*           `.`.  |     */
// /*              .`.|     */
// /*               `._>    */
// /*                       */
// /*       g o o d w i n   */
// /*                       */
// /*************************/

// HasText reports whether s is text: a string with at least one character that isn't whitespace, so " a " is text and " " isn't. It never panics.
func HasText(s string) bool {
	return Trim(s) != ""
}

// CheckText is a guard: it passes the text through, or panics because the caller handed it a string that's blank or just whitespace, which is a bug in the caller. Ask HasText first about a string from outside.
func CheckText(s string) string {
	if !HasText(s) {
		panic(fmt.Sprintf("not text: %q", s))
	}
	return s // pass the text through
}

// Trim takes whitespace off both ends of s, by Unicode's list. Go's own strings.TrimSpace takes off exactly that list, so this is Go's own, under the name the four editions share.
func Trim(s string) string {
	return strings.TrimSpace(s)
}

// Cut finds a tag in text and cuts the text around it: what's before, the tag itself, and what's after, along with whether it found the tag at all. CutLast does the same from the end, Cut2 finds two tags in order and the middle between them, and CutAfterLast is for text whose shape you already know, answering what comes after the last tag, and panicking when the tag isn't there.
//
// A tag is any string of at least one character, whitespace included, so a space, a tab, or a newline can be the tag to cut at. The blank string can't: it would be found at the start of everything, so a loop that cuts until nothing is left would never end.

// CutResult is what Cut and CutLast answer: whether they found the tag, and the text before it, the tag itself, and the text after it, which assemble back into the text.
type CutResult struct {
	Found  bool   // whether the tag was there
	Before string // the text before the tag, or all of it when the tag wasn't found
	Tag    string // the tag, or blank
	After  string // the text after the tag, or blank
}

// Cut2Result is what Cut2 answers: whether it found both tags, and the text before the first, the first tag, the middle between them, the second tag, and the text after it.
type Cut2Result struct {
	Found  bool   // whether both tags were there, in order
	Before string // the text before the first tag, or all of it when they weren't both found
	Tag1   string // the first tag, or blank
	Middle string // the text between the two tags, or blank
	Tag2   string // the second tag, or blank
	After  string // the text after the second tag, or blank
}

// Cut finds the first tag in s, and cuts s around it. It panics if tag is blank, which would be found at the start of everything; any tag of at least one character is fine, whitespace included.
func Cut(s, tag string) CutResult {
	checkCutTag(tag)
	before, after, found := strings.Cut(s, tag) // go's own cut, which answers the text on either side
	if !found {
		return CutResult{Before: s} // not found, so all of it is before
	}
	return CutResult{Found: true, Before: before, Tag: tag, After: after}
}

// CutLast finds the last tag in s, and cuts s around it. It panics if tag is blank.
func CutLast(s, tag string) CutResult {
	checkCutTag(tag)
	i := strings.LastIndex(s, tag)
	if i == -1 {
		return CutResult{Before: s}
	}
	return CutResult{Found: true, Before: s[:i], Tag: tag, After: s[i+len(tag):]}
}

// Cut2 finds tag1 in s, and then tag2 after it, and cuts s around both. It panics if either tag is blank.
func Cut2(s, tag1, tag2 string) Cut2Result {
	c1 := Cut(s, tag1)
	c2 := Cut(c1.After, tag2)
	if c1.Found && c2.Found {
		return Cut2Result{Found: true, Before: c1.Before, Tag1: c1.Tag, Middle: c2.Before, Tag2: c2.Tag, After: c2.After}
	}
	return Cut2Result{Before: s} // both must be found
}

// CutAfterLast is what comes after the last tag in s. It panics if s doesn't have the tag at all, since it's for text whose shape the caller already knows, or if tag is blank; ask CutLast(s, tag).Found first about text from outside.
func CutAfterLast(s, tag string) string {
	c := CutLast(s, tag)
	if !c.Found {
		panic(fmt.Sprintf("no %q in %q", tag, s)) // make sure s is what we expect
	}
	return c.After
}

// checkCutTag is a guard: it panics because a tag to cut at is blank; whitespace is fine
func checkCutTag(tag string) {
	if tag == "" {
		panic(fmt.Sprintf("not a tag to cut at: %q", tag))
	}
}

// Deindent lets a multiline string literal be indented with the code around it, so it doesn't mess up overall readability! The indent of the first line comes off every line, deeper lines keep the rest of theirs, blank lines at the start and end go, and the text ends with one newline. In JavaScript it's a tag for a template literal; here it takes the text, like a raw string in backticks.
func Deindent(s string) string {
	normal := strings.ReplaceAll(strings.ReplaceAll(s, "\r\n", "\n"), "\r", "\n") // temporarily normalize newlines to just \n
	lines := strings.Split(normal, "\n")                                          // split into lines
	for i, line := range lines {
		lines[i] = strings.TrimRightFunc(tabsToSpaces(line), unicode.IsSpace) // replace tabs at the start with two spaces, and take whitespace off the end
	}
	lines = strings.Split(strings.Trim(strings.Join(lines, "\n"), "\n"), "\n") // remove blank lines from the start and end; blank lines in the middle stay

	indent := leadingSpaces(lines[0]) // how many spaces indent the first line
	var b strings.Builder
	for _, line := range lines {
		b.WriteString(line[min(leadingSpaces(line), indent):]) // remove up to that many spaces from each line
		b.WriteString("\n")                                    // reassemble lines with trailing newlines
	}
	return b.String()
}

// tabsToSpaces replaces each tab among a line's leading spaces and tabs with two spaces
func tabsToSpaces(line string) string {
	rest := strings.TrimLeft(line, " \t")
	return strings.ReplaceAll(line[:len(line)-len(rest)], "\t", "  ") + rest
}

// leadingSpaces is how many spaces a line starts with
func leadingSpaces(line string) int {
	return len(line) - len(strings.TrimLeft(line, " "))
}

// Numbers of bytes in the binary units that memory and storage are measured in, 1024 to each rather than 1000, so 64*ergo.KB names what a bare 65536 would hide. Untyped, so they multiply an int, an int64, or a uint64 alike.
const (
	B  = 1         // one byte, the unit
	KB = 1024 * B  // kibibyte
	MB = 1024 * KB // mebibyte
	GB = 1024 * MB // gibibyte
	TB = 1024 * GB // tebibyte
	PB = 1024 * TB // pebibyte, and the last: 2^50, still a safe integer in every edition
)

// each 1024 of the one before, labeled the everyday way; bytes come before these, and are the one unit that pluralizes
var sizeUnits = []string{" KB", " MB", " GB", " TB", " PB", " EB", " ZB", " YB"}

// SaySize4 says a count of bytes in four digits or less with a unit, like "1 byte", "2 bytes", or "117 MB". The count rounds down.
// A uint64 is never negative or a fraction, the counts the JavaScript and Python editions refuse, and past the integer ceiling SaySize4 still says it right, so nothing here panics.
func SaySize4(n uint64) string {
	if n <= 9999 { // at most four digits of bytes, so say bytes
		s := "s"
		if n == 1 { // singular only for exactly one
			s = ""
		}
		return strconv.FormatUint(n, 10) + " byte" + s
	}
	w := n // whole units of the unit we're counting in
	for _, unit := range sizeUnits {
		w /= KB // count in the next unit up
		if w <= 9999 {
			return strconv.FormatUint(w, 10) + unit // fits in four digits, so this is the unit to say it in
		}
	}
	return strconv.FormatUint(n, 10) // past yottabytes, which a uint64 can't reach; the compiler still wants an answer here
}

// Spans of time in milliseconds, the unit of a tick, so now+2*ergo.Hour is a deadline two hours out. Integers of milliseconds rather than time.Duration, so they add to a tick directly and mean the same number in every edition.
const (
	Millisecond = 1 // the unit; a tick counts milliseconds since 1970
	Second      = 1000 * Millisecond
	Minute      = 60 * Second
	Hour        = 60 * Minute
	Day         = 24 * Hour
	Week        = 7 * Day
	Year        = 365*Day + Day/4 // 365 and a quarter days, the average over the leap cycle, which is exact in milliseconds
	Month       = Year / 12       // a twelfth of that year, also exact
)

// SayDuration says a span of time for a person to read: how long something took, or how long ago it happened, like "2 hours 5 minutes" or "about 3 days ago". The template is the sentence it writes, with a # wherever a number goes. A tag is that #, any whitespace, and the unit after it: years, months, weeks, days, hours, minutes, or seconds, or one of the short forms yrs, yr, y, mos, mo, wks, w, d, hrs, hr, h, mins, min, m, secs, sec, and s, each written in lowercase, title case, or all capitals, like days, Days, and DAYS. Seconds can also take a fraction, #.#, #.##, or #.###, for tenths, hundredths, or thousandths, and no other unit can. Each # after the point is a digit always written, so 1.5 seconds says "1.500" under #.###, the way Excel's 0.000 writes it rather than its #.###. The whitespace can be any character Unicode calls white space: a space, a tab, or the no-break space a typographer puts there to keep "5 days" on one line.
//
// The units a template names split the span between them, and a template lists them from largest to smallest. The largest takes everything above it, so a year said in days is 365, and each smaller unit takes what's left. Everything rounds down, so whatever is smaller than the smallest unit goes unsaid. A year and a month are ergo.Year and ergo.Month, the averages over the leap cycle, so SayDuration agrees with the rest of ergo, and a span of 365 days is a quarter of a day short of a year.
//
// A bare m is the one word that could be two units. On its own it's minutes, the way "5m" is in a feed, but because a template runs from largest to smallest, an m right after years, or right before weeks, days, or hours, can only be months, so "#y #m #d" says "1y 2m 3d" and "#d #h #m" says "1d 2h 3m".
//
// A plural, whole or short, comes out singular for exactly 1, in the case it was written in, so "# days" says "1 day" and "2 days", "# hrs" says "1 hr", and "# DAYS" says "1 DAY"; a single letter, like d, never changes.
//
// Units that come out zero at the start drop away, along with the text after them, so a template ready to say days can count up a task half an hour old: "#d #h #m #.##s" says "32m 55.45s", never "0d 0h 32m 55.45s". From the first unit that isn't zero, everything stays. A zero between two others stays, so a day and a second is "1 day 0 hours 0 minutes 1 second" rather than "1 day 1 second". And the smallest unit a template names is the precision it asks for, so it always shows, even on an exact amount: "#m #s" says "1m 0s" for exactly a minute. If every unit is zero, the last and smallest one says so, like "0 minutes".
//
// A unit only counts right after a #, so words and numbers around the tags pass through as they are: "about # days ago" says "about 3 days ago", and the d in "and" is only a letter. Only those exact spellings count, so a mix like hOURS is the tag h and then text, the way #hiring is. Whatever isn't a tag is written as it is, with no escaping and nothing refused, so a broken template writes text that's only wrong. The span is what must be right: a number of milliseconds within the integer ceiling, and anything else panics. It's an int64, so it has no fraction of a millisecond to drop, the way the editions that take a double do. A negative span says its size with a minus sign in front of everything, like "-2 hours 5 minutes", unless what it says is zero.

// durationWords is each unit's words, whole and short, with the size of the unit in milliseconds, which is also how the code tells units apart; each longer word comes before its prefixes, since the first that matches wins, and a template writes each in lowercase, title case, or all capitals
var durationWords = [...]struct {
	word string
	unit int64
}{
	{"years", Year}, {"yrs", Year}, {"yr", Year}, {"y", Year},
	{"months", Month}, {"mos", Month}, {"mo", Month},
	{"weeks", Week}, {"wks", Week}, {"w", Week},
	{"days", Day}, {"d", Day},
	{"hours", Hour}, {"hrs", Hour}, {"hr", Hour}, {"h", Hour},
	{"minutes", Minute}, {"mins", Minute}, {"min", Minute}, {"m", Minute}, // or months, by where it sits; SayDuration below decides
	{"seconds", Second}, {"secs", Second}, {"sec", Second}, {"s", Second},
}

// durationTag is a tag in a duration template: its unit's size, the whitespace and word it was written with, and how many digits of a second it wants
type durationTag struct {
	unit       int64
	whitespace string
	word       string
	digits     int
}

// durationTags finds a #, a fraction for seconds, any whitespace, and the run of english letters after them, where SayDuration looks for a unit's word;
// the whitespace is unicode's white space written out, since \s in Go's regexp is only five ascii characters
var durationTags = regexp.MustCompile(`#(\.#{1,3})?([\t\n\v\f\r \x{85}\x{a0}\x{1680}\x{2000}-\x{200a}\x{2028}\x{2029}\x{202f}\x{205f}\x{3000}]*)([A-Za-z]*)`)

// durationWord is the word as written and its unit, for the unit a run of letters starts with, spelled exactly in lowercase, title case, or all capitals, like days, Days, or DAYS, trying each longer word before its prefixes, or false
func durationWord(letters string) (string, int64, bool) {
	for _, w := range durationWords {
		title := strings.ToUpper(w.word[:1]) + w.word[1:]
		for _, spelling := range []string{w.word, title, strings.ToUpper(w.word)} {
			if strings.HasPrefix(letters, spelling) {
				return spelling, w.unit, true
			}
		}
	}
	return "", 0, false
}

// SayDuration says a span of milliseconds in the units a template names, like "# hours # minutes" for "2 hours 5 minutes"; one way, with no reader. It panics if span is past the integer ceiling, 2^53 - 1 either way, which is a bug in the caller; ask HasInt first about a number from outside.
func SayDuration(span int64, template string) string {
	CheckInt(span)

	// cut the template once, into its tags and the text around them
	var texts []string // the text before each tag, and then the text after the last one
	var tags []durationTag
	at := 0 // where the text not yet cut begins
	for _, m := range durationTags.FindAllStringSubmatchIndex(template, -1) {
		// m holds the start and end of the whole match, and then of each group: the fraction, the whitespace, and the letters
		word, unit, ok := durationWord(template[m[6]:m[7]])
		if !ok {
			continue // no unit's spelling starts the letters, so the # and any fraction it read are only text
		}
		digits := 0
		if m[2] >= 0 {
			digits = m[3] - m[2] - 1 // the # after the point
		}
		if digits > 0 && unit != Second {
			continue // a fraction only says seconds; before any other unit, the whole match stays text, so its last # can't make a tag of its own
		}
		texts = append(texts, template[at:m[0]])
		tags = append(tags, durationTag{unit, template[m[4]:m[5]], word, digits})
		at = m[6] + len(word) // just past the unit's word, so any letters after it stay text, like the "iring" of "#hiring"
	}
	texts = append(texts, template[at:])
	if len(tags) == 0 {
		return template // no tags, so nothing to fill in
	}

	// a bare m is minutes, unless it sits where only months fit in a template that runs from largest to smallest: right after years, or right before weeks, days, or hours
	for i := range tags {
		if tags[i].word != "m" && tags[i].word != "M" {
			continue
		}
		var before, after int64 // the units of the tags on either side, if there are any
		if i > 0 {
			before = tags[i-1].unit
		}
		if i+1 < len(tags) {
			after = tags[i+1].unit
		}
		if before == Year || after == Week || after == Day || after == Hour {
			tags[i].unit = Month
		}
	}

	// split the span among the units named, largest first, each taking all of itself that fits in what's left
	var units []int64
	for _, tag := range tags {
		units = append(units, tag.unit)
	}
	slices.Sort(units)
	slices.Reverse(units)
	units = slices.Compact(units)
	counts := map[int64]int64{}
	left := span // milliseconds not yet said; a negative span says its size, and gets its minus sign at the end
	if left < 0 {
		left = -left
	}
	for _, unit := range units {
		counts[unit] = left / unit
		left %= unit
	} // left now holds the remainder that rounds away, or the fraction of a second

	// write each tag, then keep everything from the first tag that isn't zero through the end
	said := make([]string, len(tags))
	first := -1 // zeros at the start drop away, along with the text after them
	for i, tag := range tags {
		count := counts[tag.unit]
		number := strconv.FormatInt(count, 10)
		zero := count == 0
		if tag.digits > 0 {
			fraction := left / int64(math.Pow10(3-tag.digits)) // tenths, hundredths, or thousandths of the second, rounded down
			number += fmt.Sprintf(".%0*d", tag.digits, fraction)
			zero = zero && fraction == 0
		}
		word := tag.word
		if number == "1" && len(word) > 1 && (strings.HasSuffix(word, "s") || strings.HasSuffix(word, "S")) {
			word = word[:len(word)-1] // a plural, whole or short, is singular for exactly 1, like day, hr, and DAY; the single letter s is seconds, not a plural, and never changes
		}
		said[i] = number + tag.whitespace + word
		if !zero && first == -1 {
			first = i
		}
	}
	sign := ""
	if first == -1 {
		first = len(said) - 1 // every unit is zero, so say the last and smallest, like "0 minutes"
	} else if span < 0 {
		sign = "-" // a minus sign only in front of something that isn't zero
	}
	text := sign + texts[0] // text before the first tag always stays, like "about"
	for i := first; i < len(said); i++ {
		text += said[i] + texts[i+1] // from there to the end, every tag and the text after it, so the smallest unit always shows, and so does text after the last tag, like "ago"
	}
	return text
}

// Three shortcuts sit on top of SayDuration, each with its template chosen for one use, in place of a template from the caller. They come from ergo's ancestor from 2013, which said time these three ways, and here each is a few lines of ergo.
//
// SayTimeRemaining says how long until something finishes, or how long it has run, in the units that matter at its size: two units while the larger is a single digit, like "9m 59s" and "1h 0m", and one unit after, like "10m" and "10h". It stays in hours up to 71 before it turns to days, since "71h" is plainer than "2d 23h" for something that finishes this week. SayTimeRemainingCoarse says the same, with the seconds rounded down to a multiple of 5, so a countdown on a page steps rather than flickers; the last five seconds still count down one at a time.
//
// SayTimeRace says how long something took to the millisecond, the way a race is timed: 5'15"223 is five minutes, fifteen seconds, and 223 milliseconds, with the minutes taking everything above them, so an hour and a quarter is 75'00"000. Its seconds and milliseconds are padded, which SayDuration never does, so it writes the three numbers itself.

// SayTimeRemaining says how long remains in one or two units, like "45s", "9m 59s", "10m", "1h 0m", "10h", or "3d", for a progress bar or an estimate. It panics if span is past the integer ceiling, 2^53 - 1 either way; ask HasInt first about a number from outside.
func SayTimeRemaining(span int64) string {
	return sayTimeRemaining(span, false)
}

// SayTimeRemainingCoarse says how long remains like SayTimeRemaining, with the seconds rounded down to a multiple of 5, so a countdown isn't distracting. It panics if span is past the integer ceiling, 2^53 - 1 either way; ask HasInt first about a number from outside.
func SayTimeRemainingCoarse(span int64) string {
	return sayTimeRemaining(span, true)
}

// sayTimeRemaining says a span with the template for the units that matter at its size
func sayTimeRemaining(span int64, coarse bool) string {
	CheckInt(span)
	size := span // the size picks the template; a negative span gets its minus sign from SayDuration
	if size < 0 {
		size = -size
	}
	if coarse && size > 5*Second {
		size -= size % (5 * Second) // down to a multiple of 5 seconds, while the last five still count one at a time
	}
	signed := size // the size, with the sign of the span
	if span < 0 {
		signed = -size
	}

	switch {
	case size < Minute:
		return SayDuration(signed, "#s") // "0s" to "59s"
	case size < 10*Minute:
		return SayDuration(signed, "#m #s") // "1m 0s" to "9m 59s"
	case size < Hour:
		return SayDuration(signed, "#m") // "10m" to "59m"
	case size < 10*Hour:
		return SayDuration(signed, "#h #m") // "1h 0m" to "9h 59m"
	case size < 3*Day:
		return SayDuration(signed, "#h") // "10h" to "71h"
	}
	return SayDuration(signed, "#d") // "3d" and up
}

// SayTimeRace says how long something took to the millisecond, the way a race is timed, like 5'15"223 for five minutes, fifteen seconds, and 223 milliseconds. It panics if span is past the integer ceiling, 2^53 - 1 either way; ask HasInt first about a number from outside.
func SayTimeRace(span int64) string {
	CheckInt(span)
	if span < 0 {
		return "-" + SayTimeRace(-span) // a negative span says its size, with a minus sign in front
	}
	minutes := span / Minute // the minutes take everything above them
	seconds := span % Minute / Second
	ms := span % Second
	// seconds and milliseconds padded, so every time reads the same way
	return fmt.Sprintf(`%d'%02d"%03d`, minutes, seconds, ms)
}

// A tick is an integer count of milliseconds since the start of 1970, the way time.Now().UnixMilli() counts, and ergo keeps every tick between zero and the last millisecond of 9999. Two text forms carry a tick exactly. The iso form is like "2012-08-06T05:17:57.300Z": always 24 characters, always milliseconds, always Z. The short form is like "2012aug6.0517.57300": the year, month, and day, then a dot and the clock, then a dot and the milliseconds into the minute, written only as deep as the moment needs, so the start of 1990 is just "1990" and the longest text is 20 characters.
//
// Each form reads back to exactly the tick that wrote it, and reads nothing else. A reader accepts the text its writer produces and refuses everything else, from a missing millisecond digit to "1990feb1" where the writer would say "1990feb". The last check in each reader is the round trip itself: parse the text, write the tick, and require the same text. That's what keeps every text one to one with its tick.
//
// Every function that takes a tick panics on a number that isn't one, because a caller handing it one has a bug. A number from outside, like a system clock or a file's modified time, isn't the caller's to vouch for, so HasTick asks about it first, and asking never panics; once HasTick says yes, nothing here panics on that tick. CheckTick is the guard that enforces the same answer, and it's the gate every function here goes through.

const lastTick = 253402300799999 // the last millisecond of 9999, which keeps a year at four digits in every edition

// monthNames and dayNames are the months of the year and the days of the week, whole and in title case the way a sentence writes them, so a caller clips and then needs only strings.ToUpper or strings.ToLower for the other two cases; in english, each short name is the first three letters, like "Aug" and "Thu", which the short form and SayTick both clip out
var monthNames = [12]string{"January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"}
var dayNames = [7]string{"Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"} // sunday first in every edition, the way javascript's getUTCDay and go's Weekday count from 0; python's own weekday() starts with monday, so a python caller who wants the week the other three keep has it in ergo

// shortMonths is the short form's month names, clipped and lowercased once from monthNames, like "aug"
var shortMonths = func() (short [12]string) {
	for i, name := range monthNames {
		short[i] = strings.ToLower(name[:3])
	}
	return short
}()

// MonthNames is the months of the year, whole and in title case, like "August", for a caller to clip and case. Each short name is the first three letters, like "Aug". It's a copy, so no caller can change the names under SayTick and the short form.
func MonthNames() [12]string { return monthNames }

// DayNames is the days of the week, whole and in title case, Sunday first, like time.Weekday counts. Each short name is the first three letters, like "Thu". It's a copy, so no caller can change the names under SayTick.
func DayNames() [7]string { return dayNames }

// parts of a tick in utc: the year, month 1 through 12, day 1 through 31, hour, minute, and milliseconds into the minute
type parts struct{ y, m, d, h, i, ms int }

// HasTick is true if t is a tick, zero through the last millisecond of 9999. It never panics, so a number from outside, like a clock or a file's modified time, can be asked about before it's handed to a function that takes a tick.
func HasTick(t int64) bool {
	return 0 <= t && t <= lastTick
}

// CheckTick is a guard: it passes a tick through, or panics because the caller handed it something that isn't one, which is a bug in the caller, not an ordinary answer to handle. Ask HasTick first about a number from outside.
func CheckTick(t int64) int64 {
	if !HasTick(t) {
		panic(fmt.Sprintf("not a tick: %d", t))
	}
	return t
}

// splitTick splits a tick into its utc parts, or panics because the caller handed it something that isn't a tick, which is a bug in the caller: a tick is zero through the last millisecond of 9999
func splitTick(t int64) parts {
	u := time.UnixMilli(CheckTick(t)).UTC()
	return parts{u.Year(), int(u.Month()), u.Day(), u.Hour(), u.Minute(), u.Second()*1000 + u.Nanosecond()/1e6}
}

// joinTick is the tick at the given utc parts, or false if there's no such moment, like february 30, or it's outside the range of ticks
func joinTick(p parts) (int64, bool) {
	if p.y < 1970 || p.y > 9999 || p.m < 1 || p.m > 12 || p.d < 1 || p.d > 31 || p.h > 23 || p.i > 59 || p.ms > 59999 {
		return 0, false
	}
	u := time.Date(p.y, time.Month(p.m), p.d, p.h, p.i, 0, 0, time.UTC) // a day past the month's end rolls into the next month, and the round trip in the reader catches that
	t := u.UnixMilli() + int64(p.ms)
	return t, 0 <= t && t <= lastTick
}

// numerals is the number a text of numerals 0 through 9 names, or false if it's blank, has anything else in it, or is too long for an int
func numerals(s string) (int, bool) {
	if s == "" {
		return 0, false
	}
	for i := range len(s) {
		if s[i] < '0' || s[i] > '9' {
			return 0, false
		}
	}
	n, err := strconv.Atoi(s)
	return n, err == nil
}

// TickToISO writes a tick in the iso form, like "2012-08-06T05:17:57.300Z". It panics if t isn't a tick, which is a bug in the caller; ask HasTick first about a number from outside.
func TickToISO(t int64) string {
	p := splitTick(t)
	return fmt.Sprintf("%04d-%02d-%02dT%02d:%02d:%02d.%03dZ", p.y, p.m, p.d, p.h, p.i, p.ms/1000, p.ms%1000)
}

// ISOToTick reads the iso form back to its tick, or returns an error because the text isn't exactly what a tick writes.
func ISOToTick(s string) (int64, error) {
	t, ok := parseISO(s)
	if !ok || TickToISO(t) != s { // the round trip is the final check, catching february 30 after time.Date rolled it into march
		return 0, fmt.Errorf("ergo: not the iso form of a tick: %q", s)
	}
	return t, nil
}

// parseISO reads the iso form leniently enough to compute a tick, or false; ISOToTick above makes it strict with the round trip
func parseISO(s string) (int64, bool) {
	const shape = "0000-00-00T00:00:00.000Z" // a 0 is a numeral, and anything else is itself
	if len(s) != len(shape) {
		return 0, false
	}
	for i := range len(shape) { // byte by byte, so a multibyte character fails here rather than lining up by accident
		if (shape[i] == '0' && (s[i] < '0' || s[i] > '9')) || (shape[i] != '0' && s[i] != shape[i]) {
			return 0, false
		}
	}
	n := func(a, z int) int { v, _ := strconv.Atoi(s[a:z]); return v } // numerals only here, so Atoi can't fail
	return joinTick(parts{n(0, 4), n(5, 7), n(8, 10), n(11, 13), n(14, 16), n(17, 19)*1000 + n(20, 23)})
}

// TickToText writes a tick in the short form, like "2012aug6.0517.57300", as deep as the moment needs and no deeper. It panics if t isn't a tick, which is a bug in the caller; ask HasTick first about a number from outside.
func TickToText(t int64) string {
	p := splitTick(t)
	depth := 0 // how deep the text must go: the year alone, then the month, day, hour, minute, and millisecond
	if p.m > 1 {
		depth = 1
	}
	if p.d > 1 {
		depth = 2
	}
	if p.h > 0 {
		depth = 3
	}
	if p.i > 0 {
		depth = 4
	}
	if p.ms > 0 {
		depth = 5
	}
	text := strconv.Itoa(p.y)
	if depth >= 1 {
		text += shortMonths[p.m-1] // the short name in lowercase, like "aug"
	}
	if depth >= 2 {
		text += strconv.Itoa(p.d)
	}
	if depth >= 3 {
		text += fmt.Sprintf(".%02d", p.h)
	}
	if depth >= 4 {
		text += fmt.Sprintf("%02d", p.i)
	}
	if depth >= 5 {
		text += "." + strconv.Itoa(p.ms)
	}
	return text
}

// TextToTick reads the short form back to its tick, or returns an error because the text isn't exactly what a tick writes.
func TextToTick(s string) (int64, error) {
	t, ok := parseText(s)
	if !ok || TickToText(t) != s { // the round trip is the final check, catching february 30 and a padded or deeper text alike
		return 0, fmt.Errorf("ergo: not the short form of a tick: %q", s)
	}
	return t, nil
}

// parseText reads the short form leniently enough to compute a tick, or false; TextToTick above makes it strict with the round trip
func parseText(s string) (int64, bool) {
	pieces := strings.Split(s, ".") // the year, month, and day; then the clock; then the milliseconds into the minute
	if len(pieces) > 3 {
		return 0, false
	}
	p1 := pieces[0]
	if !slices.Contains([]int{4, 7, 8, 9}, len(p1)) { // "1990", "1990feb", "1990feb2", "1990feb28"
		return 0, false
	}
	y, ok := numerals(p1[:4])
	if !ok {
		return 0, false
	}
	p := parts{y, 1, 1, 0, 0, 0}
	if len(p1) >= 7 {
		p.m = slices.Index(shortMonths[:], p1[4:7]) + 1 // 0 if the month name isn't one of ours, which joinTick refuses
	}
	if len(p1) >= 8 {
		if p.d, ok = numerals(p1[7:]); !ok {
			return 0, false
		}
	}
	if len(pieces) >= 2 {
		p2 := pieces[1]
		if len(p2) != 2 && len(p2) != 4 { // "09" an hour, "0905" an hour and minute
			return 0, false
		}
		if p.h, ok = numerals(p2[:2]); !ok {
			return 0, false
		}
		if len(p2) == 4 {
			if p.i, ok = numerals(p2[2:]); !ok {
				return 0, false
			}
		}
	}
	if len(pieces) >= 3 {
		if p.ms, ok = numerals(pieces[2]); !ok {
			return 0, false
		}
	}
	return joinTick(p)
}

// SayTick writes a tick for a person to read, in whatever shape a template of tags describes. "Mmm D, YYYY" says a date like "Aug 6, 2026" on a page, "YYYY-MM-DD hh24:mm:ss.ttt" stamps a line in a log, and the iso form is just "YYYY-MM-DDThh24:mm:ss.tttZ". It's the one-way companion to the two forms above, which each read back exactly. This one only writes, so a template can take any shape at all.
//
// Uppercase tags are the calendar, and lowercase tags are the clock. YYYY is the year. M and MM are the month's number, without and with a leading zero, and MMM and MMMM are its name, short and long. D and DD are the day of the month, and DDD and DDDD are the day of the week. A name comes out in the case of its tag, so MMMM is AUGUST, Mmmm is August, and mmmm is august.
//
// On the clock, h12 and h24 are the hour, each saying which clock it means, and hh12 and hh24 are the same with a leading zero. mm and ss are the minute and the second, always two digits, with no form that leaves the zero off, since five past one is 1:05 and never 1:5. t, tt, and ttt are tenths, hundredths, and thousandths of a second, as many digits as letters, rounded down so 59.999 never becomes 60. AA, aa, A, and a are PM, pm, P, and p, and it's up to the template to put them beside a twelve-hour hour.
//
// The two cases also keep the calendar and the clock apart on the page, so in "YYYY-MM-DD hh24:mm", a lowercase mm among the calendar's capitals, or a capital MM among the clock's lowercase, stands out as the mistake it is.
//
// The template is read once, left to right, taking the longest tag at each position, and whatever isn't a tag is written as it is. There's no escaping, so "YYYYY" writes the year and then a Y, like "2026Y". Each value goes straight to the output and is never read again, so a value can't make a tag: the A in AUG is only a letter. Words don't belong in a template, since a single M, D, t, A, or a is a tag, and so is a doubled mm or ss. Say the date with a small template, and put what it writes into the sentence, page, or log line around it.
//
// A tick is an int64, so it has no fraction of a millisecond to drop, the way the editions that take a double do.
//
// Some things are left out on purpose. There's no reader, since the two forms above are for reading. There's no two-digit year. There are no time zones: SayTick writes the fields of the tick it's given as they fall in UTC, and works the same for a moment anywhere on earth. Keep times in UTC, and if a person should see their own wall clock, shift the tick by their offset at the very end, just before saying it. And there's no localization: the names are English, the same everywhere, and a page that wants its reader's language can hand the tick to the browser's Intl.

// sayTags is every tag, each longer one before its prefixes, since alternation takes the first that matches at a position rather than the longest
var sayTags = regexp.MustCompile(`YYYY|MMMM|Mmmm|mmmm|MMM|Mmm|mmm|MM|M|DDDD|Dddd|dddd|DDD|Ddd|ddd|DD|D|hh12|hh24|h12|h24|mm|ss|ttt|tt|t|AA|aa|A|a`)

// SayTick writes tick t for a person to read, in the shape a template of tags describes, like "Mmm D, YYYY" for "Aug 6, 2026"; one way, with no reader. It panics if t isn't a tick, which is a bug in the caller; ask HasTick first about a number from outside.
func SayTick(t int64, template string) string {
	u := time.UnixMilli(CheckTick(t)).UTC()
	return sayTags.ReplaceAllStringFunc(template, func(tag string) string { return sayTag(u, tag) }) // one pass over the template; each value goes to the output and is never read again
}

// sayTag is the text for one tag, from the moment in u as it falls in utc
func sayTag(u time.Time, tag string) string {
	hour := u.Hour()  // 0 through 23
	half := hour % 12 // the hour on a twelve-hour clock, where midnight and noon are 12 rather than 0
	if half == 0 {
		half = 12
	}
	ms := u.Nanosecond() / 1e6 // 0 through 999

	switch tag {
	case "YYYY":
		return strconv.Itoa(u.Year()) // always four digits, since a tick stays within 1970 through 9999
	case "MM":
		return fmt.Sprintf("%02d", int(u.Month()))
	case "M":
		return strconv.Itoa(int(u.Month()))
	case "DD":
		return fmt.Sprintf("%02d", u.Day())
	case "D":
		return strconv.Itoa(u.Day())
	case "hh24":
		return fmt.Sprintf("%02d", hour)
	case "h24":
		return strconv.Itoa(hour)
	case "hh12":
		return fmt.Sprintf("%02d", half)
	case "h12":
		return strconv.Itoa(half)
	case "mm":
		return fmt.Sprintf("%02d", u.Minute())
	case "ss":
		return fmt.Sprintf("%02d", u.Second())
	case "ttt":
		return fmt.Sprintf("%03d", ms)
	case "tt":
		return fmt.Sprintf("%02d", ms/10) // rounded down, like t
	case "t":
		return strconv.Itoa(ms / 100) // rounded down, so 59.99 is 59.9 and never 60.0
	}

	// the rest are words: the month's name, the weekday's name, or am and pm, each in the case of its tag
	var word string
	switch strings.ToLower(tag[:1]) { // m for the month, d for the weekday, a for am and pm
	case "m":
		word = monthNames[u.Month()-1]
	case "d":
		word = dayNames[u.Weekday()]
	default:
		word = "PM"
		if hour < 12 {
			word = "AM"
		}
	}
	if len(tag) < 4 {
		word = word[:len(tag)] // a shorter tag writes as many letters as it has: Aug for Mmm, P for A, and both letters of PM for AA
	}
	return caseLike(word, tag)
}

// caseLike writes a word in the case of the tag that asked for it: AUGUST for MMMM, August for Mmmm, and august for mmmm
func caseLike(word, tag string) string {
	if tag == strings.ToUpper(tag) {
		return strings.ToUpper(word) // every letter a capital, like MMMM, AA, and A
	}
	if tag == strings.ToLower(tag) {
		return strings.ToLower(word) // none, like mmmm, aa, and a
	}
	return word // only the first, like Mmmm, which is how the names are kept
}

// TagLength is long enough to be unique, short enough to be reasonable, and nanoid's default.
const TagLength = 21

// letters and numerals only, so a double-click selects the whole tag; nanoid's default alphabet also has - and _
const tagAlphabet = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz"

// Tag generates a new universally unique tag of 21 letters and numerals, like "AgKxru95C7jFp5iPuK9O7".
func Tag() string {
	// the algorithm is nanoid's: mask random bytes down to the alphabet's size, and skip the few values that land past its end, so every character is equally likely
	const mask = 63                                                            // the alphabet has 62 characters; 63 is the smallest all-ones mask that covers them
	step := int(math.Ceil(1.6 * mask * TagLength / float64(len(tagAlphabet)))) // random bytes per draw; 1.6 is nanoid's margin so one draw almost always suffices
	tag := make([]byte, 0, TagLength)
	bytes := make([]byte, step)
	for {
		rand.Read(bytes) // since go 1.24, crypto/rand.Read never fails and always fills the whole slice
		for _, b := range bytes {
			i := int(b & mask)        // 0 through 63
			if i < len(tagAlphabet) { // 62 and 63 fall past the end; skipping them keeps the distribution uniform
				tag = append(tag, tagAlphabet[i])
			}
			if len(tag) == TagLength {
				return string(tag)
			}
		}
	}
}

// HasTag is true if s is exactly 21 letters and numerals.
func HasTag(s string) bool {
	if len(s) != TagLength { // 21 bytes that are all ascii are 21 characters
		return false
	}
	for i := range len(s) {
		b := s[i]
		if !('0' <= b && b <= '9' || 'A' <= b && b <= 'Z' || 'a' <= b && b <= 'z') {
			return false
		}
	}
	return true
}

// CheckTag is a guard: it passes a tag through, or panics because the caller handed it something that isn't one, which is a bug in the caller, not an ordinary answer to handle.
func CheckTag(s string) string {
	if !HasTag(s) {
		panic(fmt.Sprintf("not a tag: %q", s))
	}
	return s
}

// A Data holds bytes. Every language keeps bytes its own way, in arrays and buffers and views with their own rules about copying and size, and every platform call that takes or gives bytes wants its own one of them. A Data makes those choices once: it holds the bytes, and it speaks every form they travel in, text and base16, base32, base62, and base64, through methods with plain names. Make one from any form, read it out as any other, and it's the same bytes. In Go a Data holds a slice, so making one from a slice, or clipping a view of part of one, never copies the bytes.
//
// Every function in ergo that trades in bytes trades in Data. A hash takes a Data and answers one, and never decides what text the caller wants next; the caller says, with the next method. Only Data knows the encodings.
//
// Each form is exact, both ways. Every reader ends by writing the bytes back out and requiring the text it read, so it accepts only what its writer produces: base16 in lowercase, base32 without padding, base64 with it, nothing extra and nothing missing. Text becomes UTF-8 in Unicode Normalization Form C, so the same characters typed two ways become the same bytes, and reading bytes as text refuses bytes that aren't UTF-8, or aren't in that form, rather than guess. A reader refuses with an error, the way the tick readers do.
//
// ClipView looks at part of a Data without copying it, which is fast, and right whenever the bytes beneath won't change. ClipCopy copies them out, for when they could. Array hands over the slice itself.
//
// A Data can hold no bytes at all: the empty input every hash has a standard answer for, the bytes of blank text, a clip of nothing, an empty Bin.
//
// The standard library has base16, base32, and base64, and golang.org/x/text has Normalization Form C, which the standard library leaves out; base62 is ergo's own, and the few lines of it are here.

// Data is bytes, made from exactly one form, like DataFromText("hello") or DataFromBase16("68656c6c6f"), and read out as any: Array, Text, Base16, Base32, Base62, or Base64. The zero Data is no bytes at all.
type Data struct {
	b []byte
}

// DataFromArray makes a Data of the given bytes, which it looks at without copying.
func DataFromArray(b []byte) Data {
	return Data{b}
}

// DataFromText makes a Data of text, as UTF-8 in Normalization Form C, so the same characters typed two ways become the same bytes. It panics if s isn't UTF-8, which a Go string can hold, a bug in the caller; utf8.ValidString asks first.
func DataFromText(s string) Data {
	if !utf8.ValidString(s) {
		panic(fmt.Sprintf("not text: %q", s))
	}
	return Data{[]byte(norm.NFC.String(s))}
}

// DataFromBase16 reads a Data from base16, or returns an error because the text isn't exactly what Base16 writes, two lowercase digits a byte.
func DataFromBase16(s string) (Data, error) {
	b, err := hex.DecodeString(s)
	return roundTrip("base16", s, b, err, hex.EncodeToString) // the round trip refuses uppercase, which DecodeString takes
}

// DataFromBase32 reads a Data from base32, or returns an error because the text isn't exactly what Base32 writes, uppercase and without padding.
func DataFromBase32(s string) (Data, error) {
	b, err := base32Encoding.DecodeString(s)
	return roundTrip("base32", s, b, err, base32Encoding.EncodeToString) // the round trip refuses the newlines DecodeString skips, and a last character with bits that belong to no byte
}

// DataFromBase62 reads a Data from base62, or returns an error because the text isn't exactly what Base62 writes.
func DataFromBase62(s string) (Data, error) {
	b, err := base62ToBytes(s)
	return roundTrip("base62", s, b, err, bytesToBase62)
}

// DataFromBase64 reads a Data from base64, or returns an error because the text isn't exactly what Base64 writes, with its padding.
func DataFromBase64(s string) (Data, error) {
	b, err := base64.StdEncoding.DecodeString(s)
	return roundTrip("base64", s, b, err, base64.StdEncoding.EncodeToString) // the round trip refuses the newlines DecodeString skips, and a last character with bits that belong to no byte
}

// RandomData makes a Data of n bytes from the operating system's random source. It panics if n is negative, a bug in the caller.
func RandomData(n int) Data {
	if n < 0 {
		panic(fmt.Sprintf("not a count: %d", n))
	}
	b := make([]byte, n)
	rand.Read(b) // since go 1.24, crypto/rand.Read never fails and always fills the whole slice
	return Data{b}
}

// Size is the size in bytes.
func (d Data) Size() int {
	return len(d.b)
}

// Array is the bytes themselves, which this doesn't copy.
func (d Data) Array() []byte {
	return d.b
}

// Text is the bytes as text, or an error because they aren't UTF-8 in Normalization Form C, since other bytes would become different bytes as text. A leading byte order mark stays the character it is.
func (d Data) Text() (string, error) {
	if !utf8.Valid(d.b) || !norm.NFC.IsNormal(d.b) {
		return "", fmt.Errorf("ergo: not text: %s", d.Base16())
	}
	return string(d.b), nil
}

// Base16 is the bytes in base16, two lowercase digits each.
func (d Data) Base16() string {
	return hex.EncodeToString(d.b)
}

// Base32 is the bytes in base32, uppercase and without padding.
func (d Data) Base32() string {
	return base32Encoding.EncodeToString(d.b)
}

// Base62 is the bytes in base62.
func (d Data) Base62() string {
	return bytesToBase62(d.b)
}

// Base64 is the bytes in base64, with its padding.
func (d Data) Base64() string {
	return base64.StdEncoding.EncodeToString(d.b)
}

// Get is the byte at index i, a number 0x00 0 through 0xff 255. It panics if there's no byte at i, a bug in the caller; Size asks first.
func (d Data) Get(i int) byte {
	if i < 0 || i >= len(d.b) {
		panic(fmt.Sprintf("no byte %d in %d bytes", i, len(d.b)))
	}
	return d.b[i]
}

// ClipView is the bytes from index start up to end, looked at without copying; fast, and right when the bytes beneath won't change. It panics unless 0 <= start <= end <= Size, a bug in the caller. A clip of nothing is fine.
func (d Data) ClipView(start, end int) Data {
	checkClip(len(d.b), start, end)
	return Data{d.b[start:end:end]} // capped at end, so appending to the clip can never write over the bytes after it
}

// ClipCopy is the bytes from index start up to end, copied out, for when the bytes beneath could change 🪏. It panics like ClipView.
func (d Data) ClipCopy(start, end int) Data {
	checkClip(len(d.b), start, end)
	return Data{slices.Clone(d.b[start:end])}
}

// checkClip panics unless start through end is a clip of size bytes: a clip of nothing is fine, but not one past the end, or backwards
func checkClip(size, start, end int) {
	if start < 0 || start > end || end > size {
		panic(fmt.Sprintf("not a clip of %d bytes: %d to %d", size, start, end))
	}
}

// roundTrip is the last check in every reader: write the bytes back out, and require exactly the text it read
func roundTrip(form, s string, b []byte, err error, write func([]byte) string) (Data, error) {
	if err != nil || write(b) != s {
		return Data{}, fmt.Errorf("ergo: not %s: %q", form, s)
	}
	return Data{b}, nil
}

// base32, for hash values in a database, so they can be short, but also always the same length; the standard alphabet, without padding, as the code ergo grew from writes it
var base32Encoding = base32.StdEncoding.WithPadding(base32.NoPadding)

// Why so many different encoding formats? base16 is good because it's simple, standard, and you can see the bytes. base32 is shorter, double-clickable, and always the same length, good for storing hash values in the database. base62 is shorter and double-clickable, good for bytes in the query string of api calls and the location bar the user might see. base64 is about the same length, standard, but not double-clickable, included here for completeness.

// narrow characters in popular spots, for data
const alphabet62Stream = "1023456789ABCDEFIGHJKLMNOPQRSTUVjWXYZabcdefghkmnlopqrstuvwxyzi"

//                        ^               ^               ^               ^            ^
//                        1               I               j               l            i  moved characters
//                        0               16              32              48           61  to these indices
//
// Encoding random short data, some characters appear more frequently than others.
// The last character, at index 61, is the most common.
// Also, the characters at 0, 16, 32, and 48 are slightly more common.
// Rendered to pixels for a user, we want base62 text to be as short as possible!
// So, we've adjusted the alphabet to put narrow characters in the popular spots.

// bytesToBase62 turns data into text using base 62, each 4 or 6 bits will become a character 0-9, a-z, and A-Z
func bytesToBase62(a []byte) string {
	var s strings.Builder       // target string to build up and return
	for i := 0; i < len(a)*8; { // the index in bits, from 0 through all the bits in the given data
		byteIndex, bitIndex := i/8, i%8 // the same index as a distance in bytes followed by a distance in bits

		// copy the two bytes at byteIndex into pair; on the very last byte, leave the right byte in pair all 0s
		pair := uint32(a[byteIndex]) << 8
		if byteIndex+1 < len(a) {
			pair |= uint32(a[byteIndex+1])
		}

		// read the 6 bits at i as a number, called code, which will be 0 through 63
		code := (pair >> (10 - bitIndex)) & 63

		// describe the 6 bits with a numeral or letter, 111100 is 60, if 61 move forward 4, not 6
		if code < 61 {
			s.WriteByte(alphabet62Stream[code]) // 000000 0 through 111100 60 append character at that index
			i += 6
		} else {
			s.WriteByte(alphabet62Stream[61]) // 111101 61, 111110 62, and 111111 63 append 'i' and move past '1111'
			i += 4
		}
	}
	return s.String()
}

// base62ToBytes reads base62 back into bytes, or false for a character that isn't in the alphabet; DataFromBase62 adds the round trip, which refuses a last character with bits that belong to no byte
func base62ToBytes(s string) ([]byte, error) {
	b := NewBin(len(s)) // more characters become fewer bytes, so this capacity will always be plenty
	var hold uint32     // a place to hold bits from several characters until we have 8 and can write a byte
	bits := 0           // the number of bits stored in the right side of hold right now
	for i := range len(s) {
		code := strings.IndexByte(alphabet62Stream, s[i])
		if code < 0 {
			return nil, fmt.Errorf("ergo: not base62: %q", s)
		}

		// insert the bits from code into hold
		if code == 61 {
			hold = hold<<4 | 15 // insert 1111 for common 'i' at last alphabet index 61
			bits += 4
		} else {
			hold = hold<<6 | uint32(code) // insert 000000 through 111100 for other characters
			bits += 6
		}

		// if we have enough bits in hold to write a byte
		if bits >= 8 {
			b.AddByte(byte(hold >> (bits - 8))) // move the 8 leftmost bits in hold to our Bin object
			bits -= 8                           // remove the bits we wrote from hold, any extra bits there will be written next time
		}
	}
	return b.Array(), nil
}

// Bin is a set number of bytes, which you add a byte or a Data at a time until it's full. It's how to assemble bytes whose size you know before you start, and refusing to hold more than that catches a mistake in the prediction.
type Bin struct {
	b []byte
}

// NewBin makes a Bin that can hold capacity bytes, holding none yet. It panics if capacity is negative, a bug in the caller.
func NewBin(capacity int) *Bin {
	if capacity < 0 {
		panic(fmt.Sprintf("not a count: %d", capacity))
	}
	return &Bin{make([]byte, 0, capacity)}
}

// Capacity is how many bytes it can hold.
func (b *Bin) Capacity() int {
	return cap(b.b)
}

// Size is how many bytes it does hold.
func (b *Bin) Size() int {
	return len(b.b)
}

// Add adds all of a Data. It panics if it won't fit, which means the size predicted for this Bin was wrong.
func (b *Bin) Add(d Data) {
	if len(b.b)+len(d.b) > cap(b.b) {
		panic(fmt.Sprintf("a Bin of %d bytes holding %d can't take %d more", cap(b.b), len(b.b), len(d.b)))
	}
	b.b = append(b.b, d.b...)
}

// AddByte adds one byte. It panics if the Bin is already full.
func (b *Bin) AddByte(c byte) {
	if len(b.b) == cap(b.b) {
		panic(fmt.Sprintf("a Bin of %d bytes is full", cap(b.b)))
	}
	b.b = append(b.b, c)
}

// Array is what it holds so far, which this doesn't copy.
func (b *Bin) Array() []byte {
	return b.b[:len(b.b):len(b.b)] // capped, so appending to it can never write into the room the Bin has left
}

// Data is what it holds so far, as a Data that looks at it without copying; what the Bin adds later goes after it, never into it.
func (b *Bin) Data() Data {
	return Data{b.Array()}
}

// Three hashes, each the one the world already knows. SHA-1 is the oldest, the one older manifests name a file by and git names its objects with. It's broken for collisions, two different inputs made to share a hash on purpose, so use it to match a hash that already exists, never to protect anything. SHA-256 is the standard everywhere today. BLAKE3 is the newest, built for speed as a tree, so pieces of one input can hash on many cores at once, and it's the one to name a file by for years to come.
//
// Each hash takes a Data and answers one, and comes in two forms that give the same answer. The function takes all the bytes at once, like SHA256(data), for data already in memory. The hasher takes them a piece at a time, as a file or a stream delivers them: make one with NewSHA256, Update it with each piece in order, and call Digest for the hash. A Hasher is also an io.Writer, so io.Copy can feed it from a file, and io.MultiWriter can feed all three hashers from a single read, the raw bytes of each read going in with no Data made of them. A file of any size hashes this way in the memory of one piece, and Digest doesn't finish a hasher: update it more and Digest again for the hash of everything so far.
//
// The standard library has SHA-1 and SHA-256 and no BLAKE3, which x/crypto has yet to decide on, so BLAKE3 comes from lukechampine.com/blake3, the implementation written with x/crypto in mind. It already does what BLAKE3's tree allows, the one edition that does on its own: the subtrees of a large input hash on goroutines of their own, across every core. Each edition takes each hash from its platform wherever the platform has it, and otherwise from the official implementation or the best known one, and the shared vectors and the fuzz test hold all four to the same answers.

// SHA1 is the SHA-1 hash of a Data, as a Data of 20 bytes. It's for matching a hash that already exists, like an older manifest's, since SHA-1 is broken for collisions.
func SHA1(data Data) Data {
	h := sha1.Sum(data.b)
	return Data{h[:]}
}

// SHA256 is the SHA-256 hash of a Data, as a Data of 32 bytes.
func SHA256(data Data) Data {
	h := sha256.Sum256(data.b)
	return Data{h[:]}
}

// Blake3 is the BLAKE3 hash of a Data, as a Data of 32 bytes.
func Blake3(data Data) Data {
	h := blake3.Sum256(data.b)
	return Data{h[:]}
}

// HashText is the SHA-256 hash of text, as base32, the shortcut for the common case: text to Data, hashed, and out in the form a database keeps hash values in. It panics like DataFromText if s isn't UTF-8.
func HashText(s string) string {
	return SHA256(DataFromText(s)).Base32()
}

// Hasher takes bytes a piece at a time: Update with each piece in order, each a Data, then Digest for the hash of all of them as a Data. Digest doesn't finish it: update more and Digest again for the hash of everything so far. It's also an io.Writer, which takes the raw bytes of each write, so io.Copy can feed one from a file and io.MultiWriter can feed several from a single read.
type Hasher struct {
	h hash.Hash
}

// NewSHA1 makes a SHA-1 Hasher, for the same Data of 20 bytes SHA1 gives all at once.
func NewSHA1() *Hasher {
	return &Hasher{sha1.New()}
}

// NewSHA256 makes a SHA-256 Hasher, for 32 bytes.
func NewSHA256() *Hasher {
	return &Hasher{sha256.New()}
}

// NewBlake3 makes a BLAKE3 Hasher, for 32 bytes.
func NewBlake3() *Hasher {
	return &Hasher{blake3.New(32, nil)} // 32 bytes of output and no key, the plain hash
}

// Update hashes the next piece.
func (h *Hasher) Update(d Data) {
	h.h.Write(d.b) // a hash.Hash never returns an error from Write
}

// Digest is the hash of every piece so far, as a Data.
func (h *Hasher) Digest() Data {
	return Data{h.h.Sum(nil)}
}

// Write hashes p, the next piece as raw bytes, so a Hasher is an io.Writer; it never fails.
func (h *Hasher) Write(p []byte) (int, error) {
	return h.h.Write(p)
}

// The fuji hashes are two of ergo's own, from the original, the JavaScript ergo grew from, and both are SHA-256 over a summary of a file rather than over the file itself. The piece hash covers every byte: it cuts the file into 4 MiB pieces, hashes each one, and hashes the list of those hashes. The tip hash covers almost none of them: it hashes the file's first 4 KiB, 4 KiB from its middle, and its last two, at most 16 KiB however large the file, so it's done in a few reads and finds likely copies of a file across millions of others for the cost of those few reads each. A file of 16 KiB or less is its own tip. Each summary starts with a title that names the protocol and the file's size, like "Fuji.Tips.SHA256.4KiB.5.", so two files of different sizes never share a hash, even with identical tips.
//
// The pieces are there by design, for JavaScript, whose Web Crypto hashes only all at once: hashing 4 MiB pieces and then the list of them lets a page hash a file of any size at Web Crypto's full speed. Here each piece fills a slice of its own, which a Data then holds without a copy, where JavaScript slides a conveyor belt. The tip hash can come two ways that must agree: from a file, by reading just the stripes at their offsets, or from a stream, by noticing the stripes as they pass on the way to the piece hash.

// HashMeasurement is the pieces of a file at a unit, and the stripes the tip hash reads, each a start and end measured from the start of the file, not a start and a length.
type HashMeasurement struct {
	Pieces     int64      // total number of pieces
	Complete   int64      // quantity of complete pieces
	Fragment   int64      // byte size of last piece if it is a fragment
	Stripes    [][2]int64 // the stripes the tip hash reads
	StripeSize int64      // how many bytes all the stripes hold
}

// HashMeasure, given a file size and a unit, measures the pieces of that size, and the stripes the tip hash reads 📏. It panics unless both the file size and the unit are 1 or more bytes, a bug in the caller.
func HashMeasure(file, unit int64) HashMeasurement {
	if file < 1 || unit < 1 {
		panic(fmt.Sprintf("not a file and unit to measure: %d, %d", file, unit))
	}
	m := HashMeasurement{Pieces: (file + unit - 1) / unit, Complete: file / unit, Fragment: file % unit}
	if file <= 4*unit { // file size is 4 units or less, so just hash the whole thing
		m.Stripes = [][2]int64{{0, file}}
		m.StripeSize = file
		return m
	}
	first := int64(0)
	middle := file / (2 * unit) * unit   // byte index of middle piece
	penultimate := (m.Pieces - 2) * unit // second to last piece, must always be complete
	lastSize := unit                     // how big the last stripe is
	if m.Fragment > 0 {
		lastSize = m.Fragment
	}
	m.Stripes = [][2]int64{{first, first + unit}} // first and middle never touch
	if middle+unit == penultimate {               // but middle and penultimate can, and penultimate and last always do
		m.Stripes = append(m.Stripes, [2]int64{middle, middle + unit + unit + lastSize})
	} else { // middle with space before and after
		m.Stripes = append(m.Stripes, [2]int64{middle, middle + unit}, [2]int64{penultimate, penultimate + unit + lastSize})
	}
	m.StripeSize = 3*unit + lastSize
	return m
}

// HashStripeOverlap, given a foreground stripe and a background pattern of stripes that don't touch or overlap, is the ranges where they overlap, and an empty list for none.
func HashStripeOverlap(foreground [2]int64, background [][2]int64) [][2]int64 {
	overlaps := [][2]int64{}
	a, b := foreground[0], foreground[1]
	for _, s := range background {
		if c, d := s[0], s[1]; a < d && b > c { // there's an overlap!
			overlaps = append(overlaps, [2]int64{max(a, c), min(b, d)})
		}
	}
	return overlaps
}

// HashUnit is one of a protocol's two units: the title its summary starts with, and the size of a piece or a stripe.
type HashUnit struct {
	Title string
	Size  int64
}

// HashProtocol is the units of the piece hash and the tip hash.
type HashProtocol struct {
	Pieces HashUnit
	Tips   HashUnit
}

// DefaultHashProtocol is the fuji protocol, a new simple protocol that can hash huge files on both the server and the page 🗻.
//
// 4 MiB hashes in ~10ms similar to the frequency of page animation frames, uploads in under a second over a typical cable modem, and keeps the index of hash values smaller than the piece size for files under 500 GB 🗄️. 4 KiB matches the cluster and sector size on Windows NTFS and the block size on Mac APFS, so hard drive reads can be as fast as possible 💽.
var DefaultHashProtocol = HashProtocol{
	Pieces: HashUnit{"Fuji.Pieces.SHA256.4MiB.", 4 * MB},
	Tips:   HashUnit{"Fuji.Tips.SHA256.4KiB.", 4 * KB},
}

// HashStatus is what HashFile and HashStream know so far, given to a progress callback as it changes and returned at the end.
type HashStatus struct {
	TotalSize  int64 // the size the file was said to be
	HashedSize int64 // how much of it has been hashed
	TipHash    Data  // the tip hash, once it's done
	PieceHash  Data  // the piece hash, once it's done, from HashStream only
	StartTime  int64 // a tick, milliseconds since 1970
	UpdateTime int64 // when we last changed anything here
	Duration   int64 // milliseconds, once it's done
}

// HashFile, given a file to read at offsets and its size, quickly computes just the tip hash with the fuji protocol, reading only the stripes; a HashProtocol's own HashFile takes another protocol. It panics if size is zero, a bug in the caller: an empty file has no fuji hash.
func HashFile(file io.ReaderAt, size int64) (HashStatus, error) {
	return DefaultHashProtocol.HashFile(file, size)
}

// HashStream, given a stream to read and its size, computes both the tip hash and the piece hash with the fuji protocol in one pass, telling onProgress, which may be nil, as each piece is hashed; a HashProtocol's own HashStream takes another protocol. It panics if size is zero, a bug in the caller.
func HashStream(stream io.Reader, size int64, onProgress func(HashStatus)) (HashStatus, error) {
	return DefaultHashProtocol.HashStream(stream, size, onProgress)
}

// HashFile is the package's HashFile with this protocol.
func (p HashProtocol) HashFile(file io.ReaderAt, size int64) (HashStatus, error) {
	// based on the file size, pick stripes at the start, middle, and end for us to hash quickly
	measure := HashMeasure(size, p.Tips.Size)
	status := newStatus(size)

	// for tip hashing, the summary we'll hash is the title followed by stripes of file data (hashing file data, not hashes); different sized files hash differently even with identical tips
	title := DataFromText(fmt.Sprintf("%s%d.", p.Tips.Title, size))
	tips := NewBin(title.Size() + int(measure.StripeSize))
	tips.Add(title)
	for _, s := range measure.Stripes {
		stripe := make([]byte, s[1]-s[0])
		if n, err := file.ReadAt(stripe, s[0]); n < len(stripe) { // ReadAt may answer io.EOF beside a full read of the last stripe, which is fine
			return status, fmt.Errorf("ergo: reading a stripe at %d: %w", s[0], err)
		}
		tips.Add(DataFromArray(stripe))
	}

	// hash the summary of the file in the bin, which must be exactly full, a check on our prediction of how much data we'd hash
	status.TipHash = SHA256(full(tips))
	status.HashedSize = measure.StripeSize // we actually hashed more because of the title, but don't count this as to speed
	finish(&status)
	return status, nil
}

// HashStream is the package's HashStream with this protocol.
func (p HashProtocol) HashStream(stream io.Reader, size int64, onProgress func(HashStatus)) (HashStatus, error) {
	hasher := p.NewFujiHasher(size)
	status := newStatus(size)
	progress := func() {
		if onProgress != nil {
			onProgress(status)
		}
	}

	buffer := make([]byte, 64*KB) // each read from the stream lands here
	var fed int64                 // how many bytes we've given the hasher
	for fed <= size {             // loop while boxes of data arrive from the stream; past the size, stop reading, and Digest will refuse
		n, err := stream.Read(buffer)
		if n > 0 {
			hasher.Update(DataFromArray(buffer[:n])) // a new delivery of bytes from the stream has arrived! 📦
			fed += int64(n)
			hashed := fed - fed%p.Pieces.Size // each update hashes every piece it fills, so this much is hashed
			if status.HashedSize < hashed && hashed <= size {
				status.HashedSize = hashed
				status.UpdateTime = now()
				progress()
			}
		}
		if err == io.EOF {
			break
		} else if err != nil {
			return status, err
		}
	}

	// hash the summary of the file
	hashes, err := hasher.Digest()
	if err != nil {
		return status, err
	}
	status.HashedSize = size // the fragment piece, if any, is hashed too
	status.TipHash = hashes.TipHash
	status.PieceHash = hashes.PieceHash
	finish(&status)
	progress()
	return status, nil
}

// FujiHashes are both fuji hashes of a file, which a FujiHasher answers.
type FujiHashes struct {
	TipHash   Data // from the stripes at the file's start, middle, and end
	PieceHash Data // from the hashes of all of its pieces
}

// FujiHasher takes a file a piece at a time: make one for a file of a given size, Update it with each piece of the file in order, each a Data, then Digest for both its tip hash and its piece hash. It's for bytes that arrive as events, like the chunks of an upload; HashStream reads a stream into one, and it's an io.Writer, so io.Copy can feed one too. Update never fails, and Digest answers an error when the pieces didn't add up to the size, since that's a fact about the data that came in rather than a bug. Digest doesn't finish it, as with a Hasher.
type FujiHasher struct {
	size      int64
	pieceSize int
	stripes   [][2]int64
	pieces    *Bin
	tips      *Bin
	piece     []byte
	address   int64
}

// NewFujiHasher makes a FujiHasher for a file of size bytes with the fuji protocol; a HashProtocol's own NewFujiHasher takes another protocol. It panics if size is zero, a bug in the caller: an empty file has no fuji hash.
func NewFujiHasher(size int64) *FujiHasher {
	return DefaultHashProtocol.NewFujiHasher(size)
}

// NewFujiHasher is the package's NewFujiHasher with this protocol.
func (p HashProtocol) NewFujiHasher(size int64) *FujiHasher {
	measurePieces := HashMeasure(size, p.Pieces.Size) // measurements for the piece hash
	measureTips := HashMeasure(size, p.Tips.Size)     // and for the tip hash, which we'll peek for as the pieces pass

	// for the pieces hash, the summary we'll hash is the title followed by hashes of pieces of file data, each SHA-256 hash 32 bytes (hashing hashes, not file data)
	piecesTitle := DataFromText(fmt.Sprintf("%s%d.", p.Pieces.Title, size))
	pieces := NewBin(piecesTitle.Size() + 32*int(measurePieces.Pieces))
	pieces.Add(piecesTitle)

	// for the tips hash, the summary we'll hash is the title followed by the start, middle, and end stripes of data
	tipsTitle := DataFromText(fmt.Sprintf("%s%d.", p.Tips.Title, size))
	tips := NewBin(tipsTitle.Size() + int(measureTips.StripeSize))
	tips.Add(tipsTitle)

	pieceSize := int(p.Pieces.Size)
	return &FujiHasher{
		size:      size,
		pieceSize: pieceSize,
		stripes:   measureTips.Stripes,
		pieces:    pieces,
		tips:      tips,
		piece:     make([]byte, 0, pieceSize), // the piece we're filling, which a Data holds without a copy once it's full
	}
}

// Update hashes the next piece.
func (h *FujiHasher) Update(d Data) {
	h.feed(d.b)
}

// Write hashes p, the next piece as raw bytes, so a FujiHasher is an io.Writer; it never fails, and Digest tells of too many bytes or too few.
func (h *FujiHasher) Write(p []byte) (int, error) {
	h.feed(p)
	return len(p), nil
}

// feed takes the next box of bytes 📦
func (h *FujiHasher) feed(delivery []byte) {
	n := len(delivery)
	stripe := [2]int64{h.address, h.address + int64(n)} // where in the file this box of data is
	h.address += int64(n)                               // outside the box, move address past this box to use to mark the start of the next one
	if h.address > h.size {
		return // more bytes than the size, which Digest will refuse; past the size, there's nothing to hash them into
	}

	// first, for the tip hash, we don't have random access through the file, but we do through this box!
	for _, o := range HashStripeOverlap(stripe, h.stripes) {
		h.tips.Add(DataFromArray(delivery[o[0]-stripe[0] : o[1]-stripe[0]])) // subtract box address to clip in data
	}

	// second, for the piece hash, shovel bytes from the box into the piece until the box is empty 🪏
	for shoveled := 0; shoveled < n; {
		shovel := min(n-shoveled, h.pieceSize-len(h.piece))
		h.piece = append(h.piece, delivery[shoveled:shoveled+shovel]...)
		shoveled += shovel
		if len(h.piece) == h.pieceSize {
			h.pieces.Add(SHA256(DataFromArray(h.piece)))
			h.piece = make([]byte, 0, h.pieceSize)
		}
	}
}

// Digest is both hashes of the pieces so far, or an error because they didn't add up to the size.
func (h *FujiHasher) Digest() (FujiHashes, error) {
	if h.address != h.size {
		return FujiHashes{}, fmt.Errorf("ergo: the pieces were %d bytes, not the %d they were said to be", h.address, h.size)
	}

	// most files will end with a fragment piece smaller than a full piece; hash it into a copy of the summary, so the hasher is unchanged and Digest can be asked again
	summary := NewBin(h.pieces.Capacity())
	summary.Add(h.pieces.Data())
	if len(h.piece) > 0 {
		summary.Add(SHA256(DataFromArray(h.piece)))
	}

	// hash the summaries of the file in the bins, which must be exactly full, a check on our prediction of how much data we'd hash
	return FujiHashes{TipHash: SHA256(full(h.tips)), PieceHash: SHA256(full(summary))}, nil
}

// full is the Data a Bin holds, which must be exactly full; a Bin that isn't means this code predicted wrong how much data it would hash, a bug here
func full(b *Bin) Data {
	if b.Size() != b.Capacity() {
		panic(fmt.Sprintf("a Bin of %d bytes holding %d isn't full", b.Capacity(), b.Size()))
	}
	return b.Data()
}

// now is the tick now, milliseconds since 1970
func now() int64 {
	return time.Now().UnixMilli()
}

// newStatus is a status at the start
func newStatus(size int64) HashStatus {
	t := now()
	return HashStatus{TotalSize: size, StartTime: t, UpdateTime: t}
}

// finish sets a status at the end, with its duration
func finish(status *HashStatus) {
	status.UpdateTime = now()
	status.Duration = status.UpdateTime - status.StartTime
}

// Outline: A tree structure where each outline has a name (string), a value (Data, empty for none), and contents (other outlines).
//
// An Outline is like JSON, but stripped down to just one type of value: bytes. No strings vs numbers vs booleans vs nulls — just a name, binary data, and contents. This simplicity makes it useful as a canonical container for cryptographic data, configuration, and structured binary payloads where type richness would be overhead.
//
// The closest inspiration is Bencoding (Bram Cohen, 2001), the serialization format inside BitTorrent .torrent files. Bencoding is minimal and deterministic — dictionaries must have sorted keys, so the same data always produces the same bytes, which is critical when the SHA-1 hash of a bencoded dictionary is the torrent's identity. Outline borrows this philosophy: deterministic sort for canonical comparison, so two Outlines with the same information produce identical binary output and identical hashes. Where Outline improves on Bencoding: a text form that is visible and useful in source code and documentation (Bencoding is barely legible for anything nontrivial), clear separation of name and value (Bencoding overloads byte strings for both keys and data), and a clean tree structure rather than Bencoding's four types (strings, integers, lists, dictionaries).
//
// MessagePack (Furuhashi, 2008), CBOR (RFC 7049, 2013), and Protocol Buffers (Google, 2008) solve a different problem. They are general-purpose serialization formats with rich type systems — integers, floats, strings, arrays, maps, booleans, nulls, extension types. Powerful, but more complex than needed when all your values are just bytes. Protobuf additionally requires schema files compiled ahead of time. Outline is deliberately simpler.
//
// Outline has two serialization formats:
//   - Text: indented lines like «name:"value"0a» using quoted encoding with DataToQuoted. Visible in source code as raw string literals, explains itself in markdown documentation, diffs alongside code in version control. The parser is lenient about indentation width (2 spaces, 4 spaces, tabs), accepts trailing whitespace, blank lines, and # comments at the end of a line or on a line of their own, and takes contents in any order, so pasted or hand-edited text tolerates reasonable sloppiness. The writer is canonical, sorted at every level.
//   - Binary: span-prefixed length encoding (name size, name, value size, value, contents size, contents) with no delimiters or escaping. Compact and unambiguous — data stays the same size on the wire, unlike JSON which inflates binary with escape sequences. Strict, too: the reader takes only what the writer writes, sorted at every level.
//
// One value type (bytes), one tree structure, deterministic sort, a text form and a binary form. It runs anywhere Go does, with no dependencies, no schema files, and no build step.
//
// Rules for designing an outline that you're going to use in your application:
//   - Names are short, lowercase, a-z and 0-9 only. Blank names are fine, duplicate names are fine.
//   - Insertion order cannot matter. Sorting is deterministic and automatic, so the same information always serializes to the same bytes. If your data has ordering that matters, encode the order explicitly--for example, wrap each item in an outline named "1", "2", "3".
//   - Names are labels, not data. Don't generate names from data or encode meaning in them; that's what values are for.
//   - Values are bytes, as raw and granular as possible. No compression, no structured formats, no length-prefixed sub-records; if you have structure, use nested contents instead of packing it into a value.
//   - No version numbers or vendor codes. An outline grows by adding new names; readers ignore names they don't recognize.
//   - Outlines are short packets, not bulk storage. A typical outline is well under 8 KB as binary.
//   - Outlines must be strict trees. No cycles, no shared subtrees--the same outline instance appearing in two places is rejected at serialization time. This matches the text form, which has no way to express sharing.

// HasOutlineName reports whether s is a name an outline can have: lowercase letters and numerals, or blank. It never panics, so a name from outside can be asked about first.
func HasOutlineName(s string) bool {
	for i := 0; i < len(s); i++ {
		c := s[i] // a byte at a time, since every character a name can hold is one byte
		if !('a' <= c && c <= 'z' || '0' <= c && c <= '9') {
			return false
		}
	}
	return true
}

// CheckOutlineName is a guard: it passes an outline name through, or panics because it has a character outside a-z and 0-9, which is a bug in the caller. Ask HasOutlineName first about a name from outside.
func CheckOutlineName(s string) string {
	if !HasOutlineName(s) {
		panic(fmt.Sprintf("not an outline name: %q", s))
	}
	return s
}

// Outline is a tree of outlines, each with a name, a value, and contents, made with NewOutline and filled with Add, AddName, AddValue, and M; the essay above says what it's for and how to design one. The zero Outline has no name, no value, and no contents.
type Outline struct {
	name     string     // outlines can have names, [a-z0-9] only, or no name at all
	value    Data       // and hold one Data value, which holds no bytes when there's no value
	contents []*Outline // and hold contents, a slice of lower nested outlines
}

// NewOutline makes an outline with a name, or none with "", and then contents added to it. It panics if name isn't a name, a bug in the caller; HasOutlineName asks first.
func NewOutline(name string) *Outline {
	return &Outline{name: CheckOutlineName(name)}
}

// Length is how many outlines this one holds in its contents.
func (o *Outline) Length() int {
	return len(o.contents) // how many we have
}

// getters and setters for the name and value of this outline
//
// o.Name() returns o's current name
// o.SetName("") sets o's name to blank, totally fine and useful in Outline
// o.SetName("key1") sets o's name to "key1" only numerals and lowercase letters are allowed
//
// o.Value() gets the value, a Data, which holds no bytes if there's no value here (could still be contents, though!)
// o.SetValue(DataFromText("alice@example.com")) sets the value
// o.SetValue(Data{}) clears the value, since a Data of no bytes is no value; outlines absolutely can be nameless and/or valueless
//
// the name [a-z0-9] requirement may seem restrictive, but prevents the case ambiguity you've likely encountered when examining a JavaScript object of HTTP headers (did the cloud provider lowercase? could there be "origin" *and* "Origin" in here?!!) and discourages overloading values into keys (if you want dots and slashes, build that into nested contents)

// Name is this outline's name, blank if it has none.
func (o *Outline) Name() string {
	return o.name
}

// SetName sets this outline's name, where "" is no name. It panics if k isn't a name, a bug in the caller; HasOutlineName asks first.
func (o *Outline) SetName(k string) {
	o.name = CheckOutlineName(k)
}

// Value is this outline's value, a Data, which holds no bytes if there's no value.
func (o *Outline) Value() Data {
	return o.value
}

// SetValue sets this outline's value, where a Data of no bytes, like Data{}, is no value.
func (o *Outline) SetValue(d Data) {
	o.value = d
}

// Get is the outline at index i in contents. It panics if there's no outline at i, a bug in the caller; Length asks first.
func (o *Outline) Get(i int) *Outline {
	if i < 0 || i >= len(o.contents) {
		panic(fmt.Sprintf("no outline %d in %d", i, len(o.contents)))
	}
	return o.contents[i]
}

// Add adds an outline you've already built to this outline's contents. It panics if c is nil, a bug in the caller.
func (o *Outline) Add(c *Outline) {
	if c == nil {
		panic("an outline adds an Outline, not nil")
	}
	o.contents = append(o.contents, c)
}

// AddName adds a new outline with this name and no value to this outline's contents. It panics if k isn't a name, a bug in the caller.
func (o *Outline) AddName(k string) {
	o.Add(NewOutline(k))
}

// AddValue adds a new unnamed outline with this value to this outline's contents.
func (o *Outline) AddValue(d Data) {
	o.Add(&Outline{value: d})
}

// Has reports whether contents includes an outline with name k. It panics if k isn't a name, since lookups validate too, so a mistake is a loud error instead of a silent "not found".
func (o *Outline) Has(k string) bool {
	CheckOutlineName(k)
	for _, c := range o.contents {
		if c.name == k {
			return true
		}
	}
	return false
}

// o.N() for "navigate": find the first outline in contents with name k, panic if not found
// o.M() for "make": like N, but creates the outline if it doesn't exist yet
// o.List("") returns all unnamed contents
// o.List("type1") or filter out a named sublist, all contents named "type1"

// N navigates to the first outline in contents with name k. It panics if there isn't one, a bug in the caller; Has asks first.
func (o *Outline) N(k string) *Outline {
	CheckOutlineName(k)
	for _, c := range o.contents {
		if c.name == k {
			return c
		}
	}
	panic(fmt.Sprintf("no outline named %q", k))
}

// M makes: like N, but creates the outline if it doesn't exist yet, so a chain like o.M("basket").M("color").SetValue(d) builds as it goes.
func (o *Outline) M(k string) *Outline {
	if !o.Has(k) {
		o.AddName(k)
	}
	return o.N(k)
}

// List is all the contents named k, or with "", all the unnamed ones.
func (o *Outline) List(k string) []*Outline {
	CheckOutlineName(k)
	list := []*Outline{}
	for _, c := range o.contents {
		if c.name == k {
			list = append(list, c)
		}
	}
	return list
}

// Remove removes all outlines in contents with name k.
func (o *Outline) Remove(k string) {
	CheckOutlineName(k)
	o.contents = slices.DeleteFunc(o.contents, func(c *Outline) bool { return c.name == k })
}

// Clear discards this outline's value and contents (but keeps the name!).
func (o *Outline) Clear() {
	o.value = Data{}
	o.contents = nil
}

// Sort sorts all nested contents, at every level, so identical information produces identical serialization. It panics if the outline holds itself, or holds the same outline in two places, a bug in the caller, since an outline is a strict tree.
func (o *Outline) Sort() {
	outlineFlat(o) // refuse a cycle or a shared subtree first, as the writers do, since sorting walks the whole tree
	walk(o, nil, func(x *Outline, depth int) {
		slices.SortStableFunc(x.contents, outlineCompare) // sort from the deepest levels up, since the walk leaves an outline only after all its contents, then sort this level
	})
}

// Text is the outline as indented lines of text, its text serialization, visible in source and documentation. It panics like Sort.
func (o *Outline) Text() string {
	return outlineToText(o)
}

// Data is the outline as bytes, its binary serialization, compact on the wire. It panics like Sort, and if the bytes would pass the span's cap of about 256 MB.
func (o *Outline) Data() Data {
	return outlineToData(o)
}

// outlineToText expresses an outline as indented lines of text for code, documentation, and diffs
func outlineToText(o *Outline) string {
	outlineFlat(o)    // guard against user-supplied cycles before sort
	o = sortedCopy(o) // write a sorted copy, so the text is canonical and the outline the caller has stays as it is
	var s strings.Builder
	indents := []string{""} // the indent at each depth, two spaces per level, each made once from the one above it
	walk(o, func(x *Outline, depth int) {
		if depth == len(indents) {
			indents = append(indents, indents[depth-1]+"  ")
		}
		s.WriteString(indents[depth] + x.name + ":" + DataToQuoted(x.value) + "\n") // one line for each outline as the walk reaches it, so its contents follow it at a deeper indent
	}, nil)
	return s.String()
}

// TextToOutline parses a text outline into its object form, or returns an error because the text isn't one. It reads leniently, the way the essay above describes, for text a person or an agent wrote by hand.
func TextToOutline(s string) (*Outline, error) {
	var lines []string
	for _, line := range strings.Split(s, "\n") { // split at the line feed alone, rather than at every line ending unicode knows
		line = strings.TrimSuffix(line, "\r") // accept both \r\n and \n line endings
		if HasText(line) && !strings.HasPrefix(strings.TrimLeft(line, " \t"), "#") {
			lines = append(lines, line) // skip blank lines, and lines that are only a comment, at any indent
		}
	}
	if len(lines) == 0 {
		return nil, fmt.Errorf("ergo: no outline in %q", s)
	}

	// first pass: parse each line into {o, indent}, where indent is the count of leading spaces or tabs
	// each space or tab counts as one, so 2-space, 4-space, or tab indentation all work--but must be consistent within a parent's contents
	type parsedLine struct {
		o      *Outline
		indent int
	}
	var parsed []parsedLine
	for _, l := range lines {
		indent := len(l) - len(strings.TrimLeft(l, " \t"))
		c := Cut(l[indent:], ":")
		if !c.Found {
			return nil, fmt.Errorf("ergo: no colon in %q", l) // every line must have a colon separating name from quoted value
		}

		// find where the value ends: walk forward through c.After tracking quote state, stop at the first whitespace or # outside quotes
		valueEnd := len(c.After)
		inQuote := false
		for j := 0; j < len(c.After); j++ { // a byte at a time, since each character it looks for is ascii, and in utf-8 an ascii byte is never part of another character
			ch := c.After[j]
			if ch == '"' {
				inQuote = !inQuote
				continue
			}
			if !inQuote && (ch == ' ' || ch == '\t' || ch == '#') {
				valueEnd = j
				break
			}
		}
		valuePart := c.After[:valueEnd]
		tail := strings.TrimLeft(c.After[valueEnd:], " \t")
		if tail != "" && tail[0] != '#' {
			return nil, fmt.Errorf("ergo: more after the value in %q", l) // trailing content after the value must be whitespace, or a # comment to end of line, whatever characters it holds
		}

		if !HasOutlineName(c.Before) {
			return nil, fmt.Errorf("ergo: not an outline name: %q", c.Before)
		}
		value, err := QuotedToData(valuePart) // a blank quoted form is no bytes, which is no value
		if err != nil {
			return nil, err
		}
		parsed = append(parsed, parsedLine{&Outline{name: c.Before, value: value}, indent})
	}

	// second pass: build the tree with a stack that tracks the current ancestry and enforces indent consistency
	// each stack entry is {o, indent, childIndent}: the outline, its own indent, and the indent its children will use (set when the first child appears, then required to match for further children)
	if parsed[0].indent != 0 {
		return nil, fmt.Errorf("ergo: the first line of an outline is indented: %q", s) // the root must be at zero indent--no leading whitespace
	}
	type entry struct {
		o           *Outline
		indent      int
		childIndent int // -1 until the first child appears
	}
	root := parsed[0].o
	stack := []*entry{{root, 0, -1}}
	for _, p := range parsed[1:] {
		for len(stack) > 0 && stack[len(stack)-1].indent >= p.indent {
			stack = stack[:len(stack)-1] // pop entries that can't be this line's parent (at our level or deeper)
		}
		if len(stack) == 0 {
			return nil, fmt.Errorf("ergo: more than one root in %q", s) // outdented past the root, which means there's a second root or worse
		}
		parent := stack[len(stack)-1]
		if parent.childIndent == -1 {
			parent.childIndent = p.indent // first child establishes the indent for this parent's contents
		} else if parent.childIndent != p.indent {
			return nil, fmt.Errorf("ergo: uneven indent in %q", s) // siblings must share the same indent; a mismatched indent is ambiguous
		}
		parent.o.Add(p.o)
		stack = append(stack, &entry{p.o, p.indent, -1})
	}
	return root, nil
}

// outlineToData serializes an outline to binary data for disk or wire
func outlineToData(o *Outline) Data {
	outlineFlat(o)    // guard against user-supplied cycles before sort
	o = sortedCopy(o) // sort a copy depth-first before we walk the tree, so the output is deterministic, and the outline the caller has stays as it is

	// first pass: measure the exact size we'll need, so we can allocate one buffer and write into it
	contentsSizes := map[*Outline]int{} // each outline's contents size, measured on the way out of it, so the second pass looks it up in O(1) instead of recomputing
	var measured []int                  // at each depth, the bytes measured so far of the contents of the outline the walk is in there
	total := 0
	walk(o, func(x *Outline, depth int) {
		if depth == len(measured) {
			measured = append(measured, 0)
		}
		measured[depth] = 0
	}, func(x *Outline, depth int) {
		nameBytes := len(x.name) // names are [a-z0-9], so string length equals utf-8 byte count
		valueBytes := x.value.Size()
		contentsBytes := measured[depth] // every outline in x's contents has been measured, since the walk leaves x after them
		contentsSizes[x] = contentsBytes
		size := len(spanEncode(nameBytes)) + nameBytes + // three (span, bytes) pairs: name, value, contents
			len(spanEncode(valueBytes)) + valueBytes +
			len(spanEncode(contentsBytes)) + contentsBytes
		if depth > 0 {
			measured[depth-1] += size // count x in the contents of the outline above it
		} else {
			total = size
		}
	})
	buffer := make([]byte, 0, total) // second pass: write the bytes in the same order we just measured, into one buffer made the exact size

	walk(o, func(x *Outline, depth int) {
		buffer = append(buffer, spanEncode(len(x.name))...) // name span and bytes
		buffer = append(buffer, x.name...)
		buffer = append(buffer, spanEncode(x.value.Size())...) // value span and bytes
		buffer = append(buffer, x.value.Array()...)
		buffer = append(buffer, spanEncode(contentsSizes[x])...) // contents span, then each contained outline, which the walk reaches next
	}, nil)
	return DataFromArray(buffer)
}

// DataToOutline parses binary data into an outline object, or returns an error because the bytes aren't exactly what Data writes. It's strict, the way the essay above describes, for bytes from the wire.
func DataToOutline(d Data) (*Outline, error) {
	a := d.Array()
	w := 0 // walker, moves forward through a as we read bytes out of it

	// span reads a size prefix, advancing the walker
	span := func() (int, error) {
		s, err := spanDecode(a, w)
		w += s.BytesRead
		return s.Value, err
	}
	// read reads n bytes, advancing the walker
	read := func(n int) ([]byte, error) {
		if w+n > len(a) {
			return nil, fmt.Errorf("ergo: an outline needs %d bytes at %d of %d", n, w, len(a))
		}
		b := slices.Clone(a[w : w+n]) // copied, so the outline doesn't change if the bytes it came from do
		w += n
		return b, nil
	}
	type open struct {
		o        *Outline
		end      int      // where its contents end
		previous *Outline // the outline parsed just before the next one in its contents
	}
	// start starts parsing one outline: name span + bytes, value span + bytes, then the contents span, which says where its contents end
	start := func() (*open, error) {
		o := &Outline{}

		nameSize, err := span()
		if err != nil {
			return nil, err
		}
		if nameSize > 0 {
			b, err := read(nameSize)
			if err != nil {
				return nil, err
			}
			name, err := DataFromArray(b).Text() // a name's bytes are its text, which must be a-z and 0-9
			if err != nil || !HasOutlineName(name) {
				return nil, fmt.Errorf("ergo: not an outline name: %x", b)
			}
			o.name = name
		}

		valueSize, err := span()
		if err != nil {
			return nil, err
		}
		if valueSize > 0 {
			b, err := read(valueSize)
			if err != nil {
				return nil, err
			}
			o.value = DataFromArray(b) // zero size stays as the default, no value
		}

		contentsSize, err := span()
		if err != nil {
			return nil, err
		}
		return &open{o: o, end: w + contentsSize}, nil
	}

	// parse outlines inside outlines with a stack of our own, rather than calling ourselves for each level down: the outlines still being parsed, from the root down, each with where its contents end
	first, err := start()
	if err != nil {
		return nil, err
	}
	stack := []*open{first}
	for len(stack) > 0 {
		top := stack[len(stack)-1]
		if w < top.end {
			next, err := start() // parse contained outlines until we've consumed exactly contentsSize bytes
			if err != nil {
				return nil, err
			}
			stack = append(stack, next)
			continue
		}
		if w != top.end {
			return nil, fmt.Errorf("ergo: an outline's contents ran past their size") // overran or underran the contents region, input is malformed
		}

		// top is whole, its contents all parsed, so now it can be compared and added to the outline above it
		stack = stack[:len(stack)-1]
		if len(stack) == 0 {
			break // top is the root
		}
		parent := stack[len(stack)-1]
		if parent.previous != nil && outlineCompare(parent.previous, top.o) > 0 {
			return nil, fmt.Errorf("ergo: the contents of an outline in binary are out of order") // the binary form is the canonical one, sorted at every level, so the same information is always the same bytes, and an outline's hash is its identity
		}
		parent.o.Add(top.o)
		parent.previous = top.o
	}
	if w != len(a) {
		return nil, fmt.Errorf("ergo: %d bytes after the outline", len(a)-w) // trailing bytes after the root outline mean malformed input
	}
	return first.o, nil
}

// Every walk through an outline keeps a stack of its own, rather than calling itself for each level down. The writers, Sort, and the check for cycles walk with walk, which calls enter with each outline as it reaches it, and leave once all of that outline's contents are done, so what a recursive function would do before calling itself goes in enter, and what it would do after, in leave. Compare keeps a stack of the pairs it has yet to compare, and the binary reader a stack of the outlines it's still reading, each with where its contents end. The text reader has kept a stack of the outlines above the line it's reading since it was first written.
//
// The other walks were written recursively at first, which reads more simply, and moved to loops when depth came to matter. A walk that calls itself spends a frame of the language's own stack on every level, and that stack is small: a few thousand levels in JavaScript and in Rust, about a thousand in Python. An outline's binary form arrives from the wire, so a few kilobytes of nesting from anyone could run it out. Past that point, JavaScript and Python throw, Go, whose stacks grow to a gigabyte before it gives up, stops with a fatal error, and Rust aborts the whole process, which nothing can catch. A walk with a stack of its own needs only memory, so how deep an outline goes is limited by memory alone, how large its bytes can be by the span's cap, and no nesting from anywhere can crash it. In JavaScript, timed against the recursion they replaced, the loops are no slower.

// walk walks an outline depth first: it calls enter with each outline as the walk reaches it, and how many levels down it is, and leave with each outline and its depth after all its contents; either can be nil
func walk(root *Outline, enter, leave func(o *Outline, depth int)) {
	path := []*Outline{root} // the outlines from the root down to where the walk is
	next := []int{0}         // and at each level, the index of the next outline to visit in its contents
	if enter != nil {
		enter(root, 0)
	}
	for len(path) > 0 {
		level := len(path) - 1
		o := path[level]
		if next[level] < len(o.contents) {
			c := o.contents[next[level]] // go down into the next outline in contents
			next[level]++
			path = append(path, c)
			next = append(next, 0)
			if enter != nil {
				enter(c, level+1)
			}
		} else {
			path = path[:level] // or back up, having visited them all
			next = next[:level]
			if leave != nil {
				leave(o, level)
			}
		}
	}
}

// sortedCopy is a copy of an outline with its contents sorted at every level, deepest first, for the writers; the values are shared rather than copied, since a Data's bytes are the same either way
func sortedCopy(o *Outline) *Outline {
	var copies []*Outline     // at each depth, the copy the walk is making there
	var contents [][]*Outline // and the copies of its contents, gathered to sort before they go in
	walk(o, func(x *Outline, depth int) {
		if depth == len(copies) {
			copies = append(copies, nil)
			contents = append(contents, nil)
		}
		copies[depth] = &Outline{name: x.name, value: x.value}
		contents[depth] = nil
	}, func(x *Outline, depth int) {
		slices.SortStableFunc(contents[depth], outlineCompare) // sort from the deepest levels up, since the walk leaves an outline only after all its contents, then sort this level
		copies[depth].contents = contents[depth]
		if depth > 0 {
			contents[depth-1] = append(contents[depth-1], copies[depth]) // and gather this copy, now whole, for the copy above it
		}
	})
	return copies[0]
}

// outlineFlat makes sure o doesn't have a cycle or duplicate
func outlineFlat(o *Outline) {
	seen := map[*Outline]bool{}
	walk(o, func(x *Outline, depth int) {
		if seen[x] {
			panic("an outline appears twice in its tree, in a cycle or as a shared subtree") // cycle or DAG: x already appears somewhere else in the tree, which the walk finds the first time it comes back to x, before it can go around again
		}
		seen[x] = true
	}, nil)
}

// outlineCompare compares two outlines for sort order: name first, then value, then nested contents pairwise down the whole tree
func outlineCompare(o1, o2 *Outline) int {
	if c := outlineCompareHere(o1, o2); c != 0 {
		return c // names or values differ, which settles nearly every comparison a sort makes, with nothing made to do it
	}

	// value tied, so we examine contents pairwise, with a stack of our own of the pairs still to compare, rather than calling ourselves for each level down
	type pair struct {
		a, b   *Outline
		counts bool // a pair marked counts compares how many contents its two outlines hold, once all the pairs of their contents have tied
	}
	var pending []pair // the pair on top is next
	push := func(a, b *Outline) {
		pending = append(pending, pair{a, b, true})
		for i := min(len(a.contents), len(b.contents)) - 1; i >= 0; i-- {
			pending = append(pending, pair{a.contents[i], b.contents[i], false}) // compare contents pairwise at each index, the first pair on top, so it and everything inside it is compared before the second
		}
	}
	push(o1, o2)
	for len(pending) > 0 {
		p := pending[len(pending)-1]
		pending = pending[:len(pending)-1]
		if p.counts {
			if c := cmp.Compare(len(p.a.contents), len(p.b.contents)); c != 0 {
				return c // all shared contents match, fewer contents sorts first
			}
		} else {
			if c := outlineCompareHere(p.a, p.b); c != 0 {
				return c // every caller sorts or checks deeper levels first, so contents at each level are already in order
			}
			push(p.a, p.b)
		}
	}
	return 0 // a complete and deep tie: identical name, value, and contents
}

// outlineCompareHere compares two outlines by their names and values alone, for compare as it goes down the tree
func outlineCompareHere(o1, o2 *Outline) int {
	if c := cmp.Compare(o1.name, o2.name); c != 0 {
		return c // name first, where the lighter name wins
	}
	// same name, so we move on to compare value; no value is an empty Data, which sorts before any value, since a shorter value sorts first
	// bytes.Compare compares the way an outline sorts: byte by byte up to the shorter length, where the first differing byte decides the order, and then the shorter value first
	return bytes.Compare(o1.value.Array(), o2.value.Array()) // and if the values tie too, compare goes on to examine contents pairwise
}

// Span: a variable-length integer encoding for size prefixes in the binary format. Each byte contributes 7 bits of value; the high bit is 1 to continue, 0 to stop. Least-significant byte first (little-endian), matching LEB128 (WebAssembly, DWARF debug info) and Protocol Buffers' varint. Capped at 4 bytes (28 bits), giving a maximum value of 268,435,455 — about 256 MB. Outlines are designed to be small packets, not bulk storage, so this ceiling is generous. An Outline that approaches this size is almost certainly a misuse.
//
// The size of an outline's contents is a span too, the root's included, so the cap bounds an outline's whole binary form at about 256 MB: the writer refuses an outline whose bytes would pass it, and no span can tell the reader to expect more. That's still more than a program should take from outside. An outline takes as few as 3 bytes in its binary form and about 70 in memory, so reading bytes can take more than 20 times their size, and when Go runs out of memory, it stops with a fatal error, which nothing can recover. So a program that reads outlines from the wire caps how many bytes it accepts before it hands them to DataToOutline, the way a server caps the size of a request body.

// Span is what SpanDecode reads: the size the span says, and how many bytes the span took, so a reader can move past it.
type Span struct {
	Value     int
	BytesRead int
}

// SpanEncode is a size as a span, a Data of 1 to 4 bytes, the way an outline's binary form writes each size. It panics unless 0 <= n <= 268,435,455, a bug in the caller.
func SpanEncode(n int) Data {
	return DataFromArray(spanEncode(n))
}

// SpanDecode reads the span at an offset into a Data, answering its value and how many bytes it took, so a reader can move past it, or returns an error because the bytes there aren't a span written the one way a span is. It panics if offset is negative, a bug in the caller.
func SpanDecode(d Data, offset int) (Span, error) {
	if offset < 0 {
		panic(fmt.Sprintf("not a count: %d", offset))
	}
	return spanDecode(d.Array(), offset)
}

// spanEncode is a size as a span's bytes, at most 4
func spanEncode(n int) []byte {
	if n < 0 || n >= 0x10000000 {
		panic(fmt.Sprintf("not a size a span can hold: %d", n)) // max 268,435,455 (~256 MB)
	}
	var b []byte
	for {
		y := byte(n & 0x7f) // take the lowest 7 bits
		n >>= 7             // shift them off, ready to take the next 7
		if n > 0 {
			y |= 0x80 // set the high bit to signal "more bytes follow" to the decoder
		}
		b = append(b, y)
		if n == 0 {
			return b
		}
	}
}

// spanDecode answers the span's value and how many bytes it took, so the caller can advance its cursor
func spanDecode(a []byte, offset int) (Span, error) {
	n := 0
	shift := 0
	i := offset
	for count := 0; count < 4; count++ { // at most 4 bytes; more than that means malformed input
		if i >= len(a) {
			return Span{}, fmt.Errorf("ergo: a span runs past the end at %d", i)
		}
		y := a[i]
		i++
		n |= int(y&0x7f) << shift // accumulate the 7 value bits at their proper place
		if y&0x80 == 0 {          // high bit clear means this is the last byte
			if count > 0 && y == 0 {
				return Span{}, fmt.Errorf("ergo: a span written longer than it needs to be at %d", offset) // reject non-canonical encoding: a multi-byte span whose top byte contributes no bits could have fit in fewer bytes
			}
			return Span{n, i - offset}, nil
		}
		shift += 7
	}
	return Span{}, fmt.Errorf("ergo: a span longer than 4 bytes at %d", offset) // ran past the 4 byte limit without finding a terminating byte
}

// "Quoted" Encoding: A text format for binary data that mixes quoted ASCII text with base16
//
// «"Hello"0a» is easier to read than base16 '48656c6c6f0a'
// «"A"00» shows the bits of a null terminator in a language-agnostic way, unlike 'A\0'
// «» blank text is valid quoted encoding to express nothing, size zero bytes
//
// unlike the base encoding of Data, quoted is *not* round-trip!
// a heuristic below balances brevity and readability when choosing which ASCII bytes to expose in quotes

// quotable is true if the given character c is understandable to read in quotes: space through tilde, but not the double quotation mark itself, which is our delimiter!
func quotable(c byte) bool {
	return c >= 0x20 && c <= 0x7e && c != 0x22
}

// DataToQuoted encodes the given Data as text in quoted encoding, where no bytes at all are blank text.
func DataToQuoted(d Data) string {
	a := d.Array()

	textCount := 0
	for _, c := range a {
		if quotable(c) {
			textCount++
		}
	}
	if textCount*2 < len(a) {
		return d.Base16() // under half quotable? it's random or binary, just use base16; ties go to quoted
	}

	var s strings.Builder
	for i := 0; i < len(a); { // alternate between:
		start := i
		if quotable(a[i]) { // runs of quotable text, and
			for i < len(a) && quotable(a[i]) {
				i++
			}
			s.WriteString(`"` + string(a[start:i]) + `"`)
		} else { // runs of unquotable bytes in base16
			for i < len(a) && !quotable(a[i]) {
				i++
			}
			s.WriteString(hex.EncodeToString(a[start:i]))
		}
	}
	return s.String()
}

// QuotedToData converts quoted text back to Data, where blank text is valid quoted encoding to describe zero bytes of nothing, or returns an error because the text isn't quoted encoding. It reads leniently, so many texts read to the same bytes.
func QuotedToData(s string) (Data, error) {
	b, ok := quotedBytes(s)
	if !ok {
		return Data{}, fmt.Errorf("ergo: not quoted encoding: %q", s)
	}
	return DataFromArray(b), nil
}

// quotedBytes is the bytes quoted text holds, or false when it isn't quoted encoding
func quotedBytes(s string) ([]byte, bool) {
	// walk through the string, alternating between quoted sections and base16 pairs, a byte at a time, since anything that isn't ascii is refused either way
	b := []byte{}
	for i := 0; i < len(s); {
		if s[i] == '"' { // opening quote starts a text section
			i++
			end := strings.IndexByte(s[i:], '"')
			if end == -1 {
				return nil, false // no closing quote
			}
			for _, c := range []byte(s[i : i+end]) { // strict: every character inside must be something DataToQuoted would have quoted
				if !quotable(c) {
					return nil, false // rejects control chars, high bytes, unicode, and the quote char itself
				}
				b = append(b, c) // quotable guarantees ASCII, one byte per char
			}
			i += end + 1
		} else { // outside quotes, consume base16 digit pairs
			if i+1 >= len(s) {
				return nil, false // a lone base16 digit with no pair is invalid
			}
			c0, ok0 := base16Digit(s[i])
			c1, ok1 := base16Digit(s[i+1])
			if !ok0 || !ok1 {
				return nil, false // lowercase base16 only
			}
			b = append(b, c0<<4|c1)
			i += 2
		}
	}
	return b, true
}

// base16Digit is what a lowercase base16 digit is worth, or false for any other character
func base16Digit(c byte) (byte, bool) {
	switch {
	case '0' <= c && c <= '9':
		return c - '0', true
	case 'a' <= c && c <= 'f':
		return c - 'a' + 10, true
	}
	return 0, false
}

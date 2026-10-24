package ergo_test

import (
	"fmt"
	"math/rand"
	"os"
	"slices"
	"strconv"
	"strings"
	"testing"
	"time"

	ergo "github.com/zootella/ergo/go"
)

// fuzzSeconds is the seconds ERGO_FUZZ names, or a skip when it doesn't, so go test stays deterministic and fast. Run the fuzz tests with ERGO_FUZZ=4 go test -run Fuzz -v, where -v shows the counts they reached.
func fuzzSeconds(t *testing.T) float64 {
	seconds, _ := strconv.ParseFloat(os.Getenv("ERGO_FUZZ"), 64)
	if seconds <= 0 {
		t.Skip("set ERGO_FUZZ to a number of seconds")
	}
	return seconds
}

// TestFuzzTick runs random ticks through both text forms and SayTick, as fast as this edition can, for the number of seconds ERGO_FUZZ names.
func TestFuzzTick(t *testing.T) {
	seconds := fuzzSeconds(t)
	// centers and radii from the tick's original fuzz test: a broad sweep of decades, leap day, each unit boundary at a narrow and then a wide aperture, and the two edges of the range
	centers := []struct {
		text   string
		radius int64
	}{
		{"2005", 30 * ergo.Year},
		{"2000feb29", 3 * ergo.Day},
		{"2000", ergo.Hour},
		{"2000mar", ergo.Hour},
		{"2000jan1.12", 6 * ergo.Hour},
		{"2000jan1.1230", 30 * ergo.Minute},
		{"2000jan1.1230.30000", 30 * ergo.Second},
		{"2000", 15 * ergo.Day},
		{"2000mar", 15 * ergo.Day},
		{"2000jan1.12", 15 * ergo.Day},
		{"2000jan1.1230", 12 * ergo.Hour},
		{"2000jan1.1230.30000", 30 * ergo.Minute},
		{"1970", ergo.Day},
		{"9999dec31", ergo.Day},
	}
	last, err := ergo.TextToTick("9999dec31.2359.59999") // the ceiling, so a tick drawn near an edge center stays a tick
	if err != nil {
		t.Fatal(err)
	}
	budget := time.Duration(seconds / float64(len(centers)) * float64(time.Second)) // time to spend on each center
	everyField := "Dddd YYYY-MM-DD hh12:mm:ss.ttt AA"                               // a template that says every field of a tick, the weekday among them
	cycles := 0
	for _, c := range centers {
		center, err := ergo.TextToTick(c.text)
		if err != nil {
			t.Fatal(err)
		}
		stop := time.Now().Add(budget)
		for time.Now().Before(stop) {
			tick := min(max(center+rand.Int63n(2*c.radius+1)-c.radius, 0), last) // a random tick within the radius, clamped to the range of ticks
			if back, err := ergo.TextToTick(ergo.TickToText(tick)); err != nil || back != tick {
				t.Fatalf("%d did not round-trip through the short form: %d, %v", tick, back, err)
			}
			if back, err := ergo.ISOToTick(ergo.TickToISO(tick)); err != nil || back != tick {
				t.Fatalf("%d did not round-trip through the iso form: %d, %v", tick, back, err)
			}
			ergo.SayTick(tick, everyField) // and say it with every field, as the fuzz across the editions does, which is where what it says gets compared
			cycles++
		}
	}
	t.Logf("go fuzzed %d ticks through both text forms and SayTick in %g seconds, %.0f a second", cycles, seconds, float64(cycles)/seconds)
}

// TestFuzzTag is a hot loop making tags: the smoke test that this edition makes them as fast as it can.
func TestFuzzTag(t *testing.T) {
	seconds := fuzzSeconds(t)
	stop := time.Now().Add(time.Duration(seconds * float64(time.Second)))
	count := 0
	for time.Now().Before(stop) {
		if tag := ergo.Tag(); !ergo.HasTag(tag) {
			t.Fatalf("%q is not a tag", tag)
		}
		count++
	}
	t.Logf("go made %d tags in %g seconds, %.0f a second, each checked with HasTag", count, seconds, float64(count)/seconds)
}

// TestFuzzHash runs each hash as fast as this edition can, on one mebibyte of random bytes again and again, and once a piece at a time to check it agrees.
func TestFuzzHash(t *testing.T) {
	seconds := fuzzSeconds(t)
	data := ergo.RandomData(ergo.MB) // one mebibyte of random bytes, hashed again and again
	// each hash by name, all at once, and its hasher
	hashes := []struct {
		name   string
		all    func(ergo.Data) ergo.Data
		hasher func() *ergo.Hasher
	}{
		{"sha1", ergo.SHA1, ergo.NewSHA1},
		{"sha256", ergo.SHA256, ergo.NewSHA256},
		{"blake3", ergo.Blake3, ergo.NewBlake3},
	}
	share := seconds / float64(len(hashes)) // an even share of the seconds for each hash
	var said []string
	for _, h := range hashes {
		stop := time.Now().Add(time.Duration(share * float64(time.Second)))
		count := 0
		var last ergo.Data
		for {
			last = h.all(data)
			count++
			if !time.Now().Before(stop) {
				break // at least once, however few the seconds
			}
		}
		p := h.hasher() // and once a piece at a time, at a split that lines up with nothing, which must agree
		p.Update(data.ClipView(0, 1000))
		p.Update(data.ClipView(1000, data.Size()))
		if p.Digest().Base16() != last.Base16() {
			t.Fatalf("%s disagreed with itself a piece at a time", h.name)
		}
		said = append(said, fmt.Sprintf("%s %.0f MiB a second", h.name, float64(count)/share))
	}
	t.Logf("go hashed one mebibyte again and again: %s", strings.Join(said, ", "))
}

var (
	outlineNames      = []string{"", "", "", "a", "a", "b", "ab", "a0", "z9"}            // the names random outlines take, a few of them, blank most of all, so siblings share names and ties come up
	outlineValues     = []string{"", "", "", "00", "01", "61", "6161", "22", "20", "23"} // and the values, as base16: none most of all, and ones that tie, that begin one another, and that are the characters quoted encoding treats with care, a quote, a space, and a number sign
	outlineCharacters = []string{"a", "b", ":", "#", " ", `"`, "\t", "\n"}               // and the characters of a value that reads as text, with the ones a text outline treats with care
)

// pick is one of the things in a list, at random
func pick[T any](a []T) T {
	return a[rand.Intn(len(a))]
}

// randomValue is a random value: often one from the list above, sometimes text, sometimes random bytes
func randomValue() ergo.Data {
	r := rand.Float64()
	if r < 0.6 {
		d, _ := ergo.DataFromBase16(pick(outlineValues))
		return d
	}
	if r < 0.8 {
		var s strings.Builder
		for range rand.Intn(12) {
			s.WriteString(pick(outlineCharacters))
		}
		return ergo.DataFromText(s.String())
	}
	return ergo.RandomData(rand.Intn(40))
}

// randomOutline is a random outline, its contents nested down at most depth levels
func randomOutline(depth int) *ergo.Outline {
	o := ergo.NewOutline(pick(outlineNames))
	o.SetValue(randomValue())
	if depth > 0 {
		for range rand.Intn(5) {
			o.Add(randomOutline(depth - 1 - rand.Intn(2)))
		}
	}
	return o
}

// outlineShape is an outline as plain text, in the order it holds its contents, to compare two outlines, or one before and after
func outlineShape(o *ergo.Outline) string {
	var s strings.Builder
	s.WriteString(fmt.Sprintf("[%q %q [", o.Name(), o.Value().Base16()))
	for i := range o.Length() {
		s.WriteString(outlineShape(o.Get(i)))
	}
	s.WriteString("]]")
	return s.String()
}

// shuffledOutline is a copy of an outline with its contents in a random order at every level
func shuffledOutline(o *ergo.Outline) *ergo.Outline {
	c := ergo.NewOutline(o.Name())
	c.SetValue(o.Value())
	for _, i := range rand.Perm(o.Length()) {
		c.Add(shuffledOutline(o.Get(i)))
	}
	return c
}

// sloppyQuoted is a value in quoted encoding a person might write instead: as the writer writes it, as base16 alone, or with quotable bytes quoted one at a time, or not
func sloppyQuoted(d ergo.Data) string {
	r := rand.Float64()
	if r < 0.4 {
		return ergo.DataToQuoted(d)
	}
	if r < 0.7 {
		return d.Base16()
	}
	var s strings.Builder
	for _, b := range d.Array() {
		if b >= 0x20 && b <= 0x7e && b != 0x22 && rand.Float64() < 0.5 {
			s.WriteString(`"` + string(rune(b)) + `"`)
		} else {
			s.WriteString(fmt.Sprintf("%02x", b))
		}
	}
	return s.String()
}

// sloppyText is an outline as text the way a person might write it by hand: contents in any order, any indent, any quoting, trailing whitespace and comments, lines of comments, blank lines, and either line ending
func sloppyText(o *ergo.Outline) string {
	var s strings.Builder
	var walk func(o *ergo.Outline, indent string)
	walk = func(o *ergo.Outline, indent string) {
		s.WriteString(indent + o.Name() + ":" + sloppyQuoted(o.Value()))
		s.WriteString(pick([]string{"", "", " ", "\t", " #note", "#", ` # "quoted" #more`, "\t#" + string(rune(0x2028)) + "\r"}))
		s.WriteString(pick([]string{"\n", "\r\n"}))
		if rand.Float64() < 0.1 {
			s.WriteString(pick([]string{"", "  ", "\t", string(rune(0x3000))}) + "\n") // a blank line, of nothing or of whitespace
		}
		if rand.Float64() < 0.1 {
			s.WriteString(pick([]string{"", " ", "\t", indent}) + pick([]string{"#", "# a note", `#x:"commented out"`}) + "\n") // a line that's only a comment, at any indent
		}
		unit := pick([]string{"  ", "    ", "\t", " ", " \t"}) // each parent picks the indent its own contents share
		for _, i := range rand.Perm(o.Length()) {
			walk(o.Get(i), indent+unit)
		}
	}
	walk(o, "")
	return s.String()
}

// mutatedData is a copy of some bytes with one thing changed: a byte replaced, inserted, or deleted, or the end cut off
func mutatedData(d ergo.Data) ergo.Data {
	a := slices.Clone(d.Array())
	i := rand.Intn(len(a) + 1)
	b := pick([]byte{byte(rand.Intn(256)), pick([]byte{0x00, 0x01, 0x30, 0x61, 0x62, 0x80})})
	r := rand.Float64()
	switch {
	case r < 0.4 && len(a) > 0:
		a[min(i, len(a)-1)] = b
	case r < 0.6:
		a = slices.Insert(a, i, b)
	case r < 0.8 && len(a) > 0:
		a = slices.Delete(a, min(i, len(a)-1), min(i, len(a)-1)+1)
	default:
		a = a[:i]
	}
	return ergo.DataFromArray(a)
}

// mutatedText is a copy of some text with one character replaced, inserted, or deleted, from the characters a text outline cares about
func mutatedText(s string) string {
	r := []rune(s)
	i := rand.Intn(len(r) + 1)
	c := pick([]rune{'a', ':', '"', ' ', '\t', '#', '\n', '0', 'A', '\r', 0xa0})
	switch x := rand.Float64(); {
	case x < 0.4 && i < len(r):
		r[i] = c
	case x < 0.7:
		r = slices.Insert(r, i, c)
	case i < len(r):
		r = slices.Delete(r, i, i+1)
	}
	return string(r)
}

// TestFuzzOutline has a random outline round-trip through both forms, and write the same bytes from any order and any hand-written text.
func TestFuzzOutline(t *testing.T) {
	seconds := fuzzSeconds(t)
	stop := time.Now().Add(time.Duration(seconds * float64(time.Second)))
	cycles := 0
	for time.Now().Before(stop) {
		o := randomOutline(5)
		before := outlineShape(o)
		d := o.Data()  // the canonical bytes
		t1 := o.Text() // and the canonical text
		if outlineShape(o) != before {
			t.Fatalf("a writer changed the outline: %s", t1)
		}
		back, err := ergo.DataToOutline(d)
		if err != nil || back.Data().Base16() != d.Base16() || back.Text() != t1 {
			t.Fatalf("the bytes did not read back to the same outline: %s, %v", t1, err)
		}
		fromText, err := ergo.TextToOutline(t1)
		if err != nil || fromText.Data().Base16() != d.Base16() {
			t.Fatalf("the text read to other bytes: %s, %v", t1, err)
		}
		shuffled := shuffledOutline(o)
		if shuffled.Data().Base16() != d.Base16() {
			t.Fatalf("the same outline in another order wrote other bytes: %s", t1)
		}
		shuffled.Sort()
		if outlineShape(shuffled) != outlineShape(back) {
			t.Fatalf("sort in place disagreed with the sort the writers do: %s", t1)
		}
		sloppy := sloppyText(o)
		fromSloppy, err := ergo.TextToOutline(sloppy)
		if err != nil || fromSloppy.Data().Base16() != d.Base16() {
			t.Fatalf("hand-written text read to other bytes, from %q: %v", sloppy, err)
		}
		cycles++
	}
	t.Logf("go fuzzed %d random outlines through both forms, in any order, and as hand-written text, in %g seconds, %.0f a second", cycles, seconds, float64(cycles)/seconds)
}

// TestFuzzOutlineReaders has the readers take only what reads back the same: changed bytes, text, and spans refuse, or read to a form that round-trips.
func TestFuzzOutlineReaders(t *testing.T) {
	seconds := fuzzSeconds(t)
	stop := time.Now().Add(time.Duration(seconds * float64(time.Second)))
	cycles := 0
	refused := 0
	for time.Now().Before(stop) {
		o := randomOutline(3)

		// the binary reader is strict: whatever bytes it takes are exactly the bytes its writer writes
		d := mutatedData(o.Data())
		if fromData, err := ergo.DataToOutline(d); err != nil {
			refused++ // refused, which is fine; the property is about what it takes
		} else if fromData.Data().Base16() != d.Base16() {
			t.Fatalf("the binary reader took %s, and wrote back %s", d.Base16(), fromData.Data().Base16())
		}

		// the text reader is lenient, but whatever text it takes, it writes back as text that reads to the same outline
		s := mutatedText(o.Text())
		if fromText, err := ergo.TextToOutline(s); err != nil {
			refused++
		} else {
			t2 := fromText.Text()
			again, err := ergo.TextToOutline(t2)
			if err != nil || again.Text() != t2 || again.Data().Base16() != fromText.Data().Base16() {
				t.Fatalf("the text reader took %q, and its text did not read back the same: %v", s, err)
			}
		}

		// and a span is strict, too: a size writes as one set of bytes, and reads back from them, and no other bytes read to it
		n := rand.Intn(1 << rand.Intn(29)) // as often small as large, up to the most four bytes hold
		e := ergo.SpanEncode(n)
		if r, err := ergo.SpanDecode(e, 0); err != nil || r.Value != n || r.BytesRead != e.Size() {
			t.Fatalf("the span of %d did not round-trip", n)
		}
		x := ergo.RandomData(1 + rand.Intn(5))
		if fromSpan, err := ergo.SpanDecode(x, 0); err != nil {
			refused++
		} else if ergo.SpanEncode(fromSpan.Value).Base16() != x.ClipView(0, fromSpan.BytesRead).Base16() {
			t.Fatalf("the span reader took %s as %d, which writes as other bytes", x.Base16(), fromSpan.Value)
		}
		cycles++
	}
	t.Logf("go changed %d random outlines' bytes and text, and read random spans, in %g seconds, %.0f a second, and the readers refused %d of the %d", cycles, seconds, float64(cycles)/seconds, refused, cycles*3)
}

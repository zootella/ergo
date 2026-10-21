package ergo_test

import (
	"fmt"
	"math/rand"
	"os"
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

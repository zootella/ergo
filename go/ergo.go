// Package ergo is short for ergonomic: simple helper functions, the same in JavaScript, Python, Rust, and Go.
package ergo

import (
	"crypto/rand"
	"fmt"
	"math"
	"slices"
	"strconv"
	"strings"
	"time"
)

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

// A tick is an integer count of milliseconds since the start of 1970, the way time.Now().UnixMilli() counts, and ergo keeps every tick between zero and the last millisecond of 9999. Two text forms carry a tick exactly. The iso form is like "2012-08-06T05:17:57.300Z": always 24 characters, always milliseconds, always Z. The short form is like "2012aug6.0517.57300": the year, month, and day, then a dot and the clock, then a dot and the milliseconds into the minute, written only as deep as the moment needs, so the start of 1990 is just "1990" and the longest text is 20 characters.
//
// Each form reads back to exactly the tick that wrote it, and reads nothing else. A reader accepts the text its writer produces and refuses everything else, from a missing millisecond digit to "1990feb1" where the writer would say "1990feb". The last check in each reader is the round trip itself: parse the text, write the tick, and require the same text. That's what keeps every text one to one with its tick.

const lastTick = 253402300799999 // the last millisecond of 9999, which keeps a year at four digits in every edition

var months = []string{"jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec"}

// parts of a tick in utc: the year, month 1 through 12, day 1 through 31, hour, minute, and milliseconds into the minute
type parts struct{ y, m, d, h, i, ms int }

// splitTick splits a tick into its utc parts, or panics because the caller handed it something that isn't a tick, which is a bug in the caller: a tick is zero through the last millisecond of 9999
func splitTick(t int64) parts {
	if t < 0 || t > lastTick {
		panic(fmt.Sprintf("not a tick: %d", t))
	}
	u := time.UnixMilli(t).UTC()
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

// TickToISO writes a tick in the iso form, like "2012-08-06T05:17:57.300Z". It panics if t isn't a tick, which is a bug in the caller.
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

// TickToText writes a tick in the short form, like "2012aug6.0517.57300", as deep as the moment needs and no deeper. It panics if t isn't a tick, which is a bug in the caller.
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
		text += months[p.m-1]
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
		p.m = slices.Index(months, p1[4:7]) + 1 // 0 if the month name isn't one of ours, which joinTick refuses
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

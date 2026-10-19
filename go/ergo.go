// Package ergo is short for ergonomic: simple helper functions, the same in JavaScript, Python, Rust, and Go.
package ergo

import (
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
	"slices"
	"strconv"
	"strings"
	"time"
	"unicode/utf8"

	"golang.org/x/text/unicode/norm"
	"lukechampine.com/blake3"
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
//
// Every function that takes a tick panics on a number that isn't one, because a caller handing it one has a bug. A number from outside, like a system clock or a file's modified time, isn't the caller's to vouch for, so HasTick asks about it first, and asking never panics; once HasTick says yes, nothing here panics on that tick. CheckTick is the guard that enforces the same answer, and it's the gate every function here goes through.

const lastTick = 253402300799999 // the last millisecond of 9999, which keeps a year at four digits in every edition

var months = []string{"jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec"}

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

// A Data holds bytes. Every language keeps bytes its own way, in arrays and buffers and views with their own rules about copying and size, and every platform call that takes or gives bytes wants its own one of them. A Data makes those choices once: it holds the bytes, and it speaks every form they travel in, text and base16, base32, base62, and base64, through methods with plain names. Make one from any form, read it out as any other, and it's the same bytes. In Go a Data holds a slice, so making one from a slice, or clipping a view of part of one, never copies the bytes.
//
// Every function in ergo that trades in bytes trades in Data. A hash takes a Data and answers one, and never decides what text the caller wants next; the caller says, with the next method. Only Data knows the encodings.
//
// Each form is exact, both ways. Every reader ends by writing the bytes back out and requiring the text it read, so it accepts only what its writer produces: base16 in lowercase, base32 without padding, base64 with it, nothing extra and nothing missing. Text becomes UTF-8 in Unicode Normalization Form C, so the same characters typed two ways become the same bytes, and reading bytes as text refuses bytes that aren't UTF-8, or aren't in that form, rather than guess. A reader refuses with an error, the way the tick readers do.
//
// ClipView looks at part of a Data without copying it, which is fast, and right whenever the bytes beneath won't change. ClipCopy copies them out, for when they could. Array hands over the slice itself.
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

// base32, for hash values in the database, so they can be short, but also always the same length; the standard alphabet, without padding, as icarus writes it
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

// HashText is the SHA-256 hash of text, as base32, the shortcut for the common case: text to Data, hashed, and out as the form icarus keeps hash values in. It panics like DataFromText if s isn't UTF-8.
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

// The fuji hashes are two of ergo's own, from icarus, and both are SHA-256 over a summary of a file rather than over the file itself. The piece hash covers every byte: it cuts the file into 4 MiB pieces, hashes each one, and hashes the list of those hashes. The tip hash covers almost none of them: it hashes the file's first 4 KiB, 4 KiB from its middle, and its last two, at most 16 KiB however large the file, so it's done in a few reads and finds likely copies of a file across millions of others for the cost of those few reads each. A file of 16 KiB or less is its own tip. Each summary starts with a title that names the protocol and the file's size, like "Fuji.Tips.SHA256.4KiB.5.", so two files of different sizes never share a hash, even with identical tips.
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

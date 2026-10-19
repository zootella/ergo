// Package dispatch maps the protocol's function names, which are the JavaScript edition's, to this edition's functions, with json-decoded arguments and results. The vector reader and the driver both use it; being internal, it is not part of the library's api.
package dispatch

import (
	"encoding/hex"
	"fmt"
	"hash"
	"math"

	ergo "github.com/zootella/ergo/go"
)

// Call runs the named function with json-decoded arguments; a constant is a function with no arguments, and a dotted name like "Size.kb" is a constant in a namespace. A panic comes back as an error rather than unwinding into the caller, and a function's own error comes back as itself, so the harness sees both as the call being refused.
func Call(name string, args []any) (result any, err error) {
	defer func() {
		if r := recover(); r != nil {
			err = fmt.Errorf("%v", r)
		}
	}()
	switch name {
	case "Size.b":
		return ergo.B, nil
	case "Size.kb":
		return ergo.KB, nil
	case "Size.mb":
		return ergo.MB, nil
	case "Size.gb":
		return ergo.GB, nil
	case "Size.tb":
		return ergo.TB, nil
	case "Size.pb":
		return ergo.PB, nil
	case "saySize4":
		return ergo.SaySize4(count(args, 0)), nil
	case "Time.millisecond":
		return ergo.Millisecond, nil
	case "Time.second":
		return ergo.Second, nil
	case "Time.minute":
		return ergo.Minute, nil
	case "Time.hour":
		return ergo.Hour, nil
	case "Time.day":
		return ergo.Day, nil
	case "Time.week":
		return ergo.Week, nil
	case "Time.year":
		return ergo.Year, nil
	case "Time.month":
		return ergo.Month, nil
	case "hasTick":
		return ergo.HasTick(integer(args, 0)), nil
	case "checkTick":
		return ergo.CheckTick(integer(args, 0)), nil
	case "tickToText":
		return ergo.TickToText(integer(args, 0)), nil
	case "textToTick":
		return ergo.TextToTick(text(args, 0))
	case "tickToIso":
		return ergo.TickToISO(integer(args, 0)), nil
	case "isoToTick":
		return ergo.ISOToTick(text(args, 0))
	case "tagLength":
		return ergo.TagLength, nil
	case "Tag":
		return ergo.Tag(), nil
	case "hasTag":
		return ergo.HasTag(text(args, 0)), nil
	case "checkTag":
		return ergo.CheckTag(text(args, 0)), nil
	case "sha1":
		h := ergo.SHA1(bytes(args, 0))
		return base16(h[:]), nil
	case "sha256":
		h := ergo.SHA256(bytes(args, 0))
		return base16(h[:]), nil
	case "blake3":
		h := ergo.Blake3(bytes(args, 0))
		return base16(h[:]), nil
	case "Sha1":
		return base16(fed(ergo.NewSHA1(), pieces(args, 0))), nil
	case "Sha256":
		return base16(fed(ergo.NewSHA256(), pieces(args, 0))), nil
	case "Blake3":
		return base16(fed(ergo.NewBlake3(), pieces(args, 0))), nil
	default:
		return nil, fmt.Errorf("unknown function: %s", name)
	}
}

// number is argument i as a json number, or a panic naming the argument, which Call turns into an error
func number(args []any, i int) float64 {
	if i >= len(args) {
		panic(fmt.Sprintf("argument %d is missing", i))
	}
	f, ok := args[i].(float64) // json numbers decode as float64, which is exact for every integer within the ceiling
	if !ok {
		panic(fmt.Sprintf("argument %d must be a number", i))
	}
	return f
}

// count is argument i as a count of zero or more
func count(args []any, i int) uint64 {
	f := number(args, i)
	if f < 0 || f != math.Trunc(f) {
		panic(fmt.Sprintf("argument %d must be a count", i))
	}
	return uint64(f)
}

// integer is argument i as an integer, which may be negative
func integer(args []any, i int) int64 {
	f := number(args, i)
	if f != math.Trunc(f) {
		panic(fmt.Sprintf("argument %d must be an integer", i))
	}
	return int64(f)
}

// text is argument i as text, or a panic naming the argument, which Call turns into an error
func text(args []any, i int) string {
	if i >= len(args) {
		panic(fmt.Sprintf("argument %d is missing", i))
	}
	s, ok := args[i].(string)
	if !ok {
		panic(fmt.Sprintf("argument %d must be text", i))
	}
	return s
}

// fed is the hash of every piece written in order: a hasher in the protocol takes one argument, a list of pieces, so a vector pins the piece-at-a-time form without the harness keeping an object between calls
func fed(h hash.Hash, pieces [][]byte) []byte {
	for _, piece := range pieces {
		h.Write(piece) // a hash.Hash never returns an error from Write
	}
	return h.Sum(nil)
}

// bytes is argument i as bytes, from either tagged form, or a panic naming the argument, which Call turns into an error
func bytes(args []any, i int) []byte {
	if i >= len(args) {
		panic(fmt.Sprintf("argument %d is missing", i))
	}
	b, ok := tagged(args[i])
	if !ok {
		panic(fmt.Sprintf("argument %d must be bytes", i))
	}
	return b
}

// pieces is argument i as a list of pieces of bytes
func pieces(args []any, i int) [][]byte {
	if i >= len(args) {
		panic(fmt.Sprintf("argument %d is missing", i))
	}
	list, ok := args[i].([]any)
	if !ok {
		panic(fmt.Sprintf("argument %d must be a list of pieces of bytes", i))
	}
	out := make([][]byte, len(list))
	for j, v := range list {
		if out[j], ok = tagged(v); !ok {
			panic(fmt.Sprintf("piece %d of argument %d must be bytes", j, i))
		}
	}
	return out
}

// tagged decodes bytes from the protocol's tagged json: {"base16": "616263"} is those three bytes, and {"repeat": "61", "length": 1000000} is those bytes repeated to that length, so a vector can name a million bytes in a line; false if v is neither
func tagged(v any) ([]byte, bool) {
	m, ok := v.(map[string]any)
	if !ok {
		return nil, false
	}
	if s, ok := m["base16"].(string); ok {
		b, err := hex.DecodeString(s)
		return b, err == nil
	}
	s, ok := m["repeat"].(string)
	if !ok {
		return nil, false
	}
	length, ok := m["length"].(float64) // a json number, which must be a count within the integer ceiling
	if !ok || length < 0 || length != math.Trunc(length) || length > 1<<53-1 {
		return nil, false
	}
	unit, err := hex.DecodeString(s)
	if err != nil || len(unit) == 0 { // a unit of at least one byte, the same rule in every edition
		return nil, false
	}
	b := make([]byte, int(length))
	for j := range b {
		b[j] = unit[j%len(unit)]
	}
	return b, true
}

// base16 is a result that is bytes, tagged the first way, in lowercase base16
func base16(b []byte) map[string]any {
	return map[string]any{"base16": hex.EncodeToString(b)}
}

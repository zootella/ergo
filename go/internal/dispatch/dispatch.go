// Package dispatch maps the protocol's function names, which are the JavaScript edition's, to this edition's functions, with json-decoded arguments and results. The vector reader and the driver both use it; being internal, it is not part of the library's api.
package dispatch

import (
	goBytes "bytes" // named so, since this package's own bytes reads an argument as a Data
	"fmt"
	"io"
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
	case "hasInt":
		return ergo.HasInt(integer(args, 0)), nil
	case "checkInt":
		return ergo.CheckInt(integer(args, 0)), nil
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
	case "sayDuration":
		return ergo.SayDuration(integer(args, 0), text(args, 1)), nil
	case "sayTimeRemaining":
		return ergo.SayTimeRemaining(integer(args, 0)), nil
	case "sayTimeRemainingCoarse":
		return ergo.SayTimeRemainingCoarse(integer(args, 0)), nil
	case "sayTimeRace":
		return ergo.SayTimeRace(integer(args, 0)), nil
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
	case "monthNames":
		return ergo.MonthNames(), nil
	case "dayNames":
		return ergo.DayNames(), nil
	case "sayTick":
		return ergo.SayTick(integer(args, 0), text(args, 1)), nil
	case "tagLength":
		return ergo.TagLength, nil
	case "Tag":
		return ergo.Tag(), nil
	case "hasTag":
		return ergo.HasTag(text(args, 0)), nil
	case "checkTag":
		return ergo.CheckTag(text(args, 0)), nil
	case "sha1":
		return base16(ergo.SHA1(bytes(args, 0))), nil
	case "sha256":
		return base16(ergo.SHA256(bytes(args, 0))), nil
	case "blake3":
		return base16(ergo.Blake3(bytes(args, 0))), nil
	case "hashText":
		return ergo.HashText(text(args, 0)), nil
	case "Sha1":
		return base16(fed(ergo.NewSHA1(), pieces(args, 0))), nil
	case "Sha256":
		return base16(fed(ergo.NewSHA256(), pieces(args, 0))), nil
	case "Blake3":
		return base16(fed(ergo.NewBlake3(), pieces(args, 0))), nil
	case "Data":
		return dataMethod(args)
	case "Bin":
		return binMethod(args)
	case "hashProtocol.pieces.title":
		return ergo.DefaultHashProtocol.Pieces.Title, nil
	case "hashProtocol.pieces.size":
		return ergo.DefaultHashProtocol.Pieces.Size, nil
	case "hashProtocol.tips.title":
		return ergo.DefaultHashProtocol.Tips.Title, nil
	case "hashProtocol.tips.size":
		return ergo.DefaultHashProtocol.Tips.Size, nil
	case "hashMeasure":
		m, _ := args[0].(map[string]any)
		return measured(ergo.HashMeasure(integer([]any{m["file"]}, 0), integer([]any{m["unit"]}, 0))), nil
	case "hashStripeOverlap":
		background := []([2]int64){}
		for _, b := range list(args, 1) {
			background = append(background, stripe(b))
		}
		return stripes(ergo.HashStripeOverlap(stripe(args[0]), background)), nil
	case "hashFile":
		file := bytes(args, 0)
		status, err := protocol(args, 1).HashFile(goBytes.NewReader(file.Array()), int64(file.Size()))
		if err != nil {
			return nil, err
		}
		return map[string]any{"tipHash": base16(status.TipHash)}, nil
	case "hashStream":
		list := pieces(args, 0)
		var size int64
		for _, piece := range list {
			size += int64(piece.Size())
		}
		status, err := protocol(args, 1).HashStream(&boxes{pieces: list}, size, nil)
		if err != nil {
			return nil, err
		}
		return map[string]any{"tipHash": base16(status.TipHash), "pieceHash": base16(status.PieceHash)}, nil
	case "FujiHasher":
		h := protocol(args, 2).NewFujiHasher(int64(count(args, 0)))
		for _, piece := range pieces(args, 1) {
			h.Update(piece)
		}
		hashes, err := h.Digest()
		if err != nil {
			return nil, err
		}
		return map[string]any{"tipHash": base16(hashes.TipHash), "pieceHash": base16(hashes.PieceHash)}, nil
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

// fed is the hash of every piece fed in order: a hasher in the protocol takes one argument, a list of pieces, so a vector pins the piece-at-a-time form without the harness keeping an object between calls
func fed(h *ergo.Hasher, pieces []ergo.Data) ergo.Data {
	for _, piece := range pieces {
		h.Update(piece)
	}
	return h.Digest()
}

// dataMethod is a Data in the protocol: the form to make it from, then the name of a method and that method's arguments, like [{"text": "ABC"}, "base16"] or [{"base16": "414243"}, "clipView", 1, 3]; tagged bytes are already a Data, and any other form, like {"text": ...}, makes one here
func dataMethod(args []any) (any, error) {
	if len(args) < 2 {
		panic("a Data needs a form and a method")
	}
	d, ok, err := tagged(args[0])
	if !ok {
		d, err = made(args[0])
	}
	if err != nil {
		return nil, err
	}
	switch method := text(args, 1); method {
	case "size":
		return d.Size(), nil
	case "text":
		return d.Text()
	case "base16":
		return d.Base16(), nil
	case "base32":
		return d.Base32(), nil
	case "base62":
		return d.Base62(), nil
	case "base64":
		return d.Base64(), nil
	case "get":
		return int(d.Get(int(integer(args, 2)))), nil
	case "clipView":
		return base16(d.ClipView(int(integer(args, 2)), int(integer(args, 3)))), nil
	case "clipCopy":
		return base16(d.ClipCopy(int(integer(args, 2)), int(integer(args, 3)))), nil
	default:
		return nil, fmt.Errorf("unknown method of Data: %s", method)
	}
}

// made is a Data from a form that isn't tagged bytes: {"text": ...}, {"base32": ...}, {"base62": ...}, {"base64": ...}, or {"random": n}
func made(form any) (ergo.Data, error) {
	m, _ := form.(map[string]any)
	if s, ok := m["text"].(string); ok {
		return ergo.DataFromText(s), nil
	} else if s, ok := m["base32"].(string); ok {
		return ergo.DataFromBase32(s)
	} else if s, ok := m["base62"].(string); ok {
		return ergo.DataFromBase62(s)
	} else if s, ok := m["base64"].(string); ok {
		return ergo.DataFromBase64(s)
	} else if _, ok := m["random"]; ok {
		return ergo.RandomData(int(count([]any{m["random"]}, 0))), nil
	}
	return ergo.Data{}, fmt.Errorf("not a form a Data is made from: %v", form)
}

// binMethod is a Bin in the protocol: its capacity, a list of what to add, each a byte or a Data, and the name of a method to answer with
func binMethod(args []any) (any, error) {
	b := ergo.NewBin(int(integer(args, 0)))
	items, ok := args[1].([]any)
	if !ok {
		panic("argument 1 must be a list")
	}
	for _, item := range items {
		if d, ok, err := tagged(item); ok {
			if err != nil {
				return nil, err
			}
			b.Add(d)
		} else {
			n := integer([]any{item}, 0)
			if n < 0 || n > 255 {
				panic(fmt.Sprintf("not a byte: %d", n))
			}
			b.AddByte(byte(n))
		}
	}
	switch method := text(args, 2); method {
	case "size":
		return b.Size(), nil
	case "capacity":
		return b.Capacity(), nil
	case "data":
		return base16(b.Data()), nil
	default:
		return nil, fmt.Errorf("unknown method of Bin: %s", method)
	}
}

// measured is what every edition measures alike: the pieces, and the stripes the tip hash reads
func measured(m ergo.HashMeasurement) map[string]any {
	return map[string]any{"pieces": m.Pieces, "complete": m.Complete, "fragment": m.Fragment, "stripes": stripes(m.Stripes), "stripeSize": m.StripeSize}
}

// stripes is a list of stripes as json, each [start, end]
func stripes(s [][2]int64) []any {
	out := []any{}
	for _, x := range s {
		out = append(out, []any{x[0], x[1]})
	}
	return out
}

// stripe is a stripe from json, [start, end], or a panic, which Call turns into an error
func stripe(v any) [2]int64 {
	pair, ok := v.([]any)
	if !ok || len(pair) != 2 {
		panic(fmt.Sprintf("not a stripe: %v", v))
	}
	return [2]int64{integer(pair, 0), integer(pair, 1)}
}

// list is argument i as a json list
func list(args []any, i int) []any {
	if i >= len(args) {
		panic(fmt.Sprintf("argument %d is missing", i))
	}
	l, ok := args[i].([]any)
	if !ok {
		panic(fmt.Sprintf("argument %d must be a list", i))
	}
	return l
}

// protocol is argument i as a protocol, like {"pieces": {"title": ..., "size": ...}, "tips": {...}}, or the fuji one when there's none
func protocol(args []any, i int) ergo.HashProtocol {
	if i >= len(args) || args[i] == nil {
		return ergo.DefaultHashProtocol
	}
	m, _ := args[i].(map[string]any)
	unit := func(v any) ergo.HashUnit {
		u, _ := v.(map[string]any)
		title, _ := u["title"].(string)
		return ergo.HashUnit{Title: title, Size: integer([]any{u["size"]}, 0)}
	}
	return ergo.HashProtocol{Pieces: unit(m["pieces"]), Tips: unit(m["tips"])}
}

// boxes is a stream that delivers the given pieces as its boxes, in order, each read answering no more than the rest of one piece
type boxes struct {
	pieces []ergo.Data
	at     int // which piece is next
	into   int // how far into it
}

func (b *boxes) Read(p []byte) (int, error) {
	for b.at < len(b.pieces) && b.into == b.pieces[b.at].Size() {
		b.at++ // past a piece that's done, or empty
		b.into = 0
	}
	if b.at == len(b.pieces) {
		return 0, io.EOF // the end of the stream
	}
	n := copy(p, b.pieces[b.at].Array()[b.into:])
	b.into += n
	return n, nil
}

// mulberryData is {"mulberry": seed, "length": n}, n bytes from the Mulberry32 generator at that seed, wrapping at 32 bits as every edition must, so a vector can name a realistic file of any size in a line, the way the original, the JavaScript ergo grew from, makes its test files; Mulberry32 is a well-known small PRNG by Tommy Ettinger from 2018
func mulberryData(seed uint32, length int) ergo.Data {
	s := seed
	a := make([]byte, length)
	var v uint32
	for i := range a {
		if i%4 == 0 { // a new 32-bit value every 4 bytes
			s += 0x6d2b79f5
			t := s
			t = (t ^ t>>15) * (t | 1)
			t ^= t + (t^t>>7)*(t|61)
			v = t ^ t>>14
		}
		a[i] = byte(v >> ((i % 4) * 8)) // the appropriate byte from it, low byte first
	}
	return ergo.DataFromArray(a)
}

// bytes is argument i as a Data, from either tagged form, or a panic naming the argument, which Call turns into an error
func bytes(args []any, i int) ergo.Data {
	if i >= len(args) {
		panic(fmt.Sprintf("argument %d is missing", i))
	}
	d, ok, err := tagged(args[i])
	if !ok || err != nil {
		panic(fmt.Sprintf("argument %d must be bytes", i))
	}
	return d
}

// pieces is argument i as a list of pieces, each a Data
func pieces(args []any, i int) []ergo.Data {
	if i >= len(args) {
		panic(fmt.Sprintf("argument %d is missing", i))
	}
	list, ok := args[i].([]any)
	if !ok {
		panic(fmt.Sprintf("argument %d must be a list of pieces", i))
	}
	out := make([]ergo.Data, len(list))
	for j, v := range list {
		d, ok, err := tagged(v)
		if !ok || err != nil {
			panic(fmt.Sprintf("piece %d of argument %d must be bytes", j, i))
		}
		out[j] = d
	}
	return out
}

// tagged decodes bytes from the protocol's tagged json into a Data: {"base16": "616263"} is those three bytes, and {"repeat": "61", "length": 1000000} is those bytes repeated to that length, so a vector can name a million bytes in a line; ok is false if v is neither form, and err is set if it's one of them but malformed
func tagged(v any) (d ergo.Data, ok bool, err error) {
	m, isMap := v.(map[string]any)
	if !isMap {
		return ergo.Data{}, false, nil
	}
	if s, isText := m["base16"].(string); isText {
		d, err = ergo.DataFromBase16(s) // ergo's own base16 reader, which takes exactly what its writer makes, lowercase and nothing else
		return d, true, err
	}
	if seed, isNumber := m["mulberry"].(float64); isNumber {
		length, isLength := m["length"].(float64)
		if !isLength || seed < 0 || seed > math.MaxUint32 || length < 0 || length != math.Trunc(length) {
			return ergo.Data{}, true, fmt.Errorf("not bytes: %v", v)
		}
		return mulberryData(uint32(seed), int(length)), true, nil
	}
	s, isText := m["repeat"].(string)
	if !isText {
		return ergo.Data{}, false, nil
	}
	unit, err := ergo.DataFromBase16(s)
	length, isNumber := m["length"].(float64) // a json number, which must be a count within the integer ceiling
	if err != nil || !isNumber || length < 0 || length != math.Trunc(length) || length > 1<<53-1 || unit.Size() == 0 {
		return ergo.Data{}, true, fmt.Errorf("not bytes: %v", v) // a unit of at least one byte, the same rule in every edition
	}
	b := make([]byte, int(length))
	for j := range b {
		b[j] = unit.Array()[j%unit.Size()]
	}
	return ergo.DataFromArray(b), true, nil
}

// base16 is a result that is a Data, tagged the first way, as its base16
func base16(d ergo.Data) map[string]any {
	return map[string]any{"base16": d.Base16()}
}

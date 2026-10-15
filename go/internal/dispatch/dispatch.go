// Package dispatch maps the protocol's function names, which are the JavaScript edition's, to this edition's functions, with json-decoded arguments and results. The vector reader and the driver both use it; being internal, it is not part of the library's api.
package dispatch

import (
	"fmt"
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

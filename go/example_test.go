package ergo_test

import (
	"fmt"

	ergo "github.com/zootella/ergo/go"
)

// #region size
func ExampleSaySize4() {
	buffer := make([]byte, 64*ergo.KB) // a buffer of 64 kibibytes, rather than a bare 65536
	said := ergo.SaySize4(123456789)   // "117 MB": four digits or less, with a unit
	fmt.Println(len(buffer), said)
	// Output: 65536 117 MB
}

// #endregion size

// #region time
func ExampleHour() {
	deadline := 1700000000000 + 2*ergo.Hour // two hours after a tick, which counts milliseconds since 1970
	fmt.Println(deadline)
	// Output: 1700007200000
}

// #endregion time

// #region tick
func ExampleTickToText() {
	t := int64(1344230277300)        // a tick: milliseconds since 1970, the way time.Now().UnixMilli() counts; this one is the Curiosity landing
	text := ergo.TickToText(t)       // "2012aug6.0517.57300", as short as the moment allows
	iso := ergo.TickToISO(t)         // "2012-08-06T05:17:57.300Z", always 24 characters
	back, _ := ergo.TextToTick(text) // each form reads back to exactly t, and refuses any text its writer wouldn't produce
	asked := ergo.HasTick(-1)        // false: ask about a number from outside, like a clock or a file's modified time, before handing it to a function that takes a tick
	fmt.Println(text, iso, back == t, asked)
	// Output: 2012aug6.0517.57300 2012-08-06T05:17:57.300Z true false
}

// #endregion tick

// #region tag
func ExampleTag() {
	t := ergo.Tag()                     // a new tag, like "AgKxru95C7jFp5iPuK9O7": 21 letters and numerals from the operating system's random source
	valid := ergo.HasTag(t)             // true
	invalid := ergo.HasTag("not a tag") // false
	same := ergo.CheckTag(t)            // returns t, or panics because the caller handed it something that isn't a tag
	fmt.Println(len(t), valid, invalid, same == t)
	// Output: 21 true false true
}

// #endregion tag

package ergo_test

import (
	"fmt"
	"io"
	"strings"

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

// #region hash
func ExampleSHA256() {
	d := ergo.SHA256([]byte("hello"))                                   // all at once, for bytes already in memory: 32 bytes
	file := strings.NewReader("hello")                                  // or os.Open(path), a file of any size
	h1, h256, hb3 := ergo.NewSHA1(), ergo.NewSHA256(), ergo.NewBlake3() // or a piece at a time, all three from one read of the file
	if _, err := io.Copy(io.MultiWriter(h1, h256, hb3), file); err != nil {
		panic(err) // a hasher never fails a write, so an error here is the file's
	}
	fmt.Printf("%x\n%x\n%x\n%x\n", d, h1.Sum(nil), h256.Sum(nil), hb3.Sum(nil))
	// Output:
	// 2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824
	// aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d
	// 2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824
	// ea8f163db38682925e4491c5e58d4bb3506ef8c14eb78a86e908c5624a67200f
}

// #endregion hash

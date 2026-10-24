package ergo_test

import (
	"fmt"
	"io"
	"strings"

	ergo "github.com/zootella/ergo/go"
)

// #region int
func ExampleHasInt() {
	carried := ergo.HasInt(9007199254740991) // 2^53 - 1, the largest integer every edition holds exactly
	past := ergo.HasInt(1 << 53)             // past the ceiling, where a double can't tell one integer from the next
	checked := ergo.CheckInt(1344230277300)  // passes an integer through, or panics because the caller handed it something that isn't one
	fmt.Println(carried, past, checked)
	// Output: true false 1344230277300
}

// #endregion int

// #region text
func ExampleTrim() {
	name := ergo.Trim("  Alice  ")  // whitespace off both ends, by Unicode's list in every edition
	blank := ergo.HasText("   ")    // nothing there but whitespace
	c := ergo.Cut("key=value", "=") // found, with "key" before the tag and "value" after it
	note := ergo.Deindent(`
		line one
		  line two
	`) // the first line's indent comes off every line, and a tab at the start of a line is two spaces of it
	fmt.Printf("%q %v %v %q %q %q\n", name, blank, c.Found, c.Before, c.After, note)
	// Output: "Alice" false true "key" "value" "line one\n  line two\n"
}

// #endregion text

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

// #region sayduration
func ExampleSayDuration() {
	took := ergo.SayDuration(2*ergo.Hour+5*ergo.Minute, "# hours # minutes") // the template is the sentence, with # where each number goes
	ago := ergo.SayDuration(3*ergo.Day, "about # days ago")
	left := ergo.SayTimeRemaining(9*ergo.Minute + 59*ergo.Second)  // one or two units, whichever matter at the size
	race := ergo.SayTimeRace(5*ergo.Minute + 15*ergo.Second + 223) // to the millisecond, the way a race is timed
	fmt.Println(took + " | " + ago + " | " + left + " | " + race)
	// Output: 2 hours 5 minutes | about 3 days ago | 9m 59s | 5'15"223
}

// #endregion sayduration

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

// #region saytick
func ExampleSayTick() {
	t := int64(1344230277300)                           // the Curiosity landing
	page := ergo.SayTick(t, "Dddd, Mmmm D, YYYY")       // uppercase tags are the calendar
	log := ergo.SayTick(t, "YYYY-MM-DD hh24:mm:ss.ttt") // lowercase tags are the clock
	clock := ergo.SayTick(t, "h12:mm AA")
	fmt.Println(page + " | " + log + " | " + clock)
	// Output: Monday, August 6, 2012 | 2012-08-06 05:17:57.300 | 5:17 AM
}

// #endregion saytick

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

// #region data
func ExampleData() {
	d := ergo.DataFromText("hello")                             // bytes, from text
	fmt.Println(d.Base16(), d.Base32(), d.Base62(), d.Base64()) // out in any form
	back, _ := ergo.DataFromBase62("Q6LhR6z")                   // and back from any, each read exactly
	text, _ := back.Text()
	part, _ := d.ClipView(1, 3).Text() // part of it, without copying
	b := ergo.NewBin(6)                // a set number of bytes, filled a byte or a Data at a time
	b.Add(d)
	b.AddByte('!')
	whole, _ := b.Data().Text()
	fmt.Println(text, part, whole)
	// Output:
	// 68656c6c6f NBSWY3DP Q6LhR6z aGVsbG8=
	// hello el hello!
}

// #endregion data

// #region hash
func ExampleSHA256() {
	d := ergo.SHA256(ergo.DataFromText("hello"))                        // all at once, for bytes already in memory: a Data of 32 bytes, out in any form
	file := strings.NewReader("hello")                                  // or os.Open(path), a file of any size
	h1, h256, hb3 := ergo.NewSHA1(), ergo.NewSHA256(), ergo.NewBlake3() // or a piece at a time, all three from one read of the file
	if _, err := io.Copy(io.MultiWriter(h1, h256, hb3), file); err != nil {
		panic(err) // a hasher never fails a write, so an error here is the file's
	}
	for _, h := range []ergo.Data{d, h1.Digest(), h256.Digest(), hb3.Digest()} {
		fmt.Println(h.Base16())
	}
	// Output:
	// 2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824
	// aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d
	// 2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824
	// ea8f163db38682925e4491c5e58d4bb3506ef8c14eb78a86e908c5624a67200f
}

// #endregion hash

// #region fuji
func ExampleHashFile() {
	file := strings.NewReader("hello")      // or os.Open(path), which can read at offsets too, and its size
	status, _ := ergo.HashFile(file, 5)     // the tip hash, from a few small reads
	all, _ := ergo.HashStream(file, 5, nil) // the piece hash, from one read of all of it, which gives the tip hash too
	fmt.Println(status.TipHash.Base32())
	fmt.Println(all.PieceHash.Base32())
	// Output:
	// BSOEFHWYKUFE2ZEYFGKAE2X4IZXTXXCDJZQ2YRGKLMAETUJDTIHQ
	// AD4G5U6L4LJC4DYUIUSFRIYHH5KMRVHCLOQQAKAPNNPFCRAUIV3Q
}

func ExampleFujiHasher() {
	hasher := ergo.NewFujiHasher(5) // or feed one the pieces as they arrive, like the chunks of an upload
	hasher.Update(ergo.DataFromText("he"))
	hasher.Update(ergo.DataFromText("llo"))
	both, _ := hasher.Digest() // both hashes, the same as from the stream
	fmt.Println(both.TipHash.Base32())
	fmt.Println(both.PieceHash.Base32())
	// Output:
	// BSOEFHWYKUFE2ZEYFGKAE2X4IZXTXXCDJZQ2YRGKLMAETUJDTIHQ
	// AD4G5U6L4LJC4DYUIUSFRIYHH5KMRVHCLOQQAKAPNNPFCRAUIV3Q
}

// #endregion fuji

// #region outline
func ExampleOutline() {
	o, err := ergo.TextToOutline(ergo.Deindent(`
		orchard:
		  tree:"apple"
		  basket:
		    color:"red"  # a comment, which the reader skips
	`)) // read from text written by hand, contents in any order
	if err != nil {
		panic(err)
	}
	tree, _ := o.N("tree").Value().Text()                      // "apple": N navigates to a name
	o.M("basket").M("count").SetValue(ergo.DataFromText("12")) // M makes as it navigates, so a chain builds as it goes
	text := o.Text()                                           // written back sorted, at every level, two spaces an indent
	back, _ := ergo.DataToOutline(o.Data())                    // the bytes, the canonical form, read back to the same outline
	fmt.Println(tree, back.Text() == text)
	fmt.Print(text)
	// Output:
	// apple true
	// orchard:
	//   basket:
	//     color:"red"
	//     count:"12"
	//   tree:"apple"
}

// #endregion outline

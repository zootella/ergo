package ergo_test

import (
	"testing"

	ergo "github.com/zootella/ergo/go"
)

// TestOutlineDeep has two chains ten thousand levels down write, read back, sort, and compare. Every walk keeps a stack of its own, so an outline goes as deep as memory allows. Go's own stack grows to a gigabyte, so here, unlike in the other editions, a walk that called itself would pass this too, and the test shows that depth works rather than how.
func TestOutlineDeep(t *testing.T) {
	// the text form isn't here, since a line's indent grows with its depth, so the text of an outline this deep would be a hundred million spaces
	chain := func(bottom string) *ergo.Outline {
		top := ergo.NewOutline("a")
		o := top
		for range 10000 {
			o = o.M("a")
		}
		d, err := ergo.DataFromBase16(bottom)
		if err != nil {
			t.Fatal(err)
		}
		o.SetValue(d)
		return top
	}
	bottomOf := func(o *ergo.Outline) *ergo.Outline {
		for o.Length() > 0 {
			o = o.Get(0)
		}
		return o
	}

	o := ergo.NewOutline("x")
	o.Add(chain("01"))
	o.Add(chain("00")) // the same all the way down but the value at the bottom, and in the wrong order, so putting them in order compares ten thousand levels deep
	d := o.Data()
	back, err := ergo.DataToOutline(d)
	if err != nil {
		t.Fatal(err)
	}
	if back.Data().Base16() != d.Base16() {
		t.Error("the bytes did not round-trip")
	}
	if bottomOf(back.Get(0)).Value().Base16() != "00" {
		t.Error("the bytes did not come out in order")
	}
	o.Sort()
	if bottomOf(o.Get(0)).Value().Base16() != "00" {
		t.Error("sort did not put them in order")
	}
}

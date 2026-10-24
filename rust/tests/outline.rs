//! Outline's own test in this edition, beside the shared vectors: every walk keeps a stack of its own, so an outline goes as deep as memory allows.

use zootella_ergo::{Data, Outline, data_to_outline};

/// Two chains ten thousand levels down write, read back, sort, and compare, and then drop, on a test thread's stack of 2 MB, where a walk that called itself, or an outline that freed its contents by recursion, would abort the test.
#[test]
fn outline_goes_deep() {
    // the text form isn't here, since a line's indent grows with its depth, so the text of an outline this deep would be a hundred million spaces
    let chain = |bottom: &str| {
        let mut top = Outline::new("a");
        let mut o = &mut top;
        for _ in 0..10_000 {
            o = o.m("a");
        }
        o.set_value(Data::from_base16(bottom).expect("base16"));
        top
    };
    let bottom_of = |o: &Outline| {
        let mut o = o;
        while o.length() > 0 {
            o = o.get(0);
        }
        o.value().base16()
    };

    let mut o = Outline::new("x");
    o.add(chain("01"));
    o.add(chain("00")); // the same all the way down but the value at the bottom, and in the wrong order, so putting them in order compares ten thousand levels deep
    let d = o.data();
    let back = data_to_outline(&d).expect("the bytes read back");
    assert_eq!(back.data(), d);
    assert_eq!(bottom_of(back.get(0)), "00"); // the bytes came out in order
    o.sort();
    assert_eq!(bottom_of(o.get(0)), "00"); // and sort put them in order, too
}

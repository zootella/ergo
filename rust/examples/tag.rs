//! Tag: make one, recognize one. Run with cargo run --example tag

use zootella_ergo::{check_tag, has_tag, tag};

fn main() {
    // #region tag
    let t = tag(); // a new tag, like "AgKxru95C7jFp5iPuK9O7": 21 letters and numerals from the operating system's random source
    let valid = has_tag(&t); // true
    let invalid = has_tag("not a tag"); // false
    let same = check_tag(&t); // returns t, or panics because the caller handed it something that isn't a tag
    // #endregion tag
    println!("{t} {valid} {invalid} {same}");
}

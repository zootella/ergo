//! Int: ask whether a number is an integer every edition holds exactly. Run with cargo run --example int

use zootella_ergo::{check_int, has_int};

fn main() {
    // #region int
    let carried = has_int(9_007_199_254_740_991); // true: 2^53 - 1, the largest integer every edition holds exactly
    let past = has_int(1 << 53); // false: past the ceiling, where a double can't tell one integer from the next
    let checked = check_int(1_344_230_277_300); // passes an integer through, or panics because the caller handed it something that isn't one
    // #endregion int
    println!("{carried} {past} {checked}");
}

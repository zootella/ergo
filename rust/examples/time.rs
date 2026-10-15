//! Time: name a span of milliseconds. Run with cargo run --example time

use zootella_ergo::time;

fn main() {
    // #region time
    let deadline = 1_700_000_000_000 + 2 * time::HOUR; // two hours after a tick, which counts milliseconds since 1970
    // #endregion time
    println!("{deadline}");
}

//! Size: name a number of bytes, and say one. Run with cargo run --example size

use zootella_ergo::{say_size4, size};

fn main() {
    // #region size
    let buffer = vec![0u8; 64 * size::KB as usize]; // a buffer of 64 kibibytes, rather than a bare 65536
    let said = say_size4(123_456_789); // "117 MB": four digits or less, with a unit
    // #endregion size
    println!("{} {said}", buffer.len());
}

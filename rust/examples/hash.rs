//! Hash: SHA-1, SHA-256, and BLAKE3, all at once or a piece at a time. Run with cargo run --example hash

use std::io::Read;
use zootella_ergo::{Blake3, Data, Sha1, Sha256, sha256, size};

fn main() -> std::io::Result<()> {
    // #region hash
    let d = sha256(&Data::from_text("hello")); // all at once, for bytes already in memory: a Data of 32 bytes, out in any form
    let mut file = &b"hello"[..]; // or File::open(path)?, a file of any size
    let (mut h1, mut h256, mut hb3) = (Sha1::new(), Sha256::new(), Blake3::new()); // or a piece at a time, all three from one read of the file
    let mut buffer = vec![0; 64 * size::KB as usize];
    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        let piece = Data::from_array(&buffer[..n]); // each hasher is also an io::Write, for feeding the raw read without a Data
        h1.update(&piece);
        h256.update(&piece);
        hb3.update(&piece);
    }
    // #endregion hash
    for h in [d, h1.finalize(), h256.finalize(), hb3.finalize()] {
        println!("{}", h.base16());
    }
    Ok(())
}
